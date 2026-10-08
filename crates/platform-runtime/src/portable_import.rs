//! Bounded verification and staged import for directory-form project bundles.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs::{self, File},
    io::{self, BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use rusqlite::{Connection, Transaction, TransactionBehavior, params};
use sha2::{Digest, Sha256};

use super::{
    AccessControl, BundleImportLimits, BundleManifest, NamedVersionFile, PortableError,
    asset_relative_path, collect_asset_ids, format_digest, validate_named_versions,
};
use crate::{
    asset_store::{AssetMetadata, LocalAssetStore},
    command::{CausalContext, CommandEnvelope},
    graph::{AssetId, ObjectId, ProjectGraph},
    history::{NamedVersion, Snapshot, validate_snapshot_record},
    journal::{JournalCursor, LocalJournal, materialized_digest},
};

/// Summary returned after independently validating a complete bundle tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleVerification {
    pub manifest: BundleManifest,
    pub total_bytes: u64,
    pub asset_bytes: u64,
}

/// Summary returned after a validated bundle is committed to a local journal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleImportReport {
    pub project_id: ObjectId,
    pub operations_imported: u64,
    pub snapshots_imported: u64,
    pub named_versions_imported: u64,
    pub assets_imported: u64,
    pub total_asset_bytes: u64,
}

struct TreeIndex {
    files: BTreeSet<PathBuf>,
    directories: BTreeSet<PathBuf>,
    lengths: BTreeMap<PathBuf, u64>,
    total_bytes: u64,
    entry_count: usize,
    canonical_root: PathBuf,
}

struct VerifiedBundle {
    root: PathBuf,
    manifest: BundleManifest,
    graph: ProjectGraph,
    operations: Vec<CommandEnvelope>,
    snapshots: Vec<(Snapshot, ProjectGraph)>,
    named_versions: Vec<NamedVersion>,
    total_bytes: u64,
    asset_bytes: u64,
}

type OrderKey = (u64, String, u64);
type AssetRequirements = BTreeMap<AssetId, Option<u64>>;
type OperationVerification = (Vec<CommandEnvelope>, AssetRequirements);
type SnapshotVerification = (Vec<(Snapshot, ProjectGraph)>, BTreeSet<ObjectId>, AssetRequirements);

/// Verifies a bundle using [`BundleImportLimits::default`]. This performs no
/// project or asset-store writes.
///
/// # Errors
///
/// Returns [`PortableError`] when any package path, size, hash, schema, cross-
/// record reference, asset, operation, or recovery invariant is invalid.
pub fn verify_project_bundle(
    bundle_path: impl AsRef<Path>,
) -> Result<BundleVerification, PortableError> {
    verify_project_bundle_with_limits(bundle_path, &BundleImportLimits::default())
}

/// Verifies a bundle with caller-selected resource bounds. No project or asset
/// store is modified.
///
/// # Errors
///
/// Returns [`PortableError`] when any package path, size, hash, schema, cross-
/// record reference, asset, operation, or recovery invariant is invalid.
pub fn verify_project_bundle_with_limits(
    bundle_path: impl AsRef<Path>,
    limits: &BundleImportLimits,
) -> Result<BundleVerification, PortableError> {
    let verified = verify_bundle(bundle_path.as_ref(), limits)?;
    Ok(BundleVerification {
        manifest: verified.manifest,
        total_bytes: verified.total_bytes,
        asset_bytes: verified.asset_bytes,
    })
}

/// Imports a directory bundle using default resource bounds. Bundle validation
/// completes before local assets or project records are written. Project rows
/// are inserted in one `SQLite` transaction; if an asset-copy or database failure
/// occurs, already-ingested immutable blobs may remain unreferenced in the local
/// asset store, but no partial project history is committed.
///
/// # Errors
///
/// Returns [`PortableError`] for invalid bundles, resource-limit violations,
/// project/identity collisions, asset-store failures, or `SQLite` write failures.
pub fn import_project_bundle(
    bundle_path: impl AsRef<Path>,
    journal: &mut LocalJournal,
    assets: &LocalAssetStore,
) -> Result<BundleImportReport, PortableError> {
    import_project_bundle_with_limits(bundle_path, journal, assets, &BundleImportLimits::default())
}

