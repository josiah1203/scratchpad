//! SQLite-backed local operation journal.
//!
//! The journal is the local durability boundary: `append` returns only after
//! the transaction commits with WAL mode and `synchronous=FULL`. Remote sync
//! acknowledgment is tracked separately and never changes local commit state.

use std::{fmt, path::Path, time::Duration};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    command::{CommandEnvelope, CommandError},
    graph::{GraphError, ObjectId, ProjectGraph},
};

const JOURNAL_SCHEMA_VERSION: i64 = 2;

/// A persistent local project operation journal.
pub struct LocalJournal {
    pub(crate) connection: Connection,
}

/// Stable ordering cursor for deterministic journal paging.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JournalCursor {
    pub lamport: u64,
    pub client_id: String,
    pub client_sequence: u64,
}

/// Whether a locally committed operation has reached remote storage.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicationState {
    Pending,
    RemoteAcknowledged,
}

/// An operation and its independent local/remote durability state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JournalEntry {
    pub envelope: CommandEnvelope,
    pub replication: ReplicationState,
}

struct MaterializedRow {
    graph_json: String,
    state_sha256: String,
    cursor_lamport: Option<i64>,
    cursor_client_id: Option<String>,
    cursor_client_sequence: Option<i64>,
}

/// Result of appending an operation to the local journal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppendOutcome {
    /// This call committed the operation locally.
    Committed,
    /// The same operation ID and payload were already committed.
    AlreadyCommitted,
}

/// Basic diagnostics for durability and recovery state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JournalHealth {
    pub sqlite_version: String,
    pub schema_version: i64,
    pub journal_mode: String,
    pub synchronous: i64,
    pub integrity: String,
}

impl JournalHealth {
    /// True when `SQLite` reports WAL and FULL synchronous mode, and quick check passes.
    #[must_use]
    pub fn meets_configured_durability(&self) -> bool {
        self.journal_mode.eq_ignore_ascii_case("wal")
            && self.synchronous == 2
            && self.integrity == "ok"
    }
}

/// Journal and serialization errors.
#[derive(Debug)]
pub enum JournalError {
    Sqlite(rusqlite::Error),
    Command(CommandError),
    Graph(GraphError),
    Serialization(serde_json::Error),
    UnsupportedSchema(i64),
    OperationIdCollision(ObjectId),
    ClientSequenceCollision { client_id: String, sequence: u64 },
    UnknownOperation(ObjectId),
    InvalidPageLimit(usize),
    IntegerOutOfRange(&'static str),
    ProjectMismatch { command: ObjectId, graph: ObjectId },
    MaterializedStateUnavailable(ObjectId),
    MaterializedStateMismatch(ObjectId),
    CausalOrderRegression(ObjectId),
}

impl fmt::Display for JournalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(formatter, "journal database error: {error}"),
            Self::Command(error) => write!(formatter, "invalid command envelope: {error}"),
            Self::Graph(error) => write!(formatter, "invalid materialized project graph: {error}"),
            Self::Serialization(error) => write!(formatter, "cannot serialize command: {error}"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported journal schema version {version}")
            }
            Self::OperationIdCollision(id) => write!(
                formatter,
                "operation ID {} was reused with a different payload",
                id.as_str()
            ),
            Self::ClientSequenceCollision { client_id, sequence } => write!(
                formatter,
                "client {client_id} already has an operation at sequence {sequence}"
            ),
            Self::UnknownOperation(id) => {
                write!(formatter, "unknown operation ID {}", id.as_str())
            }
            Self::InvalidPageLimit(limit) => write!(formatter, "invalid page limit {limit}"),
            Self::IntegerOutOfRange(field) => {
                write!(formatter, "{field} exceeds the supported SQLite integer range")
            }
            Self::ProjectMismatch { command, graph } => write!(
                formatter,
                "command project {} does not match materialized graph project {}",
                command.as_str(),
                graph.as_str()
            ),
            Self::MaterializedStateUnavailable(project_id) => write!(
                formatter,
                "project {} has journal operations but no trusted materialized graph",
                project_id.as_str()
            ),
            Self::MaterializedStateMismatch(project_id) => write!(
                formatter,
                "project {} materialized graph does not match the command base graph",
                project_id.as_str()
            ),
            Self::CausalOrderRegression(project_id) => write!(
                formatter,
                "operation would move project {} materialization backwards in logical order",
                project_id.as_str()
            ),
        }
    }
}

