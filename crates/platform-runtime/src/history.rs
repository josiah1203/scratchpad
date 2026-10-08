//! Snapshot checkpoints, named project versions, and materialized-state recovery.
//!
//! Snapshots are immutable, checksummed graph images. Recovery prefers a valid
//! materialized view, otherwise it restores the newest usable snapshot and
//! deterministically replays later journal operations. Operation history and
//! source assets are never pruned by this module.

use std::{error::Error, fmt};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use crate::{
    asset_store::{AssetError, LocalAssetStore},
    command::{CommandService, ReplayError},
    graph::{AssetId, GraphError, ObjectId, ProjectGraph},
    journal::{JournalCursor, JournalError, LocalJournal, graph_digest, materialized_digest},
};

const SNAPSHOT_FORMAT_VERSION: u32 = 1;
const RUNTIME_VERSION: &str = env!("CARGO_PKG_VERSION");

/// An immutable graph checkpoint and the logical journal position it includes.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub snapshot_id: ObjectId,
    pub project_id: ObjectId,
    pub snapshot_format_version: u32,
    pub graph_schema_version: u32,
    pub runtime_version: String,
    pub cursor: Option<JournalCursor>,
    pub graph_json: String,
    pub record_sha256: String,
}

/// A human-readable immutable reference to a snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NamedVersion {
    pub version_id: ObjectId,
    pub project_id: ObjectId,
    pub name: String,
    pub snapshot_id: ObjectId,
}

/// A named version resolved to its exact graph and asset references.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedVersion {
    pub version: NamedVersion,
    pub snapshot: Snapshot,
    pub graph: ProjectGraph,
}

/// The source used to recover the current materialized project.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoverySource {
    MaterializedView,
    Snapshot(ObjectId),
}

/// Evidence returned after a successful recovery attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    pub graph: ProjectGraph,
    pub source: RecoverySource,
    pub replayed_operations: usize,
    pub materialized_repaired: bool,
    pub materialized_issue: Option<String>,
    pub skipped_snapshots: Vec<String>,
}

/// Failures while checkpointing, resolving, or rebuilding project history.
#[derive(Debug)]
pub enum HistoryError {
    Journal(JournalError),
    Graph(GraphError),
    Asset(AssetError),
    Replay(ReplayError),
    InvalidIdentity(GraphError),
    EmptyVersionName,
    SnapshotIdCollision(ObjectId),
    VersionIdCollision(ObjectId),
    VersionNameCollision(String),
    SnapshotNotFound(ObjectId),
    NamedVersionNotFound(String),
    InvalidSnapshot { snapshot_id: ObjectId, reason: String },
    MaterializedStateUnavailable(ObjectId),
    MaterializedStateMismatch(ObjectId),
    InvalidCursor(String),
    CursorNotInJournal(JournalCursor),
    AssetLengthMismatch { id: AssetId, expected: u64, actual: u64 },
    NoRecoveryBase { project_id: ObjectId, diagnostics: Vec<String> },
}

impl fmt::Display for HistoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Journal(error) => error.fmt(formatter),
            Self::Graph(error) | Self::InvalidIdentity(error) => error.fmt(formatter),
            Self::Asset(error) => error.fmt(formatter),
            Self::Replay(error) => error.fmt(formatter),
            Self::EmptyVersionName => formatter.write_str("named version name must not be empty"),
            Self::SnapshotIdCollision(id) => {
                write!(formatter, "snapshot ID {} already exists", id.as_str())
            }
            Self::VersionIdCollision(id) => {
                write!(formatter, "version ID {} already exists", id.as_str())
            }
            Self::VersionNameCollision(name) => {
                write!(formatter, "project already has a named version called {name:?}")
            }
            Self::SnapshotNotFound(id) => {
                write!(formatter, "snapshot {} does not exist", id.as_str())
            }
            Self::NamedVersionNotFound(name) => {
                write!(formatter, "named version {name:?} does not exist")
            }
            Self::InvalidSnapshot { snapshot_id, reason } => {
                write!(formatter, "snapshot {} is invalid: {reason}", snapshot_id.as_str())
            }
            Self::MaterializedStateUnavailable(id) => write!(
                formatter,
                "project {} has journal operations but no trusted materialized graph",
                id.as_str()
            ),
            Self::MaterializedStateMismatch(id) => write!(
                formatter,
                "project {} materialized graph does not match the requested checkpoint",
                id.as_str()
            ),
            Self::InvalidCursor(reason) => write!(formatter, "invalid history cursor: {reason}"),
            Self::CursorNotInJournal(cursor) => write!(
                formatter,
                "journal cursor ({}, {}, {}) is not present in the project journal",
                cursor.lamport, cursor.client_id, cursor.client_sequence
            ),
            Self::AssetLengthMismatch { id, expected, actual } => write!(
                formatter,
                "asset {} has {actual} bytes; graph metadata records {expected}",
                id.as_str()
            ),
            Self::NoRecoveryBase { project_id, diagnostics } => write!(
                formatter,
                "project {} has no usable materialized graph or snapshot ({})",
                project_id.as_str(),
                diagnostics.join("; ")
            ),
        }
    }
}

impl Error for HistoryError {}