/// Imports a verified bundle with caller-selected resource bounds. Import never
/// overwrites an existing project and never imports membership or capability
/// grants.
///
/// # Errors
///
/// Returns [`PortableError`] for invalid bundles, resource-limit violations,
/// project/identity collisions, asset-store failures, or `SQLite` write failures.
pub fn import_project_bundle_with_limits(
    bundle_path: impl AsRef<Path>,
    journal: &mut LocalJournal,
    assets: &LocalAssetStore,
    limits: &BundleImportLimits,
) -> Result<BundleImportReport, PortableError> {
    let verified = verify_bundle(bundle_path.as_ref(), limits)?;
    check_import_collisions(&journal.connection, &verified)?;

    for asset in &verified.manifest.assets {
        let relative = asset_relative_path(&asset.id)?;
        let source = open_bundle_file(&verified.root, &relative)?;
        if source.metadata()?.len() != asset.byte_length {
            return Err(PortableError::InvalidBundle(format!(
                "asset {} changed after bundle verification",
                asset.id.as_str()
            )));
        }
        let mut bounded = LimitedReader::new(source, asset.byte_length);
        let ingested = assets
            .ingest(&mut bounded, AssetMetadata::new("portable-blob", "portable-bundle-import"))?;
        if ingested.id != asset.id || ingested.byte_length != asset.byte_length {
            return Err(PortableError::InvalidBundle(format!(
                "asset {} changed during import",
                asset.id.as_str()
            )));
        }
    }

    let report = BundleImportReport {
        project_id: verified.manifest.project_id.clone(),
        operations_imported: len_as_u64(verified.operations.len(), "operation count")?,
        snapshots_imported: len_as_u64(verified.snapshots.len(), "snapshot count")?,
        named_versions_imported: len_as_u64(verified.named_versions.len(), "named-version count")?,
        assets_imported: len_as_u64(verified.manifest.assets.len(), "asset count")?,
        total_asset_bytes: verified.asset_bytes,
    };

    let transaction =
        journal.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    check_import_collisions(&transaction, &verified)?;
    insert_operations(&transaction, &verified)?;
    insert_snapshots(&transaction, &verified)?;
    insert_named_versions(&transaction, &verified)?;
    insert_materialized_graph(&transaction, &verified)?;
    transaction.commit()?;
    Ok(report)
}

fn verify_bundle(
    root: &Path,
    limits: &BundleImportLimits,
) -> Result<VerifiedBundle, PortableError> {
    let tree = inspect_tree(root, limits)?;
    let manifest_path = Path::new("manifest.json");
    let manifest_len = required_file_length(&tree, manifest_path)?;
    if manifest_len > limits.max_manifest_bytes {
        return Err(limit_error("manifest bytes", limits.max_manifest_bytes));
    }
    let manifest_bytes = read_bundle_bytes(&tree.canonical_root, manifest_path, manifest_len)?;
    let manifest: BundleManifest = serde_json::from_slice(&manifest_bytes)?;
    validate_manifest_header(&manifest)?;
    check_collection_limits(&manifest, limits)?;

    let expected_files = expected_file_paths(&manifest)?;
    let expected_directories = parent_directories(&expected_files);
    if tree.files != expected_files {
        return Err(path_set_error("file", &expected_files, &tree.files));
    }
    if tree.directories != expected_directories {
        return Err(path_set_error("directory", &expected_directories, &tree.directories));
    }
    let non_asset_bytes = tree
        .lengths
        .iter()
        .filter(|(path, _)| !path.starts_with("assets"))
        .try_fold(0_u64, |total, (_, length)| total.checked_add(*length))
        .ok_or_else(|| PortableError::InvalidBundle("bundle byte count overflow".to_owned()))?;
    if non_asset_bytes > limits.max_non_asset_bytes {
        return Err(limit_error("non-asset bytes", limits.max_non_asset_bytes));
    }

    let project_path = Path::new("project.json");
    let project_len = required_file_length(&tree, project_path)?;
    let project_bytes = read_bundle_bytes(&tree.canonical_root, project_path, project_len)?;
    verify_digest(&project_bytes, &manifest.graph.sha256, "project.json")?;
    let graph_json = std::str::from_utf8(&project_bytes).map_err(|error| {
        PortableError::InvalidBundle(format!("project.json is not UTF-8: {error}"))
    })?;
    let graph = ProjectGraph::from_json(graph_json).map_err(|error| {
        PortableError::InvalidBundle(format!("project graph is invalid: {error}"))
    })?;
    if graph.project_id != manifest.project_id
        || graph.schema_version != manifest.graph.schema_version
    {
        return Err(PortableError::InvalidBundle(
            "manifest graph identity or schema does not match project.json".to_owned(),
        ));
    }

    let (operations, operation_assets) = verify_operations(&tree, &manifest, limits)?;
    let (snapshots, snapshot_file_ids, snapshot_assets) = verify_snapshots(&tree, &manifest)?;
    let named_versions = verify_named_versions(&manifest, &snapshot_file_ids)?;

    let mut referenced_assets = BTreeMap::new();
    super::add_graph_assets(&graph, &mut referenced_assets)?;
    for (_, snapshot_graph) in &snapshots {
        super::add_graph_assets(snapshot_graph, &mut referenced_assets)?;
    }
    for (asset_id, length) in operation_assets {
        super::add_asset(&mut referenced_assets, &asset_id, length)?;
    }
    for (asset_id, length) in snapshot_assets {
        super::add_asset(&mut referenced_assets, &asset_id, length)?;
    }
    verify_asset_closure(&tree, &manifest, &referenced_assets, limits)?;
    verify_snapshot_cursors(&snapshots, &operations)?;
    verify_replay_consistency(&snapshots, &operations, &graph)?;

    let asset_bytes = manifest
        .assets
        .iter()
        .try_fold(0_u64, |total, asset| total.checked_add(asset.byte_length))
        .ok_or_else(|| PortableError::InvalidBundle("asset byte count overflow".to_owned()))?;

    Ok(VerifiedBundle {
        root: tree.canonical_root,
        manifest,
        graph,
        operations,
        snapshots,
        named_versions,
        total_bytes: tree.total_bytes,
        asset_bytes,
    })
}

