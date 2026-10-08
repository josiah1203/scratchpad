//! Portable, content-addressed project bundle import and export.
//!
//! Export writes a complete staging tree first, then publishes files without
//! replacing existing paths. `manifest.json` is published last, so a directory
//! left behind by an interrupted publication is visibly incomplete. Import
//! verifies the complete package before atomically inserting project rows.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::Builder;

use crate::{
    asset_store::{AssetError, LocalAssetStore},
    graph::{AssetId, ObjectId, ProjectGraph},
    history::{HistoryError, NamedVersion, Snapshot},
    journal::{JournalCursor, JournalError, LocalJournal},
};

const OPERATIONS_PAGE_SIZE: usize = 256;
const OPERATION_ORDER: &str = "lamport_client_id_client_sequence";

#[path = "portable_import.rs"]
mod import;

pub use import::{
    BundleImportReport, BundleVerification, import_project_bundle,
    import_project_bundle_with_limits, verify_project_bundle, verify_project_bundle_with_limits,
};

/// Manifest for a directory-form portable project bundle, version 1.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    pub format: String,
    pub format_version: u32,
    pub project_id: ObjectId,
    pub graph: GraphFile,
    pub operations: OperationsFile,
    pub snapshots: Vec<SnapshotFile>,
    pub named_versions: Vec<NamedVersionFile>,
    pub assets: Vec<AssetFile>,
    pub access_control: AccessControl,
    pub operation_actor_identity: String,
}

/// Graph file metadata in a bundle manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphFile {
    pub schema_version: u32,
    pub sha256: String,
}

/// Operation journal file metadata in a bundle manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationsFile {
    pub count: u64,
    pub sha256: String,
    pub command_schema_versions: Vec<u32>,
    pub order: String,
}

/// A snapshot record included in the bundle.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotFile {
    pub snapshot_id: ObjectId,
    pub sha256: String,
}

/// A named version linked to an included snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NamedVersionFile {
    pub version_id: ObjectId,
    pub name: String,
    pub snapshot_id: ObjectId,
}

/// An immutable asset blob included in the bundle.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetFile {
    pub id: AssetId,
    pub byte_length: u64,
}

/// Access-control records are deliberately not portable bundle content.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccessControl {
    pub memberships_included: bool,
    pub capability_grants_included: bool,
}

/// Resource bounds applied when reading an untrusted directory bundle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleImportLimits {
    /// Maximum number of files and directories beneath the bundle root.
    pub max_entries: usize,
    /// Maximum combined size of every regular file in the bundle.
    pub max_total_bytes: u64,
    /// Maximum combined size of non-asset files held or parsed during verification.
    pub max_non_asset_bytes: u64,
    /// Maximum size of any one file, including an asset blob.
    pub max_single_file_bytes: u64,
    /// Maximum size of `manifest.json`.
    pub max_manifest_bytes: u64,
    /// Maximum number of operations in the journal.
    pub max_operation_count: u64,
    /// Maximum bytes in one JSONL operation line, including its newline.
    pub max_operation_line_bytes: usize,
    /// Maximum number of snapshots.
    pub max_snapshots: usize,
    /// Maximum number of named versions.
    pub max_named_versions: usize,
    /// Maximum number of assets.
    pub max_assets: usize,
}

impl Default for BundleImportLimits {
    fn default() -> Self {
        Self {
            max_entries: 120_000,
            max_total_bytes: 8 * 1024 * 1024 * 1024,
            max_non_asset_bytes: 256 * 1024 * 1024,
            max_single_file_bytes: 2 * 1024 * 1024 * 1024,
            max_manifest_bytes: 4 * 1024 * 1024,
            max_operation_count: 100_000,
            max_operation_line_bytes: 8 * 1024 * 1024,
            max_snapshots: 10_000,
            max_named_versions: 100_000,
            max_assets: 100_000,
        }
    }
}

/// Failures while validating, importing, or exporting a portable project bundle.
#[derive(Debug)]
pub enum PortableError {
    History(HistoryError),
    Journal(JournalError),
    Asset(AssetError),
    Io(io::Error),
    Serialization(serde_json::Error),
    NoSnapshots(ObjectId),
    DestinationExists(PathBuf),
    InvalidBundle(String),
    ImportCollision(String),
    AssetLengthMismatch { id: AssetId, expected: u64, actual: u64 },
    PublishFailed { destination: PathBuf, source: io::Error },
}