impl From<JournalError> for HistoryError {
    fn from(error: JournalError) -> Self {
        Self::Journal(error)
    }
}

impl From<GraphError> for HistoryError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

impl From<AssetError> for HistoryError {
    fn from(error: AssetError) -> Self {
        Self::Asset(error)
    }
}

impl From<ReplayError> for HistoryError {
    fn from(error: ReplayError) -> Self {
        Self::Replay(error)
    }
}

impl From<rusqlite::Error> for HistoryError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Journal(JournalError::Sqlite(error))
    }
}

impl LocalJournal {
    /// Saves an immutable checkpoint of the current project graph.
    ///
    /// A checkpoint is accepted only when its graph matches the current trusted
    /// materialized view and journal cursor. The initial graph may be checkpointed
    /// before any operations exist. Every referenced asset is verified first.
    ///
    /// # Errors
    ///
    /// Returns [`HistoryError`] for invalid graphs or IDs, missing/corrupt assets,
    /// a stale materialized graph, or database failures.
    pub fn create_snapshot(
        &mut self,
        snapshot_id: &ObjectId,
        graph: &ProjectGraph,
        assets: &LocalAssetStore,
    ) -> Result<Snapshot, HistoryError> {
        self.save_snapshot(snapshot_id, graph, assets, None)
    }

    /// Saves a snapshot and binds a unique human-readable name to it.
    ///
    /// Named versions are immutable. Reusing the same project/name or version ID
    /// is rejected; renaming and deletion are intentionally not implicit.
    ///
    /// # Errors
    ///
    /// Returns [`HistoryError`] for invalid identities, name collisions, an
    /// untrusted graph, asset failures, or database failures.
    pub fn create_named_version(
        &mut self,
        version_id: &ObjectId,
        snapshot_id: &ObjectId,
        name: &str,
        graph: &ProjectGraph,
        assets: &LocalAssetStore,
    ) -> Result<NamedVersion, HistoryError> {
        if name.trim().is_empty() {
            return Err(HistoryError::EmptyVersionName);
        }
        version_id.validate("version_id").map_err(HistoryError::InvalidIdentity)?;
        let named = NamedVersion {
            version_id: version_id.clone(),
            project_id: graph.project_id.clone(),
            name: name.to_owned(),
            snapshot_id: snapshot_id.clone(),
        };
        self.save_snapshot(snapshot_id, graph, assets, Some(&named))?;
        Ok(named)
    }