fn validate_manifest_header(manifest: &BundleManifest) -> Result<(), PortableError> {
    if manifest.format != "unified-creative-project" || manifest.format_version != 1 {
        return Err(PortableError::InvalidBundle(
            "unsupported bundle format or version".to_owned(),
        ));
    }
    manifest
        .project_id
        .validate("manifest.project_id")
        .map_err(|error| PortableError::InvalidBundle(error.to_string()))?;
    if manifest.graph.schema_version != 1 {
        return Err(PortableError::InvalidBundle(format!(
            "unsupported graph schema version {}",
            manifest.graph.schema_version
        )));
    }
    if manifest.operations.order != super::OPERATION_ORDER {
        return Err(PortableError::InvalidBundle("unsupported operation ordering".to_owned()));
    }
    if manifest.snapshots.is_empty() {
        return Err(PortableError::InvalidBundle("bundle has no recovery snapshots".to_owned()));
    }
    if manifest.access_control
        != (AccessControl { memberships_included: false, capability_grants_included: false })
    {
        return Err(PortableError::InvalidBundle(
            "access-control records must not be included".to_owned(),
        ));
    }
    if manifest.operation_actor_identity != "preserved_as_opaque_provenance" {
        return Err(PortableError::InvalidBundle(
            "unsupported operation actor identity policy".to_owned(),
        ));
    }
    if !is_sha256(&manifest.graph.sha256) || !is_sha256(&manifest.operations.sha256) {
        return Err(PortableError::InvalidBundle(
            "manifest has an invalid SHA-256 digest".to_owned(),
        ));
    }
    Ok(())
}

fn check_collection_limits(
    manifest: &BundleManifest,
    limits: &BundleImportLimits,
) -> Result<(), PortableError> {
    if manifest.operations.count > limits.max_operation_count {
        return Err(limit_error("operation count", limits.max_operation_count));
    }
    if manifest.snapshots.len() > limits.max_snapshots {
        return Err(limit_error("snapshot count", limits.max_snapshots));
    }
    if manifest.named_versions.len() > limits.max_named_versions {
        return Err(limit_error("named-version count", limits.max_named_versions));
    }
    if manifest.assets.len() > limits.max_assets {
        return Err(limit_error("asset count", limits.max_assets));
    }
    Ok(())
}

fn expected_file_paths(manifest: &BundleManifest) -> Result<BTreeSet<PathBuf>, PortableError> {
    let mut files = BTreeSet::from([
        PathBuf::from("manifest.json"),
        PathBuf::from("project.json"),
        PathBuf::from("operations.jsonl"),
    ]);
    let mut snapshot_ids = HashSet::new();
    for snapshot in &manifest.snapshots {
        snapshot
            .snapshot_id
            .validate("manifest.snapshot_id")
            .map_err(|error| PortableError::InvalidBundle(error.to_string()))?;
        if !snapshot_ids.insert(snapshot.snapshot_id.clone()) {
            return Err(PortableError::InvalidBundle("duplicate manifest snapshot ID".to_owned()));
        }
        if !is_sha256(&snapshot.sha256) {
            return Err(PortableError::InvalidBundle("invalid snapshot file digest".to_owned()));
        }
        files.insert(
            PathBuf::from("history")
                .join("snapshots")
                .join(format!("{}.json", snapshot.snapshot_id.as_str())),
        );
    }
    let mut asset_ids = HashSet::new();
    for asset in &manifest.assets {
        if !asset_ids.insert(asset.id.clone()) {
            return Err(PortableError::InvalidBundle("duplicate manifest asset ID".to_owned()));
        }
        files.insert(asset_relative_path(&asset.id)?);
    }
    Ok(files)
}

fn parent_directories(files: &BTreeSet<PathBuf>) -> BTreeSet<PathBuf> {
    let mut directories = BTreeSet::new();
    for file in files {
        let mut parent = file.parent();
        while let Some(path) = parent {
            if path.as_os_str().is_empty() {
                break;
            }
            directories.insert(path.to_path_buf());
            parent = path.parent();
        }
    }
    directories
}

fn inspect_tree(root: &Path, limits: &BundleImportLimits) -> Result<TreeIndex, PortableError> {
    let root_metadata = fs::symlink_metadata(root)?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(PortableError::InvalidBundle(
            "bundle root must be a real directory, not a symlink".to_owned(),
        ));
    }
    let canonical_root = root.canonicalize()?;
    let mut tree = TreeIndex {
        files: BTreeSet::new(),
        directories: BTreeSet::new(),
        lengths: BTreeMap::new(),
        total_bytes: 0,
        entry_count: 0,
        canonical_root: canonical_root.clone(),
    };
    let mut pending_directories = vec![canonical_root.clone()];
    while let Some(directory) = pending_directories.pop() {
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            tree.entry_count = tree.entry_count.checked_add(1).ok_or_else(|| {
                PortableError::InvalidBundle("bundle entry count overflow".to_owned())
            })?;
            if tree.entry_count > limits.max_entries {
                return Err(limit_error("file and directory count", limits.max_entries));
            }
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink() {
                return Err(PortableError::InvalidBundle(format!(
                    "symlink is not allowed in bundle: {}",
                    path.display()
                )));
            }
            let canonical = path.canonicalize()?;
            if !canonical.starts_with(&canonical_root) {
                return Err(PortableError::InvalidBundle(format!(
                    "bundle path escapes its root: {}",
                    path.display()
                )));
            }
            let relative = path
                .strip_prefix(&canonical_root)
                .map_err(|_| {
                    PortableError::InvalidBundle("bundle path escaped its root".to_owned())
                })?
                .to_path_buf();
            if metadata.is_dir() {
                tree.directories.insert(relative);
                pending_directories.push(path);
            } else if metadata.is_file() {
                if metadata.len() > limits.max_single_file_bytes {
                    return Err(limit_error("single-file bytes", limits.max_single_file_bytes));
                }
                tree.total_bytes =
                    tree.total_bytes.checked_add(metadata.len()).ok_or_else(|| {
                        PortableError::InvalidBundle("bundle byte count overflow".to_owned())
                    })?;
                if tree.total_bytes > limits.max_total_bytes {
                    return Err(limit_error("total bundle bytes", limits.max_total_bytes));
                }
                tree.lengths.insert(relative.clone(), metadata.len());
                tree.files.insert(relative);
            } else {
                return Err(PortableError::InvalidBundle(format!(
                    "non-regular filesystem entry in bundle: {}",
                    path.display()
                )));
            }
        }
    }
    Ok(tree)
}