impl fmt::Display for PortableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::History(error) => error.fmt(formatter),
            Self::Journal(error) => error.fmt(formatter),
            Self::Asset(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
            Self::Serialization(error) => write!(formatter, "bundle serialization failed: {error}"),
            Self::NoSnapshots(project_id) => write!(
                formatter,
                "project {} has no recovery snapshots and cannot be exported as bundle v1",
                project_id.as_str()
            ),
            Self::DestinationExists(path) => {
                write!(formatter, "bundle destination already exists: {}", path.display())
            }
            Self::InvalidBundle(reason) => {
                write!(formatter, "project bundle is inconsistent: {reason}")
            }
            Self::ImportCollision(reason) => {
                write!(formatter, "project bundle cannot be imported: {reason}")
            }
            Self::AssetLengthMismatch { id, expected, actual } => write!(
                formatter,
                "asset {} has {actual} bytes; project history records {expected}",
                id.as_str()
            ),
            Self::PublishFailed { destination, source } => write!(
                formatter,
                "bundle publication to {} failed before its manifest was installed: {source}",
                destination.display()
            ),
        }
    }
}

impl Error for PortableError {}

impl From<HistoryError> for PortableError {
    fn from(error: HistoryError) -> Self {
        Self::History(error)
    }
}

impl From<JournalError> for PortableError {
    fn from(error: JournalError) -> Self {
        Self::Journal(error)
    }
}

impl From<AssetError> for PortableError {
    fn from(error: AssetError) -> Self {
        Self::Asset(error)
    }
}

impl From<io::Error> for PortableError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for PortableError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

impl From<rusqlite::Error> for PortableError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Journal(JournalError::Sqlite(error))
    }
}

/// Exports a validated project and its history as a directory bundle.
///
/// The destination must not exist. The current materialized view is recovered
/// first, which may repair it from journal history. Every stored snapshot is
/// validated, operation actor IDs are retained as opaque provenance, and all
/// assets referenced by graphs, snapshots, or asset-ID-shaped command values
/// are copied and re-hashed. Because actor IDs may be personal data, callers
/// should disclose that before offering this export to end users.
///
/// Project writes must remain quiescent during the call: the current `SQLite`
/// APIs do not provide one read transaction spanning recovery, snapshots, and
/// paged operation export. A failed publication may leave a destination without
/// `manifest.json`; that directory is incomplete and must not be treated as a
/// bundle.
///
/// # Errors
///
/// Returns [`PortableError`] for invalid project history, missing/corrupt
/// assets, serialization or filesystem failures, absent recovery snapshots,
/// inconsistent cross-record references, or an existing destination.
pub fn export_project_bundle(
    journal: &mut LocalJournal,
    project_id: &ObjectId,
    assets: &LocalAssetStore,
    destination: impl AsRef<Path>,
) -> Result<BundleManifest, PortableError> {
    let destination = destination.as_ref();
    let parent = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let staging = Builder::new().prefix(".creative-project-bundle-").tempdir_in(parent)?;

    let records = collect_export_records(journal, project_id, assets)?;
    let manifest = write_bundle_files(journal, project_id, assets, &records, staging.path())?;
    let manifest_bytes = json_document(&manifest)?;
    write_new_file(&staging.path().join("manifest.json"), &manifest_bytes)?;
    match publish_staged(staging.path(), destination) {
        Ok(()) => {}
        Err(PublishError::DestinationExists) => {
            return Err(PortableError::DestinationExists(destination.to_path_buf()));
        }
        Err(PublishError::Io(source)) => {
            return Err(PortableError::PublishFailed {
                destination: destination.to_path_buf(),
                source,
            });
        }
    }
    Ok(manifest)
}

struct ExportRecords {
    graph: ProjectGraph,
    snapshots: Vec<Snapshot>,
    named_versions: Vec<NamedVersion>,
}

fn collect_export_records(
    journal: &mut LocalJournal,
    project_id: &ObjectId,
    assets: &LocalAssetStore,
) -> Result<ExportRecords, PortableError> {
    project_id
        .validate("portable.project_id")
        .map_err(|error| PortableError::InvalidBundle(error.to_string()))?;
    let snapshots = journal.list_snapshots(project_id, assets)?;
    if snapshots.is_empty() {
        return Err(PortableError::NoSnapshots(project_id.clone()));
    }
    let snapshot_ids: BTreeSet<_> =
        snapshots.iter().map(|snapshot| snapshot.snapshot_id.clone()).collect();
    if snapshot_ids.len() != snapshots.len() {
        return Err(PortableError::InvalidBundle("duplicate snapshot ID".to_owned()));
    }

    let recovery = journal.recover_project(project_id, assets)?;
    let graph = recovery.graph;
    if graph.project_id != *project_id {
        return Err(PortableError::InvalidBundle(
            "recovered graph belongs to another project".to_owned(),
        ));
    }

    let named_versions = journal.list_named_versions(project_id)?;
    validate_named_versions(project_id, &snapshot_ids, &named_versions)?;
    Ok(ExportRecords { graph, snapshots, named_versions })
}