    /// Resolves a named version to its snapshot graph and verifies its assets.
    ///
    /// # Errors
    ///
    /// Returns [`HistoryError::NamedVersionNotFound`] if absent, or a validation,
    /// integrity, or database error if the referenced data is damaged.
    pub fn resolve_named_version(
        &self,
        project_id: &ObjectId,
        name: &str,
        assets: &LocalAssetStore,
    ) -> Result<ResolvedVersion, HistoryError> {
        let raw = self
            .connection
            .query_row(
                "SELECT version_id, project_id, name, snapshot_id
                 FROM named_versions WHERE project_id = ?1 AND name = ?2",
                params![project_id.as_str(), name],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?;
        let Some((version_id, stored_project_id, version_name, snapshot_id)) = raw else {
            return Err(HistoryError::NamedVersionNotFound(name.to_owned()));
        };
        let version = NamedVersion {
            version_id: parse_object_id(version_id, "version_id")?,
            project_id: parse_object_id(stored_project_id, "named_version.project_id")?,
            name: version_name,
            snapshot_id: parse_object_id(snapshot_id, "named_version.snapshot_id")?,
        };
        let snapshot = read_snapshot(&self.connection, &version.snapshot_id)?
            .ok_or_else(|| HistoryError::SnapshotNotFound(version.snapshot_id.clone()))?;
        if version.project_id != *project_id || snapshot.project_id != *project_id {
            return Err(HistoryError::InvalidSnapshot {
                snapshot_id: snapshot.snapshot_id.clone(),
                reason: "named version points outside the requested project".to_owned(),
            });
        }
        let graph = validate_snapshot(&snapshot, project_id, assets)?;
        Ok(ResolvedVersion { version, snapshot, graph })
    }

    /// Lists named versions for a project in deterministic insertion order.
    ///
    /// # Errors
    ///
    /// Returns [`HistoryError`] when the database contains invalid version IDs
    /// or cannot be read.
    pub fn list_named_versions(
        &self,
        project_id: &ObjectId,
    ) -> Result<Vec<NamedVersion>, HistoryError> {
        let mut statement = self.connection.prepare(
            "SELECT version_id, project_id, name, snapshot_id FROM named_versions
             WHERE project_id = ?1 ORDER BY rowid",
        )?;
        let rows = statement.query_map([project_id.as_str()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        rows.map(|row| {
            let (version_id, project_id, name, snapshot_id) = row?;
            Ok(NamedVersion {
                version_id: parse_object_id(version_id, "version_id")?,
                project_id: parse_object_id(project_id, "named_version.project_id")?,
                name,
                snapshot_id: parse_object_id(snapshot_id, "named_version.snapshot_id")?,
            })
        })
        .collect()
    }

    /// Lists a project's snapshots in deterministic recovery order and validates
    /// each record, cursor, graph, and referenced asset before returning it.
    ///
    /// # Errors
    ///
    /// Returns [`HistoryError`] if any stored snapshot or its referenced data is
    /// invalid, or if the database cannot be read.
    pub fn list_snapshots(
        &self,
        project_id: &ObjectId,
        assets: &LocalAssetStore,
    ) -> Result<Vec<Snapshot>, HistoryError> {
        let snapshots = read_snapshot_candidates(&self.connection, project_id)?;
        for snapshot in &snapshots {
            validate_snapshot(snapshot, project_id, assets)?;
            validate_cursor(&self.connection, project_id, snapshot.cursor.as_ref())?;
        }
        Ok(snapshots)
    }

    /// Repairs or rebuilds a project's materialized graph from trusted history.
    ///
    /// The current materialized graph is used when its digest, schema, project
    /// identity, cursor, and assets validate. Otherwise snapshots are tried from
    /// newest to oldest; each candidate is restored and later operations replayed
    /// before it is committed as the new materialized view. Invalid candidates
    /// are reported, never silently rewritten or deleted.
    ///
    /// # Errors
    ///
    /// Returns [`HistoryError::NoRecoveryBase`] when neither materialized state
    /// nor any snapshot can produce a valid graph, or a journal/database error.
    pub fn recover_project(
        &mut self,
        project_id: &ObjectId,
        assets: &LocalAssetStore,
    ) -> Result<RecoveryReport, HistoryError> {
        project_id.validate("recovery.project_id").map_err(HistoryError::InvalidIdentity)?;
        let latest = latest_cursor(&self.connection, project_id)?;
        let mut diagnostics = Vec::new();
        let materialized_issue;

        if let Some(state) = read_materialized(&self.connection, project_id)? {
            match validate_materialized(&state, project_id, assets).and_then(|graph| {
                validate_cursor(&self.connection, project_id, state.cursor.as_ref()).map(|()| graph)
            }) {
                Ok(mut graph) => {
                    match CommandService::replay_from(
                        &mut graph,
                        self,
                        project_id,
                        state.cursor.clone(),
                    ) {
                        Ok(replayed_operations) => {
                            verify_graph_assets(&graph, assets)?;
                            if replayed_operations > 0 {
                                persist_materialized(self, &graph, latest.as_ref())?;
                            }
                            return Ok(RecoveryReport {
                                graph,
                                source: RecoverySource::MaterializedView,
                                replayed_operations,
                                materialized_repaired: replayed_operations > 0,
                                materialized_issue: None,
                                skipped_snapshots: Vec::new(),
                            });
                        }
                        Err(error) => {
                            let reason = format!("materialized view replay failed: {error}");
                            materialized_issue = Some(reason.clone());
                            diagnostics.push(reason);
                        }
                    }
                }
                Err(error) => {
                    let reason = format!("materialized view rejected: {error}");
                    materialized_issue = Some(reason.clone());
                    diagnostics.push(reason);
                }
            }
        } else {
            diagnostics.push("materialized view is missing".to_owned());
            materialized_issue = Some("materialized view is missing".to_owned());
        }

        let mut skipped_snapshots = Vec::new();
        for snapshot in read_snapshot_candidates(&self.connection, project_id)? {
            let snapshot_id = snapshot.snapshot_id.clone();
            let restored =
                match validate_snapshot(&snapshot, project_id, assets).and_then(|graph| {
                    validate_cursor(&self.connection, project_id, snapshot.cursor.as_ref())
                        .map(|()| graph)
                }) {
                    Ok(graph) => graph,
                    Err(error) => {
                        let diagnostic = format!("{}: {error}", snapshot_id.as_str());
                        skipped_snapshots.push(diagnostic.clone());
                        diagnostics.push(diagnostic);
                        continue;
                    }
                };

            let mut graph = restored;
            match CommandService::replay_from(&mut graph, self, project_id, snapshot.cursor.clone())
            {
                Ok(replayed_operations) => {
                    if let Err(error) = verify_graph_assets(&graph, assets) {
                        let diagnostic = format!("{} replay result: {error}", snapshot_id.as_str());
                        skipped_snapshots.push(diagnostic.clone());
                        diagnostics.push(diagnostic);
                        continue;
                    }
                    persist_materialized(self, &graph, latest.as_ref())?;
                    return Ok(RecoveryReport {
                        graph,
                        source: RecoverySource::Snapshot(snapshot_id),
                        replayed_operations,
                        materialized_repaired: true,
                        materialized_issue,
                        skipped_snapshots,
                    });
                }
                Err(error) => {
                    let diagnostic = format!("{} replay failed: {error}", snapshot_id.as_str());
                    skipped_snapshots.push(diagnostic.clone());
                    diagnostics.push(diagnostic);
                }
            }
        }

        Err(HistoryError::NoRecoveryBase { project_id: project_id.clone(), diagnostics })
    }

    fn save_snapshot(
        &mut self,
        snapshot_id: &ObjectId,
        graph: &ProjectGraph,
        assets: &LocalAssetStore,
        named_version: Option<&NamedVersion>,
    ) -> Result<Snapshot, HistoryError> {
        snapshot_id.validate("snapshot_id").map_err(HistoryError::InvalidIdentity)?;
        graph.validate()?;
        verify_graph_assets(graph, assets)?;
        let graph_json = serde_json::to_string(graph).map_err(JournalError::from)?;
        let mut snapshot = Snapshot {
            snapshot_id: snapshot_id.clone(),
            project_id: graph.project_id.clone(),
            snapshot_format_version: SNAPSHOT_FORMAT_VERSION,
            graph_schema_version: graph.schema_version,
            runtime_version: RUNTIME_VERSION.to_owned(),
            cursor: None,
            record_sha256: String::new(),
            graph_json,
        };

        let transaction =
            self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let cursor = latest_cursor(&transaction, &graph.project_id)?;
        let materialized = read_materialized(&transaction, &graph.project_id)?;
        ensure_checkpoint_matches(graph, materialized.as_ref(), cursor.as_ref())?;
        if let Some(named) = named_version {
            let version_id_exists: bool = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM named_versions WHERE version_id = ?1)",
                [named.version_id.as_str()],
                |row| row.get(0),
            )?;
            if version_id_exists {
                return Err(HistoryError::VersionIdCollision(named.version_id.clone()));
            }
            let name_exists: bool = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM named_versions WHERE project_id = ?1 AND name = ?2)",
                params![named.project_id.as_str(), named.name],
                |row| row.get(0),
            )?;
            if name_exists {
                return Err(HistoryError::VersionNameCollision(named.name.clone()));
            }
        }
        let snapshot_exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM snapshots WHERE snapshot_id = ?1)",
            [snapshot_id.as_str()],
            |row| row.get(0),
        )?;
        if snapshot_exists {
            return Err(HistoryError::SnapshotIdCollision(snapshot_id.clone()));
        }

        snapshot.cursor = cursor;
        snapshot.record_sha256 = snapshot_digest(&snapshot);
        insert_snapshot(&transaction, &snapshot)?;
        if let Some(named) = named_version {
            transaction.execute(
                "INSERT INTO named_versions (version_id, project_id, name, snapshot_id)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    named.version_id.as_str(),
                    named.project_id.as_str(),
                    named.name,
                    named.snapshot_id.as_str(),
                ],
            )?;
        }
        if materialized.is_none() {
            upsert_materialized(
                &transaction,
                graph,
                &snapshot.graph_json,
                snapshot.cursor.as_ref(),
            )?;
        }
        transaction.commit()?;
        Ok(snapshot)
    }
}