fn required_file_length(tree: &TreeIndex, relative: &Path) -> Result<u64, PortableError> {
    tree.lengths.get(relative).copied().ok_or_else(|| {
        PortableError::InvalidBundle(format!(
            "required bundle file is missing: {}",
            relative.display()
        ))
    })
}

fn read_bundle_bytes(
    root: &Path,
    relative: &Path,
    expected_length: u64,
) -> Result<Vec<u8>, PortableError> {
    let file = open_bundle_file(root, relative)?;
    let actual_length = file.metadata()?.len();
    if actual_length != expected_length {
        return Err(PortableError::InvalidBundle(format!(
            "bundle file changed while reading: {}",
            relative.display()
        )));
    }
    let capacity = usize::try_from(expected_length).map_err(|_| {
        PortableError::InvalidBundle("bundle file is too large for this platform".to_owned())
    })?;
    let mut bytes = Vec::with_capacity(capacity);
    let read_limit = expected_length.checked_add(1).ok_or_else(|| {
        PortableError::InvalidBundle("bundle file length is out of range".to_owned())
    })?;
    file.take(read_limit).read_to_end(&mut bytes)?;
    if bytes.len() != capacity {
        return Err(PortableError::InvalidBundle(format!(
            "bundle file changed while reading: {}",
            relative.display()
        )));
    }
    Ok(bytes)
}

fn open_bundle_file(root: &Path, relative: &Path) -> Result<File, PortableError> {
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(PortableError::InvalidBundle(format!(
            "bundle entry is not a regular file: {}",
            relative.display()
        )));
    }
    let canonical = path.canonicalize()?;
    if !canonical.starts_with(root) {
        return Err(PortableError::InvalidBundle(format!(
            "bundle path escapes its root: {}",
            relative.display()
        )));
    }
    Ok(File::open(canonical)?)
}

fn path_set_error(
    kind: &str,
    expected: &BTreeSet<PathBuf>,
    actual: &BTreeSet<PathBuf>,
) -> PortableError {
    if let Some(extra) = actual.difference(expected).next() {
        PortableError::InvalidBundle(format!("unexpected {kind}: {}", extra.display()))
    } else if let Some(missing) = expected.difference(actual).next() {
        PortableError::InvalidBundle(format!("missing {kind}: {}", missing.display()))
    } else {
        PortableError::InvalidBundle(format!("bundle {kind} layout is inconsistent"))
    }
}