fn write_bundle_files(
    journal: &LocalJournal,
    project_id: &ObjectId,
    assets: &LocalAssetStore,
    records: &ExportRecords,
    staging: &Path,
) -> Result<BundleManifest, PortableError> {
    let mut required_assets = BTreeMap::new();
    add_graph_assets(&records.graph, &mut required_assets)?;
    for snapshot in &records.snapshots {
        let snapshot_graph = ProjectGraph::from_json(&snapshot.graph_json)
            .map_err(|error| PortableError::InvalidBundle(error.to_string()))?;
        if snapshot_graph.project_id != *project_id {
            return Err(PortableError::InvalidBundle(format!(
                "snapshot {} belongs to another project",
                snapshot.snapshot_id.as_str()
            )));
        }
        add_graph_assets(&snapshot_graph, &mut required_assets)?;
    }

    let graph_bytes = json_document(&records.graph)?;
    let graph_digest = write_new_file(&staging.join("project.json"), &graph_bytes)?;

    let operations_result = write_operations(
        journal,
        project_id,
        &staging.join("operations.jsonl"),
        &mut required_assets,
    )?;

    let mut snapshot_files = Vec::with_capacity(records.snapshots.len());
    let snapshot_directory = staging.join("history").join("snapshots");
    fs::create_dir_all(&snapshot_directory)?;
    for snapshot in &records.snapshots {
        let bytes = json_document(snapshot)?;
        let path = snapshot_directory.join(format!("{}.json", snapshot.snapshot_id.as_str()));
        let digest = write_new_file(&path, &bytes)?;
        snapshot_files
            .push(SnapshotFile { snapshot_id: snapshot.snapshot_id.clone(), sha256: digest });
    }
    snapshot_files.sort_by(|left, right| left.snapshot_id.cmp(&right.snapshot_id));

    let asset_files = write_assets(assets, required_assets, staging)?;
    let mut named_versions = records.named_versions.clone();
    named_versions.sort_by(|left, right| left.version_id.cmp(&right.version_id));
    let named_version_files = named_versions.iter().map(named_version_file).collect();

    let manifest = BundleManifest {
        format: "unified-creative-project".to_owned(),
        format_version: 1,
        project_id: project_id.clone(),
        graph: GraphFile { schema_version: records.graph.schema_version, sha256: graph_digest },
        operations: OperationsFile {
            count: operations_result.count,
            sha256: operations_result.sha256,
            command_schema_versions: operations_result.schema_versions,
            order: OPERATION_ORDER.to_owned(),
        },
        snapshots: snapshot_files,
        named_versions: named_version_files,
        assets: asset_files,
        access_control: AccessControl {
            memberships_included: false,
            capability_grants_included: false,
        },
        operation_actor_identity: "preserved_as_opaque_provenance".to_owned(),
    };
    Ok(manifest)
}

fn write_assets(
    assets: &LocalAssetStore,
    required_assets: BTreeMap<AssetId, Option<u64>>,
    staging: &Path,
) -> Result<Vec<AssetFile>, PortableError> {
    let mut asset_files = Vec::with_capacity(required_assets.len());
    for (id, expected_length) in required_assets {
        let output_path = staging.join(asset_relative_path(&id)?);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = OpenOptions::new().write(true).create_new(true).open(&output_path)?;
        let copied = assets.copy_verified_to(&id, &mut output)?;
        output.sync_all()?;
        let (actual_digest, actual_length) = hash_file(&output_path)?;
        let id_digest = id
            .as_str()
            .strip_prefix("sha256:")
            .ok_or_else(|| PortableError::InvalidBundle("invalid asset ID".to_owned()))?;
        if actual_digest != id_digest || copied != actual_length {
            return Err(PortableError::InvalidBundle(format!(
                "asset {} changed while it was being copied",
                id.as_str()
            )));
        }
        if let Some(expected) = expected_length
            && actual_length != expected
        {
            return Err(PortableError::AssetLengthMismatch { id, expected, actual: actual_length });
        }
        asset_files.push(AssetFile { id, byte_length: actual_length });
    }
    Ok(asset_files)
}

struct OperationsResult {
    count: u64,
    sha256: String,
    schema_versions: Vec<u32>,
}