#[derive(Clone, Debug)]
struct MaterializedRecord {
    graph_json: String,
    state_sha256: String,
    cursor: Option<JournalCursor>,
}

#[derive(Debug)]
struct StoredSnapshot {
    snapshot_id: String,
    project_id: String,
    snapshot_format_version: i64,
    graph_schema_version: i64,
    runtime_version: String,
    cursor_lamport: Option<i64>,
    cursor_client_id: Option<String>,
    cursor_client_sequence: Option<i64>,
    graph_json: String,
    record_sha256: String,
}

impl StoredSnapshot {
    fn into_snapshot(self) -> Result<Snapshot, HistoryError> {
        let snapshot_id = parse_object_id(self.snapshot_id.clone(), "snapshot_id")?;
        let project_id = parse_object_id(self.project_id, "snapshot.project_id")?;
        let snapshot_format_version =
            u32::try_from(self.snapshot_format_version).map_err(|_| {
                HistoryError::InvalidSnapshot {
                    snapshot_id: snapshot_id.clone(),
                    reason: "snapshot format version is out of range".to_owned(),
                }
            })?;
        let graph_schema_version = u32::try_from(self.graph_schema_version).map_err(|_| {
            HistoryError::InvalidSnapshot {
                snapshot_id: snapshot_id.clone(),
                reason: "graph schema version is out of range".to_owned(),
            }
        })?;
        Ok(Snapshot {
            snapshot_id,
            project_id,
            snapshot_format_version,
            graph_schema_version,
            runtime_version: self.runtime_version,
            cursor: decode_cursor(
                self.cursor_lamport,
                self.cursor_client_id,
                self.cursor_client_sequence,
            )?,
            graph_json: self.graph_json,
            record_sha256: self.record_sha256,
        })
    }
}

fn ensure_checkpoint_matches(
    graph: &ProjectGraph,
    materialized: Option<&MaterializedRecord>,
    cursor: Option<&JournalCursor>,
) -> Result<(), HistoryError> {
    match materialized {
        Some(state) => {
            if materialized_digest(&graph.project_id, &state.graph_json, state.cursor.as_ref())
                != state.state_sha256
            {
                return Err(HistoryError::MaterializedStateMismatch(graph.project_id.clone()));
            }
            let stored_graph = ProjectGraph::from_json(&state.graph_json)?;
            if &stored_graph != graph || state.cursor.as_ref() != cursor {
                return Err(HistoryError::MaterializedStateMismatch(graph.project_id.clone()));
            }
        }
        None if cursor.is_some() => {
            return Err(HistoryError::MaterializedStateUnavailable(graph.project_id.clone()));
        }
        None => {}
    }
    Ok(())
}