fn verify_operations(
    tree: &TreeIndex,
    manifest: &BundleManifest,
    limits: &BundleImportLimits,
) -> Result<OperationVerification, PortableError> {
    let relative = Path::new("operations.jsonl");
    let expected_length = required_file_length(tree, relative)?;
    let file = open_bundle_file(&tree.canonical_root, relative)?;
    if file.metadata()?.len() != expected_length {
        return Err(PortableError::InvalidBundle(
            "operations.jsonl changed during verification".to_owned(),
        ));
    }
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut operations = Vec::new();
    let mut assets = BTreeMap::new();
    let mut schema_versions = BTreeSet::new();
    let mut operation_ids = HashSet::new();
    let mut client_sequences = HashSet::new();
    let mut previous_order: Option<OrderKey> = None;
    let mut observed_bytes = 0_u64;

    loop {
        let Some(line) = read_bounded_line(&mut reader, limits.max_operation_line_bytes)? else {
            break;
        };
        observed_bytes = observed_bytes
            .checked_add(
                u64::try_from(line.len()).map_err(|_| {
                    PortableError::InvalidBundle("operation bytes overflow".to_owned())
                })?,
            )
            .ok_or_else(|| PortableError::InvalidBundle("operation bytes overflow".to_owned()))?;
        if observed_bytes > limits.max_non_asset_bytes {
            return Err(limit_error("operation journal bytes", limits.max_non_asset_bytes));
        }
        hasher.update(&line);
        let payload = &line[..line.len() - 1];
        let envelope: CommandEnvelope = serde_json::from_slice(payload)?;
        envelope.validate().map_err(|error| {
            PortableError::InvalidBundle(format!(
                "operation {} is invalid: {error}",
                envelope.operation_id.as_str()
            ))
        })?;
        if envelope.project_id != manifest.project_id {
            return Err(PortableError::InvalidBundle(format!(
                "operation {} belongs to another project",
                envelope.operation_id.as_str()
            )));
        }
        let order = causal_order_key(&envelope.causal);
        if previous_order.as_ref().is_some_and(|previous| previous >= &order) {
            return Err(PortableError::InvalidBundle(
                "operations are not strictly ordered".to_owned(),
            ));
        }
        if !operation_ids.insert(envelope.operation_id.clone()) {
            return Err(PortableError::InvalidBundle("duplicate operation ID".to_owned()));
        }
        if !client_sequences
            .insert((envelope.causal.client_id.clone(), envelope.causal.client_sequence))
        {
            return Err(PortableError::InvalidBundle("duplicate client sequence".to_owned()));
        }
        let command_json = serde_json::to_value(&envelope.command)?;
        collect_asset_ids(&command_json, &mut assets);
        schema_versions.insert(envelope.schema_version);
        previous_order = Some(order);
        operations.push(envelope);
        if len_as_u64(operations.len(), "operation count")? > limits.max_operation_count {
            return Err(limit_error("operation count", limits.max_operation_count));
        }
    }

    let observed_count = len_as_u64(operations.len(), "operation count")?;
    if observed_bytes != expected_length {
        return Err(PortableError::InvalidBundle(
            "operations.jsonl changed while it was being verified".to_owned(),
        ));
    }
    if observed_count != manifest.operations.count {
        return Err(PortableError::InvalidBundle(
            "operation count does not match manifest".to_owned(),
        ));
    }
    if format_digest(&hasher.finalize()) != manifest.operations.sha256 {
        return Err(PortableError::InvalidBundle("operations.jsonl digest mismatch".to_owned()));
    }
    let observed_schemas: Vec<_> = schema_versions.into_iter().collect();
    if observed_schemas != manifest.operations.command_schema_versions {
        return Err(PortableError::InvalidBundle(
            "operation schema versions do not match manifest".to_owned(),
        ));
    }
    Ok((operations, assets))
}

fn read_bounded_line<R: BufRead>(
    reader: &mut R,
    max_bytes: usize,
) -> Result<Option<Vec<u8>>, PortableError> {
    let mut line = Vec::new();
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            if line.is_empty() {
                return Ok(None);
            }
            return Err(PortableError::InvalidBundle(
                "operations.jsonl ends with an unterminated line".to_owned(),
            ));
        }
        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(buffer.len(), |index| index + 1);
        if line.len().saturating_add(consumed) > max_bytes {
            return Err(limit_error("operation line bytes", max_bytes));
        }
        line.extend_from_slice(&buffer[..consumed]);
        reader.consume(consumed);
        if newline.is_some() {
            if line.len() == 1 {
                return Err(PortableError::InvalidBundle("blank operation line".to_owned()));
            }
            return Ok(Some(line));
        }
    }
}

fn verify_snapshots(
    tree: &TreeIndex,
    manifest: &BundleManifest,
) -> Result<SnapshotVerification, PortableError> {
    let mut snapshots = Vec::with_capacity(manifest.snapshots.len());
    let mut snapshot_ids = BTreeSet::new();
    let mut assets = BTreeMap::new();
    for entry in &manifest.snapshots {
        let relative = PathBuf::from("history")
            .join("snapshots")
            .join(format!("{}.json", entry.snapshot_id.as_str()));
        let length = required_file_length(tree, &relative)?;
        let bytes = read_bundle_bytes(&tree.canonical_root, &relative, length)?;
        verify_digest(&bytes, &entry.sha256, &relative.display().to_string())?;
        let snapshot: Snapshot = serde_json::from_slice(&bytes)?;
        if snapshot.snapshot_id != entry.snapshot_id {
            return Err(PortableError::InvalidBundle(
                "snapshot ID does not match manifest".to_owned(),
            ));
        }
        let graph = validate_snapshot_record(&snapshot, &manifest.project_id).map_err(|error| {
            PortableError::InvalidBundle(format!("snapshot is invalid: {error}"))
        })?;
        for asset in &graph.asset_refs {
            super::add_asset(&mut assets, &asset.id, Some(asset.byte_length))?;
        }
        snapshot_ids.insert(snapshot.snapshot_id.clone());
        snapshots.push((snapshot, graph));
    }
    Ok((snapshots, snapshot_ids, assets))
}

fn verify_named_versions(
    manifest: &BundleManifest,
    snapshot_ids: &BTreeSet<ObjectId>,
) -> Result<Vec<NamedVersion>, PortableError> {
    let versions: Vec<_> = manifest
        .named_versions
        .iter()
        .map(|entry: &NamedVersionFile| NamedVersion {
            version_id: entry.version_id.clone(),
            project_id: manifest.project_id.clone(),
            name: entry.name.clone(),
            snapshot_id: entry.snapshot_id.clone(),
        })
        .collect();
    for version in &versions {
        version
            .version_id
            .validate("manifest.version_id")
            .map_err(|error| PortableError::InvalidBundle(error.to_string()))?;
        version
            .snapshot_id
            .validate("manifest.named_version.snapshot_id")
            .map_err(|error| PortableError::InvalidBundle(error.to_string()))?;
    }
    validate_named_versions(&manifest.project_id, snapshot_ids, &versions)?;
    Ok(versions)
}