fn write_operations(
    journal: &LocalJournal,
    project_id: &ObjectId,
    path: &Path,
    assets: &mut BTreeMap<AssetId, Option<u64>>,
) -> Result<OperationsResult, PortableError> {
    let file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let mut writer = HashingWriter::new(file);
    let mut count = 0_u64;
    let mut schema_versions = BTreeSet::new();
    let mut cursor: Option<JournalCursor> = None;
    let mut previous_order: Option<(u64, String, u64)> = None;

    loop {
        let entries = journal.read_range(project_id, cursor.as_ref(), OPERATIONS_PAGE_SIZE)?;
        if entries.is_empty() {
            break;
        }
        for entry in entries {
            let envelope = entry.envelope;
            envelope.validate().map_err(|error| {
                PortableError::InvalidBundle(format!(
                    "operation {} is invalid: {error}",
                    envelope.operation_id.as_str()
                ))
            })?;
            if envelope.project_id != *project_id {
                return Err(PortableError::InvalidBundle(format!(
                    "operation {} belongs to another project",
                    envelope.operation_id.as_str()
                )));
            }
            let current_order = (
                envelope.causal.lamport,
                envelope.causal.client_id.clone(),
                envelope.causal.client_sequence,
            );
            if previous_order.as_ref().is_some_and(|previous| previous >= &current_order) {
                return Err(PortableError::InvalidBundle(
                    "operation journal is not strictly ordered".to_owned(),
                ));
            }
            previous_order = Some(current_order);
            let command_json = serde_json::to_value(&envelope.command)?;
            collect_asset_ids(&command_json, assets);

            let mut operation_bytes = serde_json::to_vec(&envelope)?;
            operation_bytes.push(b'\n');
            writer.write_all(&operation_bytes)?;
            count = count.checked_add(1).ok_or_else(|| {
                PortableError::InvalidBundle("operation count overflow".to_owned())
            })?;
            schema_versions.insert(envelope.schema_version);
            cursor = Some(JournalCursor {
                lamport: envelope.causal.lamport,
                client_id: envelope.causal.client_id,
                client_sequence: envelope.causal.client_sequence,
            });
        }
    }

    let (file, digest) = writer.finish()?;
    file.sync_all()?;
    Ok(OperationsResult {
        count,
        sha256: digest,
        schema_versions: schema_versions.into_iter().collect(),
    })
}

fn add_graph_assets(
    graph: &ProjectGraph,
    assets: &mut BTreeMap<AssetId, Option<u64>>,
) -> Result<(), PortableError> {
    for asset in &graph.asset_refs {
        add_asset(assets, &asset.id, Some(asset.byte_length))?;
    }
    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn add_asset(
    assets: &mut BTreeMap<AssetId, Option<u64>>,
    id: &AssetId,
    byte_length: Option<u64>,
) -> Result<(), PortableError> {
    if let Some(existing) = assets.get_mut(id) {
        if let (Some(expected), Some(actual)) = (*existing, byte_length)
            && expected != actual
        {
            return Err(PortableError::AssetLengthMismatch { id: id.clone(), expected, actual });
        }
        if existing.is_none() {
            *existing = byte_length;
        }
    } else {
        assets.insert(id.clone(), byte_length);
    }
    Ok(())
}

fn collect_asset_ids(value: &serde_json::Value, assets: &mut BTreeMap<AssetId, Option<u64>>) {
    match value {
        serde_json::Value::String(candidate) if is_asset_id(candidate) => {
            assets.entry(AssetId::new(candidate.clone())).or_insert(None);
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_asset_ids(value, assets);
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values() {
                collect_asset_ids(value, assets);
            }
        }
        _ => {}
    }
}

fn is_asset_id(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn validate_named_versions(
    project_id: &ObjectId,
    snapshot_ids: &BTreeSet<ObjectId>,
    named_versions: &[NamedVersion],
) -> Result<(), PortableError> {
    let mut version_ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for version in named_versions {
        if version.project_id != *project_id {
            return Err(PortableError::InvalidBundle(format!(
                "named version {} belongs to another project",
                version.version_id.as_str()
            )));
        }
        if version.name.trim().is_empty() {
            return Err(PortableError::InvalidBundle("named version has an empty name".to_owned()));
        }
        if !version_ids.insert(version.version_id.clone()) {
            return Err(PortableError::InvalidBundle("duplicate named-version ID".to_owned()));
        }
        if !names.insert(version.name.clone()) {
            return Err(PortableError::InvalidBundle("duplicate named-version name".to_owned()));
        }
        if !snapshot_ids.contains(&version.snapshot_id) {
            return Err(PortableError::InvalidBundle(format!(
                "named version {} refers to a missing snapshot",
                version.version_id.as_str()
            )));
        }
    }
    Ok(())
}

fn named_version_file(version: &NamedVersion) -> NamedVersionFile {
    NamedVersionFile {
        version_id: version.version_id.clone(),
        name: version.name.clone(),
        snapshot_id: version.snapshot_id.clone(),
    }
}

fn json_document<T: Serialize>(value: &T) -> Result<Vec<u8>, PortableError> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<String, PortableError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(format_digest(&Sha256::digest(bytes)))
}

fn asset_relative_path(id: &AssetId) -> Result<PathBuf, PortableError> {
    let digest = id
        .as_str()
        .strip_prefix("sha256:")
        .ok_or_else(|| PortableError::InvalidBundle(format!("invalid asset ID {}", id.as_str())))?;
    if digest.len() != 64
        || !digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(PortableError::InvalidBundle(format!("invalid asset ID {}", id.as_str())));
    }
    Ok(PathBuf::from("assets")
        .join("sha256")
        .join(&digest[..2])
        .join(format!("{}.blob", &digest[2..])))
}