fn validate_materialized(
    state: &MaterializedRecord,
    project_id: &ObjectId,
    assets: &LocalAssetStore,
) -> Result<ProjectGraph, HistoryError> {
    if materialized_digest(project_id, &state.graph_json, state.cursor.as_ref())
        != state.state_sha256
    {
        return Err(HistoryError::MaterializedStateMismatch(project_id.clone()));
    }
    let graph = ProjectGraph::from_json(&state.graph_json)?;
    if graph.project_id != *project_id {
        return Err(HistoryError::MaterializedStateMismatch(project_id.clone()));
    }
    verify_graph_assets(&graph, assets)?;
    Ok(graph)
}

fn validate_snapshot(
    snapshot: &Snapshot,
    project_id: &ObjectId,
    assets: &LocalAssetStore,
) -> Result<ProjectGraph, HistoryError> {
    let graph = validate_snapshot_record(snapshot, project_id)?;
    verify_graph_assets(&graph, assets)?;
    Ok(graph)
}

pub(crate) fn validate_snapshot_record(
    snapshot: &Snapshot,
    project_id: &ObjectId,
) -> Result<ProjectGraph, HistoryError> {
    let invalid = |reason: &str| HistoryError::InvalidSnapshot {
        snapshot_id: snapshot.snapshot_id.clone(),
        reason: reason.to_owned(),
    };
    snapshot.snapshot_id.validate("snapshot_id").map_err(HistoryError::InvalidIdentity)?;
    if snapshot.snapshot_format_version != SNAPSHOT_FORMAT_VERSION {
        return Err(invalid("unsupported snapshot format version"));
    }
    if snapshot.project_id != *project_id {
        return Err(invalid("snapshot belongs to a different project"));
    }
    if snapshot_digest(snapshot) != snapshot.record_sha256 {
        return Err(invalid("snapshot record checksum mismatch"));
    }
    let graph = ProjectGraph::from_json(&snapshot.graph_json)
        .map_err(|error| invalid(&format!("graph validation failed: {error}")))?;
    if graph.project_id != snapshot.project_id
        || graph.schema_version != snapshot.graph_schema_version
    {
        return Err(invalid("snapshot graph identity or schema version mismatch"));
    }
    Ok(graph)
}

fn verify_graph_assets(graph: &ProjectGraph, assets: &LocalAssetStore) -> Result<(), HistoryError> {
    graph.validate()?;
    for asset in &graph.asset_refs {
        let verified = assets.verify(&asset.id)?;
        if verified.byte_length != asset.byte_length {
            return Err(HistoryError::AssetLengthMismatch {
                id: asset.id.clone(),
                expected: asset.byte_length,
                actual: verified.byte_length,
            });
        }
    }
    Ok(())
}

fn insert_snapshot(connection: &Connection, snapshot: &Snapshot) -> Result<(), HistoryError> {
    let (lamport, client_id, sequence) = encode_cursor(snapshot.cursor.as_ref())?;
    connection.execute(
        "INSERT INTO snapshots (
            snapshot_id, project_id, snapshot_format_version, graph_schema_version,
            runtime_version, cursor_lamport, cursor_client_id, cursor_client_sequence,
            graph_json, record_sha256
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            snapshot.snapshot_id.as_str(),
            snapshot.project_id.as_str(),
            i64::from(snapshot.snapshot_format_version),
            i64::from(snapshot.graph_schema_version),
            snapshot.runtime_version,
            lamport,
            client_id,
            sequence,
            snapshot.graph_json,
            snapshot.record_sha256,
        ],
    )?;
    Ok(())
}

fn read_snapshot(
    connection: &Connection,
    snapshot_id: &ObjectId,
) -> Result<Option<Snapshot>, HistoryError> {
    let raw = connection
        .query_row(
            "SELECT snapshot_id, project_id, snapshot_format_version, graph_schema_version,
                    runtime_version, cursor_lamport, cursor_client_id, cursor_client_sequence,
                    graph_json, record_sha256
             FROM snapshots WHERE snapshot_id = ?1",
            [snapshot_id.as_str()],
            read_stored_snapshot,
        )
        .optional()?;
    raw.map(StoredSnapshot::into_snapshot).transpose()
}

fn read_snapshot_candidates(
    connection: &Connection,
    project_id: &ObjectId,
) -> Result<Vec<Snapshot>, HistoryError> {
    let mut statement = connection.prepare(
        "SELECT snapshot_id, project_id, snapshot_format_version, graph_schema_version,
                runtime_version, cursor_lamport, cursor_client_id, cursor_client_sequence,
                graph_json, record_sha256
         FROM snapshots WHERE project_id = ?1
         ORDER BY cursor_lamport DESC, cursor_client_id DESC, cursor_client_sequence DESC, created_index DESC",
    )?;
    let rows = statement.query_map([project_id.as_str()], read_stored_snapshot)?;
    rows.map(|row| row?.into_snapshot()).collect()
}