impl std::error::Error for JournalError {}

impl From<rusqlite::Error> for JournalError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<CommandError> for JournalError {
    fn from(error: CommandError) -> Self {
        Self::Command(error)
    }
}

impl From<GraphError> for JournalError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

impl From<serde_json::Error> for JournalError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

impl LocalJournal {
    /// Opens or creates a journal file, configuring its durability pragmas and schema.
    ///
    /// # Errors
    ///
    /// Returns [`JournalError`] if `SQLite` cannot open/configure the file or if
    /// the file contains a newer unsupported journal schema.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "FULL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;

        let mut schema_version: i64 =
            connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if schema_version > JOURNAL_SCHEMA_VERSION {
            return Err(JournalError::UnsupportedSchema(schema_version));
        }
        if schema_version == 0 {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(
                "CREATE TABLE operations (
                    operation_id TEXT PRIMARY KEY NOT NULL,
                    project_id TEXT NOT NULL,
                    target_id TEXT NOT NULL,
                    client_id TEXT NOT NULL,
                    client_sequence INTEGER NOT NULL CHECK(client_sequence > 0),
                    lamport INTEGER NOT NULL CHECK(lamport > 0),
                    parent_operation_id TEXT,
                    payload_json TEXT NOT NULL,
                    remote_acknowledged INTEGER NOT NULL DEFAULT 0 CHECK(remote_acknowledged IN (0, 1)),
                    UNIQUE(client_id, client_sequence)
                );
                CREATE INDEX operations_project_order
                    ON operations(project_id, lamport, client_id, client_sequence);
                CREATE INDEX operations_pending
                    ON operations(project_id, remote_acknowledged, lamport, client_id, client_sequence);",
            )?;
            transaction.pragma_update(None, "user_version", 1_i64)?;
            transaction.commit()?;
            schema_version = 1;
        }
        if schema_version == 1 {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(
                "CREATE TABLE snapshots (
                    created_index INTEGER PRIMARY KEY AUTOINCREMENT,
                    snapshot_id TEXT NOT NULL UNIQUE,
                    project_id TEXT NOT NULL,
                    snapshot_format_version INTEGER NOT NULL,
                    graph_schema_version INTEGER NOT NULL,
                    runtime_version TEXT NOT NULL,
                    cursor_lamport INTEGER,
                    cursor_client_id TEXT,
                    cursor_client_sequence INTEGER,
                    graph_json TEXT NOT NULL,
                    record_sha256 TEXT NOT NULL,
                    CHECK (
                        (cursor_lamport IS NULL AND cursor_client_id IS NULL AND cursor_client_sequence IS NULL)
                        OR
                        (cursor_lamport IS NOT NULL AND cursor_lamport > 0 AND
                         cursor_client_id IS NOT NULL AND length(cursor_client_id) > 0 AND
                         cursor_client_sequence IS NOT NULL AND cursor_client_sequence > 0)
                    )
                );
                CREATE INDEX snapshots_project_cursor
                    ON snapshots(project_id, cursor_lamport, cursor_client_id, cursor_client_sequence, created_index);
                CREATE TABLE named_versions (
                    version_id TEXT PRIMARY KEY NOT NULL,
                    project_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    snapshot_id TEXT NOT NULL REFERENCES snapshots(snapshot_id),
                    UNIQUE(project_id, name)
                );
                CREATE TABLE materialized_projects (
                    project_id TEXT PRIMARY KEY NOT NULL,
                    graph_json TEXT NOT NULL,
                    state_sha256 TEXT NOT NULL,
                    cursor_lamport INTEGER,
                    cursor_client_id TEXT,
                    cursor_client_sequence INTEGER,
                    CHECK (
                        (cursor_lamport IS NULL AND cursor_client_id IS NULL AND cursor_client_sequence IS NULL)
                        OR
                        (cursor_lamport IS NOT NULL AND cursor_lamport > 0 AND
                         cursor_client_id IS NOT NULL AND length(cursor_client_id) > 0 AND
                         cursor_client_sequence IS NOT NULL AND cursor_client_sequence > 0)
                    )
                );",
            )?;
            transaction.pragma_update(None, "user_version", JOURNAL_SCHEMA_VERSION)?;
            transaction.commit()?;
        }

        Ok(Self { connection })
    }

    /// Appends a validated command and commits it before returning.
    ///
    /// Repeating an identical envelope is idempotent. Reusing an operation ID
    /// with different content or reusing a client sequence is reported as a
    /// conflict. The returned result describes local durability only.
    ///
    /// # Errors
    ///
    /// Returns [`JournalError`] for invalid envelopes, conflicting retries,
    /// serialization failures, or failed `SQLite` transactions.
    pub fn append(&mut self, envelope: &CommandEnvelope) -> Result<AppendOutcome, JournalError> {
        self.append_inner(envelope, None)
    }

    pub(crate) fn append_with_materialized(
        &mut self,
        envelope: &CommandEnvelope,
        base_graph: &ProjectGraph,
        graph: &ProjectGraph,
    ) -> Result<AppendOutcome, JournalError> {
        if base_graph.project_id != envelope.project_id {
            return Err(JournalError::ProjectMismatch {
                command: envelope.project_id.clone(),
                graph: base_graph.project_id.clone(),
            });
        }
        if graph.project_id != envelope.project_id {
            return Err(JournalError::ProjectMismatch {
                command: envelope.project_id.clone(),
                graph: graph.project_id.clone(),
            });
        }
        base_graph.validate()?;
        graph.validate()?;
        let base_json = serde_json::to_string(base_graph)?;
        let graph_json = serde_json::to_string(graph)?;
        self.append_inner(envelope, Some((&base_json, &graph_json)))
    }

    fn append_inner(
        &mut self,
        envelope: &CommandEnvelope,
        materialized: Option<(&str, &str)>,
    ) -> Result<AppendOutcome, JournalError> {
        envelope.validate()?;
        let client_sequence = to_sql_integer(envelope.causal.client_sequence, "client_sequence")?;
        let lamport = to_sql_integer(envelope.causal.lamport, "lamport")?;
        let payload_json = serde_json::to_string(envelope)?;
        let transaction =
            self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

        let existing: Option<String> = transaction
            .query_row(
                "SELECT payload_json FROM operations WHERE operation_id = ?1",
                [envelope.operation_id.as_str()],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(existing) = existing {
            if existing == payload_json {
                transaction.commit()?;
                return Ok(AppendOutcome::AlreadyCommitted);
            }
            return Err(JournalError::OperationIdCollision(envelope.operation_id.clone()));
        }

        let sequence_exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM operations WHERE client_id = ?1 AND client_sequence = ?2)",
            params![envelope.causal.client_id, client_sequence],
            |row| row.get(0),
        )?;
        if sequence_exists {
            return Err(JournalError::ClientSequenceCollision {
                client_id: envelope.causal.client_id.clone(),
                sequence: envelope.causal.client_sequence,
            });
        }

        if let Some((base_json, _)) = materialized {
            verify_materialized_base(&transaction, envelope, lamport, client_sequence, base_json)?;
        }

        transaction.execute(
            "INSERT INTO operations (
                operation_id, project_id, target_id, client_id, client_sequence,
                lamport, parent_operation_id, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                envelope.operation_id.as_str(),
                envelope.project_id.as_str(),
                envelope.target_id.as_str(),
                envelope.causal.client_id,
                client_sequence,
                lamport,
                envelope.causal.parent_operation_id.as_ref().map(ObjectId::as_str),
                payload_json,
            ],
        )?;
        if let Some((_, graph_json)) = materialized {
            let cursor = JournalCursor {
                lamport: envelope.causal.lamport,
                client_id: envelope.causal.client_id.clone(),
                client_sequence: envelope.causal.client_sequence,
            };
            let state_sha256 = materialized_digest(&envelope.project_id, graph_json, Some(&cursor));
            transaction.execute(
                "INSERT INTO materialized_projects (
                    project_id, graph_json, state_sha256,
                    cursor_lamport, cursor_client_id, cursor_client_sequence
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(project_id) DO UPDATE SET
                    graph_json = excluded.graph_json,
                    state_sha256 = excluded.state_sha256,
                    cursor_lamport = excluded.cursor_lamport,
                    cursor_client_id = excluded.cursor_client_id,
                    cursor_client_sequence = excluded.cursor_client_sequence",
                params![
                    envelope.project_id.as_str(),
                    graph_json,
                    state_sha256,
                    lamport,
                    envelope.causal.client_id,
                    client_sequence,
                ],
            )?;
        }
        transaction.commit()?;
        Ok(AppendOutcome::Committed)
    }

    /// Reads a locally committed operation by ID.
    ///
    /// # Errors
    ///
    /// Returns [`JournalError`] if `SQLite` or JSON decoding fails.
    pub fn read(&self, operation_id: &ObjectId) -> Result<Option<JournalEntry>, JournalError> {
        let row = self
            .connection
            .query_row(
                "SELECT payload_json, remote_acknowledged FROM operations WHERE operation_id = ?1",
                [operation_id.as_str()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?)),
            )
            .optional()?;
        row.map(|(payload, acknowledged)| decode_entry(&payload, acknowledged)).transpose()
    }

    /// Reads a deterministic page of operations after an optional logical cursor.
    ///
    /// # Errors
    ///
    /// Returns [`JournalError`] if the page limit is zero, exceeds 10,000,
    /// `SQLite` fails, or a stored payload cannot be decoded.
    pub fn read_range(
        &self,
        project_id: &ObjectId,
        after: Option<&JournalCursor>,
        limit: usize,
    ) -> Result<Vec<JournalEntry>, JournalError> {
        if !(1..=10_000).contains(&limit) {
            return Err(JournalError::InvalidPageLimit(limit));
        }
        let limit = i64::try_from(limit).map_err(|_| JournalError::InvalidPageLimit(limit))?;
        let mut entries = Vec::new();
        if let Some(cursor) = after {
            let lamport = to_sql_integer(cursor.lamport, "cursor.lamport")?;
            let sequence = to_sql_integer(cursor.client_sequence, "cursor.client_sequence")?;
            let mut statement = self.connection.prepare(
                "SELECT payload_json, remote_acknowledged FROM operations
                     WHERE project_id = ?1 AND (
                         lamport > ?2 OR
                         (lamport = ?2 AND client_id > ?3) OR
                         (lamport = ?2 AND client_id = ?3 AND client_sequence > ?4)
                     )
                     ORDER BY lamport, client_id, client_sequence LIMIT ?5",
            )?;
            let rows = statement.query_map(
                params![project_id.as_str(), lamport, cursor.client_id, sequence, limit],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?)),
            )?;
            for row in rows {
                let (payload, acknowledged) = row?;
                entries.push(decode_entry(&payload, acknowledged)?);
            }
        } else {
            let mut statement = self.connection.prepare(
                "SELECT payload_json, remote_acknowledged FROM operations
                     WHERE project_id = ?1
                     ORDER BY lamport, client_id, client_sequence LIMIT ?2",
            )?;
            let rows = statement.query_map(params![project_id.as_str(), limit], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?))
            })?;
            for row in rows {
                let (payload, acknowledged) = row?;
                entries.push(decode_entry(&payload, acknowledged)?);
            }
        }
        Ok(entries)
    }

    /// Marks a locally committed operation as acknowledged by the remote service.
    ///
    /// This is idempotent and does not alter the locally committed command.
    ///
    /// # Errors
    ///
    /// Returns [`JournalError::UnknownOperation`] when no operation has that ID.
    pub fn mark_remote_acknowledged(
        &mut self,
        operation_id: &ObjectId,
    ) -> Result<(), JournalError> {
        let transaction =
            self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = transaction.execute(
            "UPDATE operations SET remote_acknowledged = 1 WHERE operation_id = ?1",
            [operation_id.as_str()],
        )?;
        if changed == 0 {
            return Err(JournalError::UnknownOperation(operation_id.clone()));
        }
        transaction.commit()?;
        Ok(())
    }

    /// Checks `SQLite` configuration, schema version, and database integrity.
    ///
    /// # Errors
    ///
    /// Returns [`JournalError`] if the database cannot answer a health query.
    pub fn health(&self) -> Result<JournalHealth, JournalError> {
        Ok(JournalHealth {
            sqlite_version: self
                .connection
                .query_row("SELECT sqlite_version()", [], |row| row.get(0))?,
            schema_version: self
                .connection
                .pragma_query_value(None, "user_version", |row| row.get(0))?,
            journal_mode: self
                .connection
                .pragma_query_value(None, "journal_mode", |row| row.get(0))?,
            synchronous: self
                .connection
                .pragma_query_value(None, "synchronous", |row| row.get(0))?,
            integrity: self.connection.pragma_query_value(None, "quick_check", |row| row.get(0))?,
        })
    }
}