fn hash_file(path: &Path) -> Result<(String, u64), PortableError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        length = length
            .checked_add(u64::try_from(read).map_err(|_| {
                PortableError::InvalidBundle("asset byte count overflow".to_owned())
            })?)
            .ok_or_else(|| PortableError::InvalidBundle("asset byte count overflow".to_owned()))?;
        hasher.update(&buffer[..read]);
    }
    Ok((hex_digest(hasher.finalize().as_slice()), length))
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn format_digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex_digest(bytes))
}

struct HashingWriter<W> {
    inner: W,
    hasher: Sha256,
}

impl<W: Write> HashingWriter<W> {
    fn new(inner: W) -> Self {
        Self { inner, hasher: Sha256::new() }
    }

    fn finish(mut self) -> io::Result<(W, String)> {
        self.inner.flush()?;
        Ok((self.inner, format!("sha256:{}", hex_digest(&self.hasher.finalize()))))
    }
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buffer)?;
        self.hasher.update(&buffer[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

enum PublishError {
    DestinationExists,
    Io(io::Error),
}

fn publish_staged(staging: &Path, destination: &Path) -> Result<(), PublishError> {
    match fs::create_dir(destination) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(PublishError::DestinationExists);
        }
        Err(error) => return Err(PublishError::Io(error)),
    }
    let mut directories = Vec::new();
    let mut files = Vec::new();
    collect_staged_entries(staging, staging, &mut directories, &mut files)
        .map_err(PublishError::Io)?;
    directories.sort();
    files.sort();

    for relative in directories {
        fs::create_dir(destination.join(relative)).map_err(PublishError::Io)?;
    }
    for relative in files.iter().filter(|path| path.as_path() != Path::new("manifest.json")) {
        copy_new_file(&staging.join(relative), &destination.join(relative))
            .map_err(PublishError::Io)?;
    }
    let manifest = Path::new("manifest.json");
    if !files.iter().any(|path| path == manifest) {
        return Err(PublishError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "staged manifest is missing",
        )));
    }
    copy_new_file(&staging.join(manifest), &destination.join(manifest)).map_err(PublishError::Io)
}

fn collect_staged_entries(
    root: &Path,
    directory: &Path,
    directories: &mut Vec<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| io::Error::other("staged path escaped its root"))?
            .to_path_buf();
        if file_type.is_dir() {
            directories.push(relative);
            collect_staged_entries(root, &path, directories, files)?;
        } else if file_type.is_file() {
            files.push(relative);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "staged bundle contains a non-regular file",
            ));
        }
    }
    Ok(())
}