fn read_stored_snapshot(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredSnapshot> {
    Ok(StoredSnapshot {
        snapshot_id: row.get(0)?,
        project_id: row.get(1)?,
        snapshot_format_version: row.get(2)?,
        graph_schema_version: row.get(3)?,
        runtime_version: row.get(4)?,
        cursor_lamport: row.get(5)?,
        cursor_client_id: row.get(6)?,
        cursor_client_sequence: row.get(7)?,
        graph_json: row.get(8)?,
        record_sha256: row.get(9)?,
    })
}

fn read_materialized(
    connection: &Connection,
    project_id: &ObjectId,
) -> Result<Option<MaterializedRecord>, HistoryError> {
    let raw = connection
        .query_row(
            "SELECT graph_json, state_sha256, cursor_lamport, cursor_client_id, cursor_client_sequence
             FROM materialized_projects WHERE project_id = ?1",
            [project_id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                ))
            },
        )
        .optional()?;
    raw.map(|(graph_json, state_sha256, lamport, client_id, sequence)| {
        Ok(MaterializedRecord {
            graph_json,
            state_sha256,
            cursor: decode_cursor(lamport, client_id, sequence)?,
        })
    })
    .transpose()
}

fn latest_cursor(
    connection: &Connection,
    project_id: &ObjectId,
) -> Result<Option<JournalCursor>, HistoryError> {
    let raw = connection
        .query_row(
            "SELECT lamport, client_id, client_sequence FROM operations
             WHERE project_id = ?1 ORDER BY lamport DESC, client_id DESC, client_sequence DESC LIMIT 1",
            [project_id.as_str()],
            |row| {
                Ok((
                    Some(row.get::<_, i64>(0)?),
                    Some(row.get::<_, String>(1)?),
                    Some(row.get::<_, i64>(2)?),
                ))
            },
        )
        .optional()?;
    match raw {
        Some((lamport, client_id, sequence)) => decode_cursor(lamport, client_id, sequence),
        None => Ok(None),
    }
}

fn decode_cursor(
    lamport: Option<i64>,
    client_id: Option<String>,
    sequence: Option<i64>,
) -> Result<Option<JournalCursor>, HistoryError> {
    match (lamport, client_id, sequence) {
        (None, None, None) => Ok(None),
        (Some(lamport), Some(client_id), Some(sequence)) if lamport > 0 && sequence > 0 => {
            if client_id.trim().is_empty() {
                return Err(HistoryError::InvalidCursor("client ID is empty".to_owned()));
            }
            Ok(Some(JournalCursor {
                lamport: u64::try_from(lamport)
                    .map_err(|_| JournalError::IntegerOutOfRange("cursor.lamport"))?,
                client_id,
                client_sequence: u64::try_from(sequence)
                    .map_err(|_| JournalError::IntegerOutOfRange("cursor.client_sequence"))?,
            }))
        }
        _ => Err(HistoryError::InvalidCursor(
            "cursor fields are incomplete or non-positive".to_owned(),
        )),
    }
}

type SqlCursor<'a> = (Option<i64>, Option<&'a str>, Option<i64>);

fn encode_cursor(cursor: Option<&JournalCursor>) -> Result<SqlCursor<'_>, HistoryError> {
    match cursor {
        None => Ok((None, None, None)),
        Some(cursor) => {
            if cursor.client_id.trim().is_empty() {
                return Err(HistoryError::InvalidCursor("client ID is empty".to_owned()));
            }
            if cursor.lamport == 0 || cursor.client_sequence == 0 {
                return Err(HistoryError::InvalidCursor(
                    "Lamport clock and client sequence must be positive".to_owned(),
                ));
            }
            let lamport = i64::try_from(cursor.lamport)
                .map_err(|_| JournalError::IntegerOutOfRange("cursor.lamport"))?;
            let sequence = i64::try_from(cursor.client_sequence)
                .map_err(|_| JournalError::IntegerOutOfRange("cursor.client_sequence"))?;
            Ok((Some(lamport), Some(&cursor.client_id), Some(sequence)))
        }
    }
}

fn validate_cursor(
    connection: &Connection,
    project_id: &ObjectId,
    cursor: Option<&JournalCursor>,
) -> Result<(), HistoryError> {
    let Some(cursor) = cursor else {
        return Ok(());
    };
    let (lamport, client_id, sequence) = encode_cursor(Some(cursor))?;
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM operations
         WHERE project_id = ?1 AND lamport = ?2 AND client_id = ?3 AND client_sequence = ?4)",
        params![project_id.as_str(), lamport, client_id, sequence],
        |row| row.get(0),
    )?;
    if exists { Ok(()) } else { Err(HistoryError::CursorNotInJournal(cursor.clone())) }
}

fn upsert_materialized(
    connection: &Connection,
    graph: &ProjectGraph,
    graph_json: &str,
    cursor: Option<&JournalCursor>,
) -> Result<(), HistoryError> {
    let (lamport, client_id, sequence) = encode_cursor(cursor)?;
    let state_sha256 = materialized_digest(&graph.project_id, graph_json, cursor);
    connection.execute(
        "INSERT INTO materialized_projects (
            project_id, graph_json, state_sha256, cursor_lamport, cursor_client_id, cursor_client_sequence
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(project_id) DO UPDATE SET
            graph_json = excluded.graph_json,
            state_sha256 = excluded.state_sha256,
            cursor_lamport = excluded.cursor_lamport,
            cursor_client_id = excluded.cursor_client_id,
            cursor_client_sequence = excluded.cursor_client_sequence",
        params![graph.project_id.as_str(), graph_json, state_sha256, lamport, client_id, sequence],
    )?;
    Ok(())
}