fn verify_materialized_base(
    connection: &Connection,
    envelope: &CommandEnvelope,
    lamport: i64,
    client_sequence: i64,
    base_json: &str,
) -> Result<(), JournalError> {
    let last = connection
        .query_row(
            "SELECT lamport, client_id, client_sequence FROM operations
             WHERE project_id = ?1
             ORDER BY lamport DESC, client_id DESC, client_sequence DESC LIMIT 1",
            [envelope.project_id.as_str()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?)),
        )
        .optional()?;
    let incoming = (lamport, envelope.causal.client_id.as_str(), client_sequence);
    if let Some((last_lamport, last_client_id, last_sequence)) = &last
        && incoming <= (*last_lamport, last_client_id.as_str(), *last_sequence)
    {
        return Err(JournalError::CausalOrderRegression(envelope.project_id.clone()));
    }

    let stored: Option<MaterializedRow> = connection
        .query_row(
            "SELECT graph_json, state_sha256, cursor_lamport, cursor_client_id, cursor_client_sequence
             FROM materialized_projects WHERE project_id = ?1",
            [envelope.project_id.as_str()],
            |row| {
                Ok(MaterializedRow {
                    graph_json: row.get(0)?,
                    state_sha256: row.get(1)?,
                    cursor_lamport: row.get(2)?,
                    cursor_client_id: row.get(3)?,
                    cursor_client_sequence: row.get(4)?,
                })
            },
        )
        .optional()?;

    match (last, stored) {
        (Some(_), None) => {
            Err(JournalError::MaterializedStateUnavailable(envelope.project_id.clone()))
        }
        (Some((last_lamport, last_client_id, last_sequence)), Some(stored)) => {
            let cursor = JournalCursor {
                lamport: u64::try_from(last_lamport)
                    .map_err(|_| JournalError::IntegerOutOfRange("cursor.lamport"))?,
                client_id: last_client_id.clone(),
                client_sequence: u64::try_from(last_sequence)
                    .map_err(|_| JournalError::IntegerOutOfRange("cursor.client_sequence"))?,
            };
            let expected_digest =
                materialized_digest(&envelope.project_id, &stored.graph_json, Some(&cursor));
            if stored.graph_json != base_json
                || stored.state_sha256 != expected_digest
                || stored.cursor_lamport != Some(last_lamport)
                || stored.cursor_client_id.as_deref() != Some(last_client_id.as_str())
                || stored.cursor_client_sequence != Some(last_sequence)
            {
                return Err(JournalError::MaterializedStateMismatch(envelope.project_id.clone()));
            }
            Ok(())
        }
        (None, Some(stored)) => {
            let expected_digest =
                materialized_digest(&envelope.project_id, &stored.graph_json, None);
            if stored.graph_json != base_json
                || stored.state_sha256 != expected_digest
                || stored.cursor_lamport.is_some()
                || stored.cursor_client_id.is_some()
                || stored.cursor_client_sequence.is_some()
            {
                return Err(JournalError::MaterializedStateMismatch(envelope.project_id.clone()));
            }
            Ok(())
        }
        (None, None) => Ok(()),
    }
}