fn verify_asset_closure(
    tree: &TreeIndex,
    manifest: &BundleManifest,
    referenced: &BTreeMap<AssetId, Option<u64>>,
    limits: &BundleImportLimits,
) -> Result<(), PortableError> {
    let declared: BTreeMap<_, _> =
        manifest.assets.iter().map(|asset| (asset.id.clone(), asset.byte_length)).collect();
    if declared.len() != manifest.assets.len() {
        return Err(PortableError::InvalidBundle("duplicate manifest asset ID".to_owned()));
    }
    let referenced_ids: BTreeSet<_> = referenced.keys().cloned().collect();
    let declared_ids: BTreeSet<_> = declared.keys().cloned().collect();
    if referenced_ids != declared_ids {
        return Err(pathless_asset_set_error(&referenced_ids, &declared_ids));
    }
    let mut total_asset_bytes = 0_u64;
    for asset in &manifest.assets {
        if asset.byte_length > limits.max_single_file_bytes {
            return Err(limit_error("single asset bytes", limits.max_single_file_bytes));
        }
        if let Some(Some(expected)) = referenced.get(&asset.id)
            && *expected != asset.byte_length
        {
            return Err(PortableError::AssetLengthMismatch {
                id: asset.id.clone(),
                expected: *expected,
                actual: asset.byte_length,
            });
        }
        let relative = asset_relative_path(&asset.id)?;
        let file_length = required_file_length(tree, &relative)?;
        if file_length != asset.byte_length {
            return Err(PortableError::AssetLengthMismatch {
                id: asset.id.clone(),
                expected: asset.byte_length,
                actual: file_length,
            });
        }
        let file = open_bundle_file(&tree.canonical_root, &relative)?;
        let (digest, length) = hash_reader(file, limits.max_single_file_bytes)?;
        let id_digest = asset
            .id
            .as_str()
            .strip_prefix("sha256:")
            .ok_or_else(|| PortableError::InvalidBundle("invalid asset ID".to_owned()))?;
        if digest != id_digest || length != asset.byte_length {
            return Err(PortableError::InvalidBundle(format!(
                "asset {} failed digest or length validation",
                asset.id.as_str()
            )));
        }
        total_asset_bytes = total_asset_bytes
            .checked_add(length)
            .ok_or_else(|| PortableError::InvalidBundle("asset byte count overflow".to_owned()))?;
    }
    if total_asset_bytes > limits.max_total_bytes {
        return Err(limit_error("total asset bytes", limits.max_total_bytes));
    }
    Ok(())
}

fn verify_snapshot_cursors(
    snapshots: &[(Snapshot, ProjectGraph)],
    operations: &[CommandEnvelope],
) -> Result<(), PortableError> {
    let operation_cursors: HashSet<_> =
        operations.iter().map(|operation| causal_order_key(&operation.causal)).collect();
    for (snapshot, _) in snapshots {
        if let Some(cursor) = &snapshot.cursor
            && !operation_cursors.contains(&order_key(cursor))
        {
            return Err(PortableError::InvalidBundle(format!(
                "snapshot {} cursor is not present in the operation journal",
                snapshot.snapshot_id.as_str()
            )));
        }
    }
    Ok(())
}

fn verify_replay_consistency(
    snapshots: &[(Snapshot, ProjectGraph)],
    operations: &[CommandEnvelope],
    current_graph: &ProjectGraph,
) -> Result<(), PortableError> {
    let Some((earliest_snapshot, earliest_graph)) = snapshots.iter().min_by_key(|(snapshot, _)| {
        snapshot.cursor.as_ref().map_or((0, String::new(), 0), order_key)
    }) else {
        return Err(PortableError::InvalidBundle("bundle has no recovery snapshot".to_owned()));
    };
    let earliest_key = earliest_snapshot.cursor.as_ref().map(order_key);
    let mut reconstructed = earliest_graph.clone();
    let mut checkpoints: BTreeMap<OrderKey, Vec<&ProjectGraph>> = BTreeMap::new();
    for (snapshot, graph) in snapshots {
        if let Some(cursor) = &snapshot.cursor {
            checkpoints.entry(order_key(cursor)).or_default().push(graph);
        } else if graph != &reconstructed {
            return Err(PortableError::InvalidBundle(
                "snapshots at the initial cursor disagree".to_owned(),
            ));
        }
    }
    if let Some(key) = &earliest_key
        && checkpoints
            .get(key)
            .is_some_and(|graphs| graphs.iter().any(|graph| **graph != reconstructed))
    {
        return Err(PortableError::InvalidBundle(
            "snapshots at the earliest cursor disagree".to_owned(),
        ));
    }

    for operation in operations {
        let cursor = causal_order_key(&operation.causal);
        if earliest_key.as_ref().is_some_and(|earliest| cursor <= *earliest) {
            continue;
        }
        reconstructed.apply_envelope_to_staged(operation).map_err(|error| {
            PortableError::InvalidBundle(format!("operation replay failed: {error}"))
        })?;
        reconstructed.validate().map_err(|error| {
            PortableError::InvalidBundle(format!("replayed graph is invalid: {error}"))
        })?;
        if let Some(graphs) = checkpoints.get(&cursor)
            && graphs.iter().any(|graph| **graph != reconstructed)
        {
            return Err(PortableError::InvalidBundle(format!(
                "snapshot at operation {} does not match replayed history",
                operation.operation_id.as_str()
            )));
        }
    }
    if reconstructed != *current_graph {
        return Err(PortableError::InvalidBundle(
            "replayed history does not match project.json".to_owned(),
        ));
    }
    Ok(())
}