fn copy_new_file(source: &Path, destination: &Path) -> io::Result<()> {
    let mut source = File::open(source)?;
    let mut destination = OpenOptions::new().write(true).create_new(true).open(destination)?;
    io::copy(&mut source, &mut destination)?;
    destination.sync_all()
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Cursor, Write},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;
    use crate::{
        SAMPLE_PROJECT,
        asset_store::AssetMetadata,
        command::{Actor, CausalContext, CommandEnvelope, CreativeCommand},
        journal::AppendOutcome,
    };

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    const MANIFEST_SCHEMA: &str = include_str!("../../../schemas/project-manifest-v1.schema.json");

    fn id(number: u16) -> ObjectId {
        ObjectId::new(format!("00000000-0000-4000-8000-{number:012}"))
    }

    fn export_fixture() -> (tempfile::TempDir, BundleManifest, PathBuf, AssetId) {
        let directory = tempfile::tempdir().expect("temporary directory");
        let database = directory.path().join(format!(
            "portable-{}-{}.sqlite",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let assets = LocalAssetStore::open(directory.path(), "portable-test").expect("asset store");
        let asset_bytes = b"portable asset payload";
        let mut source = Cursor::new(asset_bytes.to_vec());
        let source_asset = assets
            .ingest(&mut source, AssetMetadata::new("image", "portable-test"))
            .expect("ingest asset");
        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture graph");
        graph.asset_refs = vec![source_asset.clone()];
        let image_node = graph.nodes.iter_mut().find(|node| node.id == id(12)).expect("image node");
        image_node.asset_ref = Some(source_asset.id.clone());
        graph.validate().expect("valid graph");

        let mut journal = LocalJournal::open(&database).expect("journal");
        journal
            .create_named_version(&id(101), &id(100), "Starting point", &graph, &assets)
            .expect("initial snapshot and named version");

        let operation_asset_bytes = b"historical operation-only asset";
        let mut operation_asset_source = Cursor::new(operation_asset_bytes.to_vec());
        let operation_asset = assets
            .ingest(&mut operation_asset_source, AssetMetadata::new("image", "operation-payload"))
            .expect("ingest operation asset");
        let operation = CommandEnvelope {
            operation_id: id(200),
            project_id: id(1),
            target_id: id(11),
            schema_version: 1,
            actor: Actor::user("private-actor-id"),
            causal: CausalContext::new("portable-client", 1, 1),
            command: CreativeCommand::SetProperty {
                node_id: id(11),
                property: "historical_asset".to_owned(),
                value: serde_json::Value::String(operation_asset.id.as_str().to_owned()),
                previous_state: crate::command::PropertyState::Absent,
            },
        };
        let mut updated = graph.clone();
        updated
            .nodes
            .iter_mut()
            .find(|node| node.id == id(11))
            .expect("frame node")
            .properties
            .insert(
                "historical_asset".to_owned(),
                serde_json::Value::String(operation_asset.id.as_str().to_owned()),
            );
        assert_eq!(
            journal
                .append_with_materialized(&operation, &graph, &updated)
                .expect("append operation"),
            AppendOutcome::Committed
        );
        journal
            .create_named_version(&id(103), &id(102), "After edit", &updated, &assets)
            .expect("later snapshot and named version");
        let destination = directory.path().join("bundle");
        let manifest = export_project_bundle(&mut journal, &id(1), &assets, &destination)
            .expect("export project bundle");
        (directory, manifest, destination, operation_asset.id)
    }

    #[test]
    fn exports_graph_history_snapshots_named_versions_and_all_referenced_assets() {
        let (_directory, manifest, destination, operation_asset_id) = export_fixture();
        let verification = verify_project_bundle(&destination).expect("verify complete bundle");
        assert_eq!(verification.manifest, manifest);
        assert!(verification.total_bytes >= verification.asset_bytes);
        assert_eq!(manifest.project_id, id(1));
        assert_eq!(manifest.operations.count, 1);
        assert_eq!(manifest.operations.command_schema_versions, vec![1]);
        assert_eq!(manifest.snapshots.len(), 2);
        assert_eq!(manifest.named_versions.len(), 2);
        assert!(!manifest.access_control.memberships_included);
        assert!(!manifest.access_control.capability_grants_included);
        assert_eq!(manifest.operation_actor_identity, "preserved_as_opaque_provenance");
        assert_eq!(manifest.assets.len(), 2);
        assert!(manifest.assets.iter().any(|asset| asset.id == operation_asset_id));

        let (graph_digest, _) = hash_file(&destination.join("project.json")).expect("graph digest");
        assert_eq!(manifest.graph.sha256, format!("sha256:{graph_digest}"));
        let operation_lines =
            fs::read_to_string(destination.join("operations.jsonl")).expect("operations file");
        let (operations_digest, _) =
            hash_file(&destination.join("operations.jsonl")).expect("operations digest");
        assert_eq!(manifest.operations.sha256, format!("sha256:{operations_digest}"));
        let operation: CommandEnvelope =
            serde_json::from_str(operation_lines.trim()).expect("operation envelope");
        assert_eq!(operation.actor.id, "private-actor-id");
        for snapshot in &manifest.snapshots {
            let snapshot_path = destination
                .join("history/snapshots")
                .join(format!("{}.json", snapshot.snapshot_id.as_str()));
            let (snapshot_digest, _) = hash_file(&snapshot_path).expect("snapshot digest");
            assert_eq!(snapshot.sha256, format!("sha256:{snapshot_digest}"));
        }
        let manifest_json: BundleManifest = serde_json::from_slice(
            &fs::read(destination.join("manifest.json")).expect("manifest file"),
        )
        .expect("manifest decodes");
        assert_eq!(manifest_json, manifest);
        for asset in &manifest.assets {
            let path = destination.join(asset_relative_path(&asset.id).expect("asset path"));
            let (digest, length) = hash_file(&path).expect("asset digest");
            assert_eq!(digest, asset.id.as_str().strip_prefix("sha256:").expect("asset ID"));
            assert_eq!(length, asset.byte_length);
        }
    }

    #[test]
    fn exported_manifest_matches_published_schema_and_rejects_policy_drift() {
        let (_directory, _manifest, destination, _) = export_fixture();
        let schema: serde_json::Value =
            serde_json::from_str(MANIFEST_SCHEMA).expect("manifest schema JSON");
        let validator = jsonschema::validator_for(&schema)
            .expect("manifest schema should compile as Draft 2020-12");
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(destination.join("manifest.json")).expect("read exported manifest"),
        )
        .expect("exported manifest JSON");

        assert!(validator.is_valid(&manifest), "exported manifest must satisfy its schema");

        let mut grant_leaking_manifest = manifest.clone();
        grant_leaking_manifest["access_control"]["memberships_included"] =
            serde_json::Value::Bool(true);
        assert!(!validator.is_valid(&grant_leaking_manifest));

        let mut manifest_with_unknown_field = manifest;
        manifest_with_unknown_field["unexpected"] = serde_json::Value::Bool(true);
        assert!(!validator.is_valid(&manifest_with_unknown_field));
    }

    #[test]
    fn imports_verified_bundle_with_history_assets_and_collision_safety() {
        let (_source_directory, _manifest, bundle_path, operation_asset_id) = export_fixture();
        let destination = tempfile::tempdir().expect("destination workspace");
        let database = destination.path().join("import.sqlite");
        let assets = LocalAssetStore::open(destination.path(), "imported-project").expect("assets");
        let mut journal = LocalJournal::open(&database).expect("journal");

        let report = import_project_bundle(&bundle_path, &mut journal, &assets)
            .expect("import validated bundle");
        assert_eq!(report.project_id, id(1));
        assert_eq!(report.operations_imported, 1);
        assert_eq!(report.snapshots_imported, 2);
        assert_eq!(report.named_versions_imported, 2);
        assert_eq!(report.assets_imported, 2);
        assert!(assets.verify(&operation_asset_id).is_ok());

        let recovery = journal.recover_project(&id(1), &assets).expect("recover imported project");
        let project_json =
            fs::read_to_string(bundle_path.join("project.json")).expect("graph file");
        let expected_graph = ProjectGraph::from_json(&project_json).expect("exported graph");
        assert_eq!(recovery.graph, expected_graph);
        assert!(!recovery.materialized_repaired);
        let resolved = journal
            .resolve_named_version(&id(1), "Starting point", &assets)
            .expect("resolve imported named version");
        assert_eq!(resolved.snapshot.snapshot_id, id(100));
        assert!(!resolved.graph.nodes[1].properties.contains_key("historical_asset"));
        let latest_version = journal
            .resolve_named_version(&id(1), "After edit", &assets)
            .expect("resolve latest named version");
        assert_eq!(latest_version.snapshot.snapshot_id, id(102));
        assert_eq!(latest_version.graph, recovery.graph);
        let imported_operation = journal.read_range(&id(1), None, 10).expect("read imported ops");
        assert_eq!(imported_operation.len(), 1);
        assert_eq!(imported_operation[0].envelope.actor.id, "private-actor-id");
        assert_eq!(imported_operation[0].replication, crate::journal::ReplicationState::Pending);

        let collision = import_project_bundle(&bundle_path, &mut journal, &assets)
            .expect_err("same project must not be overwritten");
        assert!(matches!(collision, PortableError::ImportCollision(_)));
        assert_eq!(journal.read_range(&id(1), None, 10).expect("history retained").len(), 1);
    }

    #[test]
    fn empty_operation_history_round_trips_from_a_snapshot() {
        let source = tempfile::tempdir().expect("source workspace");
        let source_database = source.path().join("source.sqlite");
        let source_assets =
            LocalAssetStore::open(source.path(), "empty-source").expect("source assets");
        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture graph");
        graph.asset_refs.clear();
        graph.nodes.iter_mut().find(|node| node.id == id(12)).expect("image node").asset_ref = None;
        graph.validate().expect("graph without assets");
        let mut source_journal = LocalJournal::open(&source_database).expect("source journal");
        source_journal
            .create_snapshot(&id(100), &graph, &source_assets)
            .expect("checkpoint empty history");
        let bundle = source.path().join("bundle");
        export_project_bundle(&mut source_journal, &id(1), &source_assets, &bundle)
            .expect("export bundle");
        let verified = verify_project_bundle(&bundle).expect("verify empty journal");
        assert_eq!(verified.manifest.operations.count, 0);
        assert!(verified.manifest.operations.command_schema_versions.is_empty());
        assert_eq!(fs::metadata(bundle.join("operations.jsonl")).expect("ops file").len(), 0);

        let destination = tempfile::tempdir().expect("destination workspace");
        let destination_database = destination.path().join("destination.sqlite");
        let destination_assets = LocalAssetStore::open(destination.path(), "empty-destination")
            .expect("destination assets");
        let mut destination_journal =
            LocalJournal::open(&destination_database).expect("destination journal");
        import_project_bundle(&bundle, &mut destination_journal, &destination_assets)
            .expect("import empty journal");
        let recovered = destination_journal
            .recover_project(&id(1), &destination_assets)
            .expect("recover imported graph");
        assert_eq!(recovered.graph, graph);
        assert!(
            destination_journal
                .read_range(&id(1), None, 10)
                .expect("empty imported history")
                .is_empty()
        );
    }

    #[test]
    fn verifier_rejects_tampering_unexpected_paths_and_resource_overages() {
        let (_directory, _manifest, bundle_path, _) = export_fixture();
        let strict_limits = BundleImportLimits { max_entries: 1, ..BundleImportLimits::default() };
        assert!(verify_project_bundle_with_limits(&bundle_path, &strict_limits).is_err());

        fs::write(bundle_path.join("unexpected.txt"), b"extra").expect("unexpected file");
        assert!(verify_project_bundle(&bundle_path).is_err());
        fs::remove_file(bundle_path.join("unexpected.txt")).expect("remove test file");

        let mut operations = fs::OpenOptions::new()
            .append(true)
            .open(bundle_path.join("operations.jsonl"))
            .expect("open operation file");
        operations.write_all(b" ").expect("tamper operation file");
        assert!(verify_project_bundle(&bundle_path).is_err());
    }

    #[test]
    fn invalid_bundle_import_changes_neither_project_rows_nor_asset_store() {
        let (_source_directory, _manifest, bundle_path, operation_asset_id) = export_fixture();
        fs::write(bundle_path.join("operations.jsonl"), b"not-json\n")
            .expect("corrupt operation file");

        let destination = tempfile::tempdir().expect("destination workspace");
        let database = destination.path().join("import.sqlite");
        let assets = LocalAssetStore::open(destination.path(), "rejected-import").expect("assets");
        let mut journal = LocalJournal::open(&database).expect("journal");
        assert!(import_project_bundle(&bundle_path, &mut journal, &assets).is_err());
        assert!(journal.read_range(&id(1), None, 10).expect("empty history").is_empty());
        assert!(matches!(
            assets.verify(&operation_asset_id),
            Err(crate::asset_store::AssetError::MissingAsset(_))
        ));
    }

    #[test]
    fn failed_history_insert_rolls_back_all_project_rows() {
        let (_source_directory, _manifest, bundle_path, operation_asset_id) = export_fixture();
        let destination = tempfile::tempdir().expect("destination workspace");
        let database = destination.path().join("import.sqlite");
        let assets =
            LocalAssetStore::open(destination.path(), "transactional-import").expect("assets");
        let mut journal = LocalJournal::open(&database).expect("journal");
        let external = rusqlite::Connection::open(&database).expect("trigger connection");
        external
            .execute_batch(
                "CREATE TRIGGER reject_named_version_import BEFORE INSERT ON named_versions
                 BEGIN SELECT RAISE(ABORT, 'forced import failure'); END;",
            )
            .expect("create abort trigger");

        assert!(import_project_bundle(&bundle_path, &mut journal, &assets).is_err());
        assert!(journal.read_range(&id(1), None, 10).expect("history rollback").is_empty());
        assert!(journal.list_snapshots(&id(1), &assets).expect("snapshot rollback").is_empty());
        assert!(assets.verify(&operation_asset_id).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn verifier_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let (_directory, _manifest, bundle_path, _) = export_fixture();
        symlink("/etc/passwd", bundle_path.join("linked-file")).expect("create hostile symlink");
        assert!(verify_project_bundle(&bundle_path).is_err());
    }

    #[test]
    fn refuses_to_overwrite_existing_destination() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let database = directory.path().join("journal.sqlite");
        let assets = LocalAssetStore::open(directory.path(), "empty-export-test").expect("assets");
        let mut graph = ProjectGraph::from_json(SAMPLE_PROJECT).expect("fixture graph");
        graph.asset_refs.clear();
        graph.nodes.iter_mut().find(|node| node.id == id(12)).expect("image node").asset_ref = None;
        graph.validate().expect("fixture without assets");
        let mut journal = LocalJournal::open(&database).expect("journal");
        journal.create_snapshot(&id(100), &graph, &assets).expect("snapshot");
        let destination = directory.path().join("already-there");
        fs::create_dir(&destination).expect("destination dir");
        fs::write(destination.join("keep.txt"), b"preserve").expect("sentinel");

        let error = export_project_bundle(&mut journal, &id(1), &assets, &destination)
            .expect_err("existing destination must be rejected");
        assert!(matches!(error, PortableError::DestinationExists(_)));
        assert_eq!(
            fs::read(destination.join("keep.txt")).expect("preserved sentinel"),
            b"preserve"
        );
    }

    #[test]
    fn requires_a_snapshot_for_v1_portable_export() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let database = directory.path().join("journal.sqlite");
        let assets = LocalAssetStore::open(directory.path(), "no-snapshot-test").expect("assets");
        let mut journal = LocalJournal::open(&database).expect("journal");
        let error =
            export_project_bundle(&mut journal, &id(1), &assets, directory.path().join("bundle"))
                .expect_err("snapshot is required");
        assert!(matches!(error, PortableError::NoSnapshots(_)));
    }
}