fn to_sql_integer(value: u64, field: &'static str) -> Result<i64, JournalError> {
    i64::try_from(value).map_err(|_| JournalError::IntegerOutOfRange(field))
}

pub(crate) fn graph_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    format!("sha256:{encoded}")
}

pub(crate) fn materialized_digest(
    project_id: &ObjectId,
    graph_json: &str,
    cursor: Option<&JournalCursor>,
) -> String {
    let cursor_text = cursor.map_or_else(
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
    let state = format!(
        "project:{}:{};graph:{}:{};cursor:{}",
        project_id.as_str().len(),
        project_id.as_str(),
        graph_json.len(),
        graph_json,
        cursor_text
    );
    graph_digest(state.as_bytes())
}

fn decode_entry(payload: &str, acknowledged: bool) -> Result<JournalEntry, JournalError> {
    Ok(JournalEntry {
        envelope: serde_json::from_str(payload)?,
        replication: if acknowledged {
            ReplicationState::RemoteAcknowledged
        } else {
            ReplicationState::Pending
        },
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;
    use crate::command::{Actor, CausalContext, CreativeCommand, PropertyState};

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

    struct TestDb {
        directory: tempfile::TempDir,
        path: PathBuf,
    }

    impl TestDb {
        fn new() -> Self {
            let directory = tempfile::tempdir().expect("temporary directory");
            let path = directory.path().join(format!(
                "journal-{}-{}.sqlite",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            Self { directory, path }
        }
    }

    fn id(number: u16) -> ObjectId {
        ObjectId::new(format!("00000000-0000-4000-8000-{number:012}"))
    }

    fn envelope(operation: u16, sequence: u64, value: serde_json::Value) -> CommandEnvelope {
        CommandEnvelope {
            operation_id: id(operation),
            project_id: id(2),
            target_id: id(10),
            schema_version: 1,
            actor: Actor::user("user-1"),
            causal: CausalContext::new("desktop-1", sequence, sequence),
            command: CreativeCommand::SetProperty {
                node_id: id(10),
                property: "opacity".to_owned(),
                value,
                previous_state: PropertyState::Present(serde_json::Value::from(1.0)),
            },
        }
    }

    #[test]
    fn append_is_durable_idempotent_and_separate_from_remote_ack() {
        let database = TestDb::new();
        let first = envelope(1, 1, serde_json::Value::from(0.5));
        {
            let mut journal = LocalJournal::open(&database.path).expect("open journal");
            assert_eq!(journal.append(&first).expect("append"), AppendOutcome::Committed);
            assert_eq!(journal.append(&first).expect("retry"), AppendOutcome::AlreadyCommitted);
            assert_eq!(
                journal.read(&first.operation_id).expect("read").unwrap().replication,
                ReplicationState::Pending
            );
            journal
                .mark_remote_acknowledged(&first.operation_id)
                .expect("mark remote acknowledgment");
            journal
                .mark_remote_acknowledged(&first.operation_id)
                .expect("repeated acknowledgment is safe");
        }
        let journal = LocalJournal::open(&database.path).expect("reopen journal");
        let recovered = journal.read(&first.operation_id).expect("read after restart").unwrap();
        assert_eq!(recovered.envelope, first);
        assert_eq!(recovered.replication, ReplicationState::RemoteAcknowledged);
        assert!(journal.health().expect("health").meets_configured_durability());
    }

    #[test]
    fn rejects_operation_id_payload_and_client_sequence_collisions() {
        let database = TestDb::new();
        let mut journal = LocalJournal::open(&database.path).expect("open journal");
        let first = envelope(1, 1, serde_json::Value::from(0.5));
        journal.append(&first).expect("first append");

        let changed_payload = envelope(1, 1, serde_json::Value::from(0.25));
        assert!(matches!(
            journal.append(&changed_payload),
            Err(JournalError::OperationIdCollision(_))
        ));

        let same_sequence = envelope(3, 1, serde_json::Value::from(0.75));
        assert!(matches!(
            journal.append(&same_sequence),
            Err(JournalError::ClientSequenceCollision { .. })
        ));
    }

    #[test]
    fn pages_by_logical_order_and_project() {
        let database = TestDb::new();
        let mut journal = LocalJournal::open(&database.path).expect("open journal");
        let first = envelope(1, 1, serde_json::Value::from(0.5));
        let second = envelope(3, 3, serde_json::Value::from(0.3));
        journal.append(&second).expect("append second");
        journal.append(&first).expect("append first");
        let page = journal.read_range(&id(2), None, 1).expect("first page");
        assert_eq!(page[0].envelope.operation_id, first.operation_id);
        let cursor = JournalCursor {
            lamport: first.causal.lamport,
            client_id: first.causal.client_id.clone(),
            client_sequence: first.causal.client_sequence,
        };
        let rest = journal.read_range(&id(2), Some(&cursor), 10).expect("next page");
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].envelope.operation_id, second.operation_id);
        assert!(journal.read_range(&id(99), None, 10).expect("other project").is_empty());
    }

    #[test]
    fn failed_transaction_does_not_acknowledge_or_store_operation() {
        let database = TestDb::new();
        let mut journal = LocalJournal::open(&database.path).expect("open journal");
        journal
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_operation BEFORE INSERT ON operations
                 BEGIN SELECT RAISE(ABORT, 'injected commit failure'); END;",
            )
            .expect("install failure injector");
        let command = envelope(1, 1, serde_json::Value::from(0.5));
        assert!(journal.append(&command).is_err());
        assert!(journal.read(&command.operation_id).expect("query journal").is_none());
    }

    #[test]
    fn initializes_empty_database_and_rejects_corrupt_file() {
        let database = TestDb::new();
        let journal = LocalJournal::open(&database.path).expect("initialize schema version zero");
        assert_eq!(journal.health().expect("health").schema_version, JOURNAL_SCHEMA_VERSION);
        drop(journal);

        let corrupt = database.directory.path().join("corrupt.sqlite");
        fs::write(&corrupt, b"not a sqlite database").expect("write corruption fixture");
        assert!(LocalJournal::open(corrupt).is_err());
    }

    #[test]
    fn migrates_existing_v1_journal_without_losing_operations() {
        let database = TestDb::new();
        let operation = envelope(7, 7, serde_json::Value::from(0.75));
        let payload = serde_json::to_string(&operation).expect("serialize legacy operation");
        let connection = Connection::open(&database.path).expect("create legacy database");
        connection
            .execute_batch(
                "CREATE TABLE operations (
                    operation_id TEXT PRIMARY KEY NOT NULL,
                    project_id TEXT NOT NULL,
                    target_id TEXT NOT NULL,
                    client_id TEXT NOT NULL,
                    client_sequence INTEGER NOT NULL CHECK(client_sequence > 0),
                    lamport INTEGER NOT NULL CHECK(lamport > 0),
                    parent_operation_id TEXT,
                    payload_json TEXT NOT NULL,
                    remote_acknowledged INTEGER NOT NULL DEFAULT 0 CHECK(remote_acknowledged IN (0, 1)),
                    UNIQUE(client_id, client_sequence)
                );
                CREATE INDEX operations_project_order
                    ON operations(project_id, lamport, client_id, client_sequence);
                CREATE INDEX operations_pending
                    ON operations(project_id, remote_acknowledged, lamport, client_id, client_sequence);
                PRAGMA user_version = 1;",
            )
            .expect("create schema version one");
        connection
            .execute(
                "INSERT INTO operations (
                    operation_id, project_id, target_id, client_id, client_sequence,
                    lamport, parent_operation_id, payload_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7)",
                params![
                    operation.operation_id.as_str(),
                    operation.project_id.as_str(),
                    operation.target_id.as_str(),
                    operation.causal.client_id,
                    i64::try_from(operation.causal.client_sequence).unwrap(),
                    i64::try_from(operation.causal.lamport).unwrap(),
                    payload,
                ],
            )
            .expect("seed prior operation");
        drop(connection);

        let journal = LocalJournal::open(&database.path).expect("migrate history tables");
        assert_eq!(journal.health().expect("health").schema_version, JOURNAL_SCHEMA_VERSION);
        assert_eq!(
            journal.read(&operation.operation_id).expect("read migrated op").unwrap().envelope,
            operation
        );
    }

    #[test]
    fn disk_full_rejection_is_reported_without_partial_commit() {
        let database = TestDb::new();
        let mut journal = LocalJournal::open(&database.path).expect("open journal");
        let page_count: i64 = journal
            .connection
            .pragma_query_value(None, "page_count", |row| row.get(0))
            .expect("page count");
        journal
            .connection
            .pragma_update(None, "max_page_count", page_count)
            .expect("cap database growth at current size");
        let large_value = serde_json::Value::String("creative-data".repeat(20_000));
        let command = envelope(1, 1, large_value);
        assert!(journal.append(&command).is_err());
        assert!(journal.read(&command.operation_id).expect("query journal").is_none());
    }
}