fn order_key(cursor: &JournalCursor) -> OrderKey {
    (cursor.lamport, cursor.client_id.clone(), cursor.client_sequence)
}

fn causal_order_key(causal: &CausalContext) -> OrderKey {
    (causal.lamport, causal.client_id.clone(), causal.client_sequence)
}

fn verify_digest(bytes: &[u8], expected: &str, file_name: &str) -> Result<(), PortableError> {
    let actual = format_digest(&Sha256::digest(bytes));
    if actual != expected {
        return Err(PortableError::InvalidBundle(format!("{file_name} digest mismatch")));
    }
    Ok(())
}

fn hash_reader(mut reader: impl Read, max_bytes: u64) -> Result<(String, u64), PortableError> {
    let mut hasher = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        length = length
            .checked_add(
                u64::try_from(read).map_err(|_| {
                    PortableError::InvalidBundle("file byte count overflow".to_owned())
                })?,
            )
            .ok_or_else(|| PortableError::InvalidBundle("file byte count overflow".to_owned()))?;
        if length > max_bytes {
            return Err(limit_error("single-file bytes", max_bytes));
        }
        hasher.update(&buffer[..read]);
    }
    Ok((super::hex_digest(&hasher.finalize()), length))
}

fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn limit_error(resource: &str, limit: impl std::fmt::Display) -> PortableError {
    PortableError::InvalidBundle(format!(
        "bundle exceeds the configured {resource} limit ({limit})"
    ))
}

fn pathless_asset_set_error(
    referenced: &BTreeSet<AssetId>,
    declared: &BTreeSet<AssetId>,
) -> PortableError {
    if let Some(missing) = referenced.difference(declared).next() {
        PortableError::InvalidBundle(format!(
            "referenced asset {} is missing from manifest",
            missing.as_str()
        ))
    } else if let Some(extra) = declared.difference(referenced).next() {
        PortableError::InvalidBundle(format!(
            "manifest contains unreferenced asset {}",
            extra.as_str()
        ))
    } else {
        PortableError::InvalidBundle("asset closure is inconsistent".to_owned())
    }
}

fn check_import_collisions(
    connection: &Connection,
    bundle: &VerifiedBundle,
) -> Result<(), PortableError> {
    let project_id = bundle.manifest.project_id.as_str();
    let project_exists: bool = connection.query_row(
        "SELECT EXISTS(
            SELECT project_id FROM materialized_projects WHERE project_id = ?1
            UNION ALL SELECT project_id FROM operations WHERE project_id = ?1
            UNION ALL SELECT project_id FROM snapshots WHERE project_id = ?1
            UNION ALL SELECT project_id FROM named_versions WHERE project_id = ?1
        )",
        [project_id],
        |row| row.get(0),
    )?;
    if project_exists {
        return Err(PortableError::ImportCollision(format!(
            "project {project_id} already exists; import never overwrites or merges projects"
        )));
    }

    let mut operation_id_check =
        connection.prepare("SELECT EXISTS(SELECT 1 FROM operations WHERE operation_id = ?1)")?;
    let mut client_sequence_check = connection.prepare(
        "SELECT EXISTS(SELECT 1 FROM operations WHERE client_id = ?1 AND client_sequence = ?2)",
    )?;
    for operation in &bundle.operations {
        let operation_exists: bool =
            operation_id_check.query_row([operation.operation_id.as_str()], |row| row.get(0))?;
        let sequence = sql_integer(operation.causal.client_sequence, "client_sequence")?;
        let sequence_exists: bool = client_sequence_check
            .query_row(params![operation.causal.client_id, sequence], |row| row.get(0))?;
        if operation_exists || sequence_exists {
            return Err(PortableError::ImportCollision(format!(
                "operation {} or its client sequence already exists",
                operation.operation_id.as_str()
            )));
        }
    }
    drop(operation_id_check);
    drop(client_sequence_check);

    let mut snapshot_check =
        connection.prepare("SELECT EXISTS(SELECT 1 FROM snapshots WHERE snapshot_id = ?1)")?;
    for (snapshot, _) in &bundle.snapshots {
        let exists: bool =
            snapshot_check.query_row([snapshot.snapshot_id.as_str()], |row| row.get(0))?;
        if exists {
            return Err(PortableError::ImportCollision(format!(
                "snapshot ID {} already exists",
                snapshot.snapshot_id.as_str()
            )));
        }
    }
    drop(snapshot_check);

    let mut version_check =
        connection.prepare("SELECT EXISTS(SELECT 1 FROM named_versions WHERE version_id = ?1)")?;
    for version in &bundle.named_versions {
        let exists: bool =
            version_check.query_row([version.version_id.as_str()], |row| row.get(0))?;
        if exists {
            return Err(PortableError::ImportCollision(format!(
                "named-version ID {} already exists",
                version.version_id.as_str()
            )));
        }
    }
    Ok(())
}