fn snapshot_digest(snapshot: &Snapshot) -> String {
    let cursor = snapshot.cursor.as_ref().map_or_else(
        || "none".to_owned(),
        |cursor| {
            format!(
                "{}:{}:{}:{}",
                cursor.lamport,
                cursor.client_id.len(),
                cursor.client_id,
                cursor.client_sequence
            )
        },
    );
    let content = format!(
        "snapshot:{}:{};project:{}:{};format:{};graph_schema:{};runtime:{}:{};cursor:{};graph:{}:{}",
        snapshot.snapshot_id.as_str().len(),
        snapshot.snapshot_id.as_str(),
        snapshot.project_id.as_str().len(),
        snapshot.project_id.as_str(),
        snapshot.snapshot_format_version,
        snapshot.graph_schema_version,
        snapshot.runtime_version.len(),
        snapshot.runtime_version,
        cursor,
        snapshot.graph_json.len(),
        snapshot.graph_json
    );
    graph_digest(content.as_bytes())
}

fn persist_materialized(
    journal: &mut LocalJournal,
    graph: &ProjectGraph,
    cursor: Option<&JournalCursor>,
) -> Result<(), HistoryError> {
    graph.validate()?;
    let graph_json = serde_json::to_string(graph).map_err(JournalError::from)?;
    let transaction =
        journal.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    upsert_materialized(&transaction, graph, &graph_json, cursor)?;
    transaction.commit()?;
    Ok(())
}