fn insert_operations(
    transaction: &Transaction<'_>,
    bundle: &VerifiedBundle,
) -> Result<(), PortableError> {
    for envelope in &bundle.operations {
        let sequence = sql_integer(envelope.causal.client_sequence, "client_sequence")?;
        let lamport = sql_integer(envelope.causal.lamport, "lamport")?;
        let payload_json = serde_json::to_string(envelope)?;
        transaction.execute(
            "INSERT INTO operations (
                operation_id, project_id, target_id, client_id, client_sequence,
                lamport, parent_operation_id, payload_json, remote_acknowledged
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)",
            params![
                envelope.operation_id.as_str(),
                envelope.project_id.as_str(),
                envelope.target_id.as_str(),
                envelope.causal.client_id,
                sequence,
                lamport,
                envelope.causal.parent_operation_id.as_ref().map(ObjectId::as_str),
                payload_json,
            ],
        )?;
    }
    Ok(())
}

fn insert_snapshots(
    transaction: &Transaction<'_>,
    bundle: &VerifiedBundle,
) -> Result<(), PortableError> {
    for (snapshot, _) in &bundle.snapshots {
        let (cursor_lamport, cursor_client_id, cursor_client_sequence) =
            if let Some(cursor) = &snapshot.cursor {
                (
                    Some(sql_integer(cursor.lamport, "snapshot.cursor.lamport")?),
                    Some(cursor.client_id.as_str()),
                    Some(sql_integer(cursor.client_sequence, "snapshot.cursor.client_sequence")?),
                )
            } else {
                (None, None, None)
            };
        transaction.execute(
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
                cursor_lamport,
                cursor_client_id,
                cursor_client_sequence,
                snapshot.graph_json,
                snapshot.record_sha256,
            ],
        )?;
    }
    Ok(())
}

fn insert_named_versions(
    transaction: &Transaction<'_>,
    bundle: &VerifiedBundle,
) -> Result<(), PortableError> {
    for version in &bundle.named_versions {
        transaction.execute(
            "INSERT INTO named_versions (version_id, project_id, name, snapshot_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                version.version_id.as_str(),
                version.project_id.as_str(),
                version.name,
                version.snapshot_id.as_str(),
            ],
        )?;
    }
    Ok(())
}

fn insert_materialized_graph(
    transaction: &Transaction<'_>,
    bundle: &VerifiedBundle,
) -> Result<(), PortableError> {
    let graph_json = serde_json::to_string(&bundle.graph)?;
    let cursor = bundle.operations.last().map(|operation| JournalCursor {
        lamport: operation.causal.lamport,
        client_id: operation.causal.client_id.clone(),
        client_sequence: operation.causal.client_sequence,
    });
    let (cursor_lamport, cursor_client_id, cursor_client_sequence) =
        if let Some(cursor) = cursor.as_ref() {
            (
                Some(sql_integer(cursor.lamport, "materialized.cursor.lamport")?),
                Some(cursor.client_id.as_str()),
                Some(sql_integer(cursor.client_sequence, "materialized.cursor.client_sequence")?),
            )
        } else {
            (None, None, None)
        };
    let state_sha256 =
        materialized_digest(&bundle.manifest.project_id, &graph_json, cursor.as_ref());
    transaction.execute(
        "INSERT INTO materialized_projects (
            project_id, graph_json, state_sha256, cursor_lamport, cursor_client_id,
            cursor_client_sequence
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            bundle.manifest.project_id.as_str(),
            graph_json,
            state_sha256,
            cursor_lamport,
            cursor_client_id,
            cursor_client_sequence,
        ],
    )?;
    Ok(())
}

fn sql_integer(value: u64, field: &'static str) -> Result<i64, PortableError> {
    i64::try_from(value)
        .map_err(|_| PortableError::InvalidBundle(format!("{field} exceeds SQLite integer range")))
}

fn len_as_u64(length: usize, field: &str) -> Result<u64, PortableError> {
    u64::try_from(length)
        .map_err(|_| PortableError::InvalidBundle(format!("{field} exceeds supported range")))
}

struct LimitedReader<R> {
    inner: R,
    remaining: u64,
}

impl<R> LimitedReader<R> {
    fn new(inner: R, limit: u64) -> Self {
        Self { inner, remaining: limit }
    }
}

impl<R: Read> Read for LimitedReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        if self.remaining == 0 {
            let mut probe = [0_u8; 1];
            return match self.inner.read(&mut probe)? {
                0 => Ok(0),
                _ => {
                    Err(io::Error::new(io::ErrorKind::InvalidData, "asset exceeds declared length"))
                }
            };
        }
        let remaining = usize::try_from(self.remaining).unwrap_or(buffer.len());
        let allowed = remaining.min(buffer.len());
        let read = self.inner.read(&mut buffer[..allowed])?;
        let read_bytes = u64::try_from(read)
            .map_err(|_| io::Error::other("asset byte count exceeds the supported range"))?;
        self.remaining -= read_bytes;
        Ok(read)
    }
}