fn parse_object_id(value: String, context: &str) -> Result<ObjectId, HistoryError> {
    let id = ObjectId::new(value);
    id.validate(context).map_err(HistoryError::InvalidIdentity)?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use std::{
        io::Cursor,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;
    use crate::{
        SAMPLE_PROJECT,
        asset_store::AssetMetadata,
        command::{Actor, CausalContext, CreativeCommand, PropertyState},
        graph::AssetRef,
        journal::AppendOutcome,
    };

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

    struct TestDb {
        directory: tempfile::TempDir,
        path: PathBuf,
    }

    impl TestDb {
        fn new() -> Self {
            let directory = tempfile::tempdir().expect("temporary directory");
            let path = directory.path().join(format!(
                "history-{}-{}.sqlite",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            Self { directory, path }
        }
    }

    fn id(number: u16) -> ObjectId {
        ObjectId::new(format!("00000000-0000-4000-8000-{number:012}"))
    }

    fn graph_with_local_asset(store: &LocalAssetStore) -> ProjectGraph {
        let mut bytes = Cursor::new(b"synthetic image bytes".to_vec());
        let asset = store
            .ingest(&mut bytes, AssetMetadata::new("image", "history-test"))
            .expect("ingest test asset");
        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture graph");
        graph.asset_refs = vec![asset.clone()];
        let node =
            graph.nodes.iter_mut().find(|node| node.id == id(12)).expect("fixture media node");
        node.asset_ref = Some(asset.id);
        graph.validate().expect("updated graph valid");
        graph
    }

    fn set_property_operation() -> crate::command::CommandEnvelope {
        crate::command::CommandEnvelope {
            operation_id: id(200),
            project_id: id(1),
            target_id: id(11),
            schema_version: 1,
            actor: Actor::user("history-test-user"),
            causal: CausalContext::new("history-test-client", 1, 1),
            command: CreativeCommand::SetProperty {
                node_id: id(11),
                property: "history_value".to_owned(),
                value: serde_json::Value::from(42),
                previous_state: PropertyState::Absent,
            },
        }
    }

    fn update_property_operation() -> crate::command::CommandEnvelope {
        crate::command::CommandEnvelope {
            operation_id: id(201),
            project_id: id(1),
            target_id: id(11),
            schema_version: 1,
            actor: Actor::user("history-test-user"),
            causal: CausalContext::new("history-test-client", 2, 2),
            command: CreativeCommand::SetProperty {
                node_id: id(11),
                property: "history_value".to_owned(),
                value: serde_json::Value::from(43),
                previous_state: PropertyState::Present(serde_json::Value::from(42)),
            },
        }
    }

    #[test]
    fn snapshot_fallback_replays_operations_repairs_materialization_and_resolves_version() {
        let database = TestDb::new();
        let asset_store =
            LocalAssetStore::open(database.directory.path(), "tenant-a").expect("asset store");
        let base = graph_with_local_asset(&asset_store);
        let mut journal = LocalJournal::open(&database.path).expect("journal");
        journal.create_snapshot(&id(100), &base, &asset_store).expect("initial checkpoint");

        let first_operation = set_property_operation();
        let mut intermediate = base.clone();
        intermediate
            .nodes
            .iter_mut()
            .find(|node| node.id == id(11))
            .expect("frame node")
            .properties
            .insert("history_value".to_owned(), serde_json::Value::from(42));
        assert_eq!(
            journal
                .append_with_materialized(&first_operation, &base, &intermediate)
                .expect("append first operation with materialized graph"),
            AppendOutcome::Committed
        );
        let second_operation = update_property_operation();
        let mut expected = intermediate.clone();
        expected
            .nodes
            .iter_mut()
            .find(|node| node.id == id(11))
            .expect("frame node")
            .properties
            .insert("history_value".to_owned(), serde_json::Value::from(43));
        assert_eq!(
            journal
                .append_with_materialized(&second_operation, &intermediate, &expected)
                .expect("append second operation with materialized graph"),
            AppendOutcome::Committed
        );
        journal.create_snapshot(&id(101), &expected, &asset_store).expect("later checkpoint");

        let external = rusqlite::Connection::open(&database.path).expect("open corruption handle");
        external
            .execute(
                "UPDATE snapshots SET cursor_lamport = 1, cursor_client_sequence = 1
                 WHERE snapshot_id = ?1",
                [id(101).as_str()],
            )
            .expect("corrupt latest snapshot cursor");
        external
            .execute(
                "UPDATE materialized_projects SET graph_json = '{' WHERE project_id = ?1",
                [id(1).as_str()],
            )
            .expect("corrupt materialized graph");

        let report = journal
            .recover_project(&id(1), &asset_store)
            .expect("recover from earlier snapshot and operation");
        assert_eq!(report.graph, expected);
        assert_eq!(report.source, RecoverySource::Snapshot(id(100)));
        assert_eq!(report.replayed_operations, 2);
        assert!(report.materialized_repaired);
        assert!(report.materialized_issue.is_some());
        assert_eq!(report.skipped_snapshots.len(), 1);
        assert!(report.skipped_snapshots[0].contains(id(101).as_str()));
        assert_eq!(report.graph.asset_refs, expected.asset_refs);
        assert_eq!(
            journal.read(&id(200)).expect("first history read").unwrap().envelope,
            first_operation
        );
        assert_eq!(
            journal.read(&id(201)).expect("second history read").unwrap().envelope,
            second_operation
        );

        let named = journal
            .create_named_version(&id(102), &id(103), "Client review", &expected, &asset_store)
            .expect("create named version");
        let resolved = journal
            .resolve_named_version(&id(1), "Client review", &asset_store)
            .expect("resolve named version");
        assert_eq!(resolved.version, named);
        assert_eq!(resolved.graph, expected);
        assert_eq!(resolved.graph.asset_refs, expected.asset_refs);
        assert_eq!(journal.list_named_versions(&id(1)).expect("list versions"), vec![named]);
        assert!(matches!(
            journal.create_named_version(
                &id(104),
                &id(105),
                "Client review",
                &expected,
                &asset_store
            ),
            Err(HistoryError::VersionNameCollision(_))
        ));
    }

    #[test]
    fn snapshot_creation_rejects_missing_referenced_assets() {
        let database = TestDb::new();
        let asset_store =
            LocalAssetStore::open(database.directory.path(), "tenant-a").expect("asset store");
        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture graph");
        graph.asset_refs.clear();
        graph.nodes.iter_mut().find(|node| node.id == id(12)).unwrap().asset_ref = None;
        graph.validate().expect("graph without external media ref");
        let mut journal = LocalJournal::open(&database.path).expect("journal");
        journal.create_snapshot(&id(110), &graph, &asset_store).expect("snapshot without assets");

        let mut damaged = graph.clone();
        damaged.asset_refs = vec![AssetRef {
            id: crate::graph::AssetId::new(format!("sha256:{}", "b".repeat(64))),
            kind: "image".to_owned(),
            byte_length: 12,
            provenance: "missing-test-asset".to_owned(),
            rights: None,
            source_revision: None,
            transform: None,
        }];
        damaged.nodes.iter_mut().find(|node| node.id == id(12)).unwrap().asset_ref =
            Some(damaged.asset_refs[0].id.clone());
        assert!(matches!(
            journal.create_snapshot(&id(111), &damaged, &asset_store),
            Err(HistoryError::Asset(AssetError::MissingAsset(_)))
        ));
    }

    #[test]
    fn materialized_append_rejects_stale_base_and_logical_order_regression() {
        let database = TestDb::new();
        let asset_store =
            LocalAssetStore::open(database.directory.path(), "tenant-a").expect("asset store");
        let base = graph_with_local_asset(&asset_store);
        let mut journal = LocalJournal::open(&database.path).expect("journal");
        let mut first = set_property_operation();
        first.causal.lamport = 5;
        let mut state_after_first = base.clone();
        state_after_first
            .nodes
            .iter_mut()
            .find(|node| node.id == id(11))
            .expect("frame")
            .properties
            .insert("history_value".to_owned(), serde_json::Value::from(42));
        journal
            .append_with_materialized(&first, &base, &state_after_first)
            .expect("first operation");

        let mut next = update_property_operation();
        let mut state_after_second = state_after_first.clone();
        state_after_second
            .nodes
            .iter_mut()
            .find(|node| node.id == id(11))
            .expect("frame")
            .properties
            .insert("history_value".to_owned(), serde_json::Value::from(43));
        next.causal.lamport = 6;
        assert!(matches!(
            journal.append_with_materialized(&next, &base, &state_after_second),
            Err(JournalError::MaterializedStateMismatch(_))
        ));

        next.causal.lamport = 4;
        assert!(matches!(
            journal.append_with_materialized(&next, &state_after_first, &state_after_second),
            Err(JournalError::CausalOrderRegression(_))
        ));
        assert!(journal.read(&next.operation_id).expect("read rejected op").is_none());
    }
}
