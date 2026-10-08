//! Immutable, content-addressed local asset storage.
//!
//! Each store is rooted under a hash of the caller-supplied trusted tenant or
//! authorization scope. Content-addressed deduplication therefore remains
//! inside that scope. The graph stores metadata and references; this module
//! stores bytes and never places them in operation payloads.

use std::{
    fmt,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use serde_json::Value;
use sha2::{Digest, Sha256};
use tempfile::Builder;

use crate::graph::{AssetId, AssetRef};

/// Local object store scoped to one trusted tenant or authorization domain.
pub struct LocalAssetStore {
    root: PathBuf,
}

/// Metadata recorded alongside an ingested asset in the project graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetMetadata {
    pub kind: String,
    pub provenance: String,
    pub rights: Option<Value>,
    pub source_revision: Option<String>,
    pub transform: Option<Value>,
}

impl AssetMetadata {
    /// Creates metadata for an imported or generated asset.
    #[must_use]
    pub fn new(kind: impl Into<String>, provenance: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            provenance: provenance.into(),
            rights: None,
            source_revision: None,
            transform: None,
        }
    }

    /// Creates metadata for an output derived from a specific project revision.
    #[must_use]
    pub fn derived(
        kind: impl Into<String>,
        provenance: impl Into<String>,
        source_revision: impl Into<String>,
        transform: Value,
    ) -> Self {
        Self {
            kind: kind.into(),
            provenance: provenance.into(),
            rights: None,
            source_revision: Some(source_revision.into()),
            transform: Some(transform),
        }
    }

    fn validate(&self) -> Result<(), AssetError> {
        if self.kind.trim().is_empty() {
            return Err(AssetError::EmptyKind);
        }
        if self.provenance.trim().is_empty() {
            return Err(AssetError::EmptyProvenance);
        }
        if self.source_revision.is_some() != self.transform.is_some() {
            return Err(AssetError::IncompleteDerivation);
        }
        Ok(())
    }
}

/// Result of verifying an object’s content digest and byte count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetVerification {
    pub id: AssetId,
    pub byte_length: u64,
}

/// Errors from local asset ingestion and integrity checks.
#[derive(Debug)]
pub enum AssetError {
    Io(io::Error),
    EmptyScope,
    EmptyKind,
    EmptyProvenance,
    IncompleteDerivation,
    InvalidAssetId(String),
    MissingAsset(AssetId),
    CorruptAsset { id: AssetId, actual_digest: String },
    ByteLengthOverflow,
}

impl fmt::Display for AssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "asset store I/O error: {error}"),
            Self::EmptyScope => formatter.write_str("asset store scope must not be empty"),
            Self::EmptyKind => formatter.write_str("asset kind must not be empty"),
            Self::EmptyProvenance => formatter.write_str("asset provenance must not be empty"),
            Self::IncompleteDerivation => formatter.write_str(
                "derived assets must include both source revision and transform parameters",
            ),
            Self::InvalidAssetId(id) => {
                write!(formatter, "invalid content-addressed asset ID: {id}")
            }
            Self::MissingAsset(id) => write!(formatter, "asset {} is missing", id.as_str()),
            Self::CorruptAsset { id, actual_digest } => write!(
                formatter,
                "asset {} failed integrity check; actual SHA-256 is {actual_digest}",
                id.as_str()
            ),
            Self::ByteLengthOverflow => formatter.write_str("asset byte length overflowed u64"),
        }
    }
}

impl std::error::Error for AssetError {}

impl From<io::Error> for AssetError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl LocalAssetStore {
    /// Opens an asset store isolated to a trusted tenant or authorization scope.
    ///
    /// The `scope_id` must come from an authenticated policy boundary. A stable
    /// hash of it selects a separate namespace beneath `base_path`; raw scope
    /// identifiers are never used as filesystem path components.
    ///
    /// # Errors
    ///
    /// Returns [`AssetError`] for an empty scope or filesystem failures.
    pub fn open(base_path: impl AsRef<Path>, scope_id: &str) -> Result<Self, AssetError> {
        if scope_id.trim().is_empty() {
            return Err(AssetError::EmptyScope);
        }
        let scope_digest = sha256(scope_id.as_bytes());
        let root = base_path.as_ref().join("scopes").join(scope_digest);
        let objects = root.join("objects");
        let staging = root.join("staging");
        fs::create_dir_all(&objects)?;
        fs::create_dir_all(&staging)?;
        Ok(Self { root })
    }

    /// Removes abandoned files created by interrupted ingests.
    ///
    /// Call this during a quiescent maintenance window with no concurrent
    /// writers for this scope. It removes only `.ingest-*.partial` files in
    /// this store’s private staging directory.
    ///
    /// # Errors
    ///
    /// Returns [`AssetError`] when the staging directory cannot be read or an
    /// owned partial file cannot be removed.
    pub fn cleanup_abandoned_staging(&self) -> Result<usize, AssetError> {
        clean_abandoned_staging(&self.root.join("staging"))
    }

    /// Streams bytes into an atomic content-addressed object and returns graph metadata.
    ///
    /// Same-byte ingest is idempotent within this store scope. A failed read or
    /// write removes the staging file and never publishes a partial object.
    ///
    /// # Errors
    ///
    /// Returns [`AssetError`] for invalid metadata, source I/O, staging or
    /// placement failures, or an existing object that fails digest validation.
    pub fn ingest<R: Read>(
        &self,
        source: &mut R,
        metadata: AssetMetadata,
    ) -> Result<AssetRef, AssetError> {
        metadata.validate()?;
        let staging = self.root.join("staging");
        let mut temporary =
            Builder::new().prefix(".ingest-").suffix(".partial").tempfile_in(staging)?;
        let mut hasher = Sha256::new();
        let mut byte_length = 0_u64;
        let mut buffer = vec![0_u8; 64 * 1024];

        loop {
            let read = source.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            byte_length = byte_length
                .checked_add(u64::try_from(read).map_err(|_| AssetError::ByteLengthOverflow)?)
                .ok_or(AssetError::ByteLengthOverflow)?;
            temporary.write_all(&buffer[..read])?;
            hasher.update(&buffer[..read]);
        }
        temporary.as_file().sync_all()?;

        let digest = hex_digest(hasher.finalize().as_slice());
        let id = AssetId::new(format!("sha256:{digest}"));
        let destination = self.object_path(&digest);
        let Some(parent) = destination.parent() else {
            return Err(AssetError::Io(io::Error::other("asset object path has no parent")));
        };
        fs::create_dir_all(parent)?;

        if destination.exists() {
            verify_path(&destination, &id, byte_length)?;
        } else {
            match temporary.persist_noclobber(&destination) {
                Ok(file) => drop(file),
                Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => {
                    verify_path(&destination, &id, byte_length)?;
                }
                Err(error) => return Err(AssetError::Io(error.error)),
            }
            sync_directory(parent)?;
        }

        Ok(AssetRef {
            id,
            kind: metadata.kind,
            byte_length,
            provenance: metadata.provenance,
            rights: metadata.rights,
            source_revision: metadata.source_revision,
            transform: metadata.transform,
        })
    }

    /// Verifies an asset’s on-disk bytes against its content-addressed ID.
    ///
    /// # Errors
    ///
    /// Returns [`AssetError::MissingAsset`] when no object exists, or
    /// [`AssetError::CorruptAsset`] when its computed digest differs.
    pub fn verify(&self, id: &AssetId) -> Result<AssetVerification, AssetError> {
        let digest = digest_from_id(id)?;
        let path = self.object_path(digest);
        let metadata = fs::metadata(&path).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                AssetError::MissingAsset(id.clone())
            } else {
                AssetError::Io(error)
            }
        })?;
        let byte_length = metadata.len();
        verify_path(&path, id, byte_length)?;
        Ok(AssetVerification { id: id.clone(), byte_length })
    }

    /// Copies verified asset bytes to a caller-provided writer.
    ///
    /// Verification completes before any bytes are written to `destination`.
    ///
    /// # Errors
    ///
    /// Returns [`AssetError`] when the source is missing/corrupt or either
    /// filesystem or destination I/O fails.
    pub fn copy_verified_to<W: Write>(
        &self,
        id: &AssetId,
        destination: &mut W,
    ) -> Result<u64, AssetError> {
        self.verify(id)?;
        let mut source = File::open(self.object_path(digest_from_id(id)?))?;
        let byte_length = io::copy(&mut source, destination)?;
        Ok(byte_length)
    }

    fn object_path(&self, digest: &str) -> PathBuf {
        self.root.join("objects").join(&digest[..2]).join(&digest[2..])
    }
}

fn digest_from_id(id: &AssetId) -> Result<&str, AssetError> {
    let digest = id.as_str().strip_prefix("sha256:").unwrap_or_default();
    if digest.len() == 64
        && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(digest)
    } else {
        Err(AssetError::InvalidAssetId(id.as_str().to_owned()))
    }
}

fn verify_path(path: &Path, expected_id: &AssetId, expected_length: u64) -> Result<(), AssetError> {
    let mut source = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut byte_length = 0_u64;
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        byte_length = byte_length
            .checked_add(u64::try_from(read).map_err(|_| AssetError::ByteLengthOverflow)?)
            .ok_or(AssetError::ByteLengthOverflow)?;
        hasher.update(&buffer[..read]);
    }
    let actual_digest = hex_digest(hasher.finalize().as_slice());
    let expected_digest = digest_from_id(expected_id)?;
    if byte_length != expected_length || actual_digest != expected_digest {
        return Err(AssetError::CorruptAsset { id: expected_id.clone(), actual_digest });
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    hex_digest(Sha256::digest(bytes).as_slice())
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

fn clean_abandoned_staging(staging: &Path) -> Result<usize, AssetError> {
    let mut removed = 0;
    for entry in fs::read_dir(staging)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if name.starts_with(".ingest-")
            && name.ends_with(".partial")
            && entry.file_type()?.is_file()
        {
            fs::remove_file(entry.path())?;
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), AssetError> {
    File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), AssetError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        io::{self, Cursor},
        path::PathBuf,
    };

    use super::*;

    struct TestStore {
        _directory: tempfile::TempDir,
        base: PathBuf,
    }

    impl TestStore {
        fn new() -> Self {
            let directory = tempfile::tempdir().expect("temporary directory");
            let base = directory.path().to_path_buf();
            Self { _directory: directory, base }
        }

        fn open(&self, scope: &str) -> LocalAssetStore {
            LocalAssetStore::open(&self.base, scope).expect("open store")
        }
    }

    struct FailsAfterPrefix {
        remaining: Cursor<Vec<u8>>,
        fail: bool,
    }

    impl Read for FailsAfterPrefix {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.fail {
                return Err(io::Error::other("injected interrupted ingest"));
            }
            let read = self.remaining.read(output)?;
            if self.remaining.position() == self.remaining.get_ref().len() as u64 {
                self.fail = true;
            }
            Ok(read)
        }
    }

    #[test]
    fn same_bytes_are_idempotent_within_scope_and_changed_bytes_get_new_ids() {
        let test_store = TestStore::new();
        let store = test_store.open("tenant-a");
        let mut first_bytes = Cursor::new(b"synthetic image".to_vec());
        let first = store
            .ingest(&mut first_bytes, AssetMetadata::new("image", "test-fixture"))
            .expect("ingest first copy");
        let mut same_bytes = Cursor::new(b"synthetic image".to_vec());
        let same = store
            .ingest(&mut same_bytes, AssetMetadata::new("image", "second-reference"))
            .expect("ingest identical copy");
        let mut changed_bytes = Cursor::new(b"different image".to_vec());
        let changed = store
            .ingest(&mut changed_bytes, AssetMetadata::new("image", "test-fixture"))
            .expect("ingest changed bytes");

        assert_eq!(first.id, same.id);
        assert_ne!(first.id, changed.id);
        assert_eq!(first.provenance, "test-fixture");
        assert!(store.verify(&first.id).is_ok());
    }

    #[test]
    fn failed_stream_leaves_no_published_asset_or_staging_file() {
        let test_store = TestStore::new();
        let store = test_store.open("tenant-a");
        let mut source =
            FailsAfterPrefix { remaining: Cursor::new(b"partial content".to_vec()), fail: false };
        assert!(store.ingest(&mut source, AssetMetadata::new("image", "test")).is_err());
        let staging = store.root.join("staging");
        assert_eq!(fs::read_dir(staging).expect("staging directory").count(), 0);
        assert_eq!(fs::read_dir(store.root.join("objects")).expect("objects directory").count(), 0);
    }

    #[test]
    fn corruption_is_reported_and_copy_is_blocked() {
        let test_store = TestStore::new();
        let store = test_store.open("tenant-a");
        let mut bytes = Cursor::new(b"original".to_vec());
        let asset = store.ingest(&mut bytes, AssetMetadata::new("image", "test")).expect("ingest");
        let path = store.object_path(digest_from_id(&asset.id).expect("valid id"));
        fs::write(&path, b"corrupted").expect("corrupt test object");
        assert!(matches!(store.verify(&asset.id), Err(AssetError::CorruptAsset { .. })));
        let mut output = Vec::new();
        assert!(store.copy_verified_to(&asset.id, &mut output).is_err());
        assert!(output.is_empty());
    }

    #[test]
    fn derived_assets_record_source_and_transform_and_are_tenant_scoped() {
        let test_store = TestStore::new();
        let tenant_a = test_store.open("tenant-a");
        let tenant_b = test_store.open("tenant-b");
        let mut derived_bytes = Cursor::new(b"derived preview".to_vec());
        let metadata = AssetMetadata::derived(
            "image/preview",
            "renderer-v1",
            "revision-17",
            serde_json::json!({"scale": 0.25}),
        );
        let asset = tenant_a.ingest(&mut derived_bytes, metadata).expect("ingest derived");
        assert_eq!(asset.source_revision.as_deref(), Some("revision-17"));
        assert_eq!(asset.transform, Some(serde_json::json!({"scale": 0.25})));
        assert!(matches!(tenant_b.verify(&asset.id), Err(AssetError::MissingAsset(_))));
    }

    #[test]
    fn staging_cleanup_removes_only_owned_abandoned_files() {
        let test_store = TestStore::new();
        let store = test_store.open("tenant-a");
        let staging = store.root.join("staging");
        fs::write(staging.join(".ingest-orphan.partial"), b"partial").expect("write orphan");
        fs::write(staging.join("keep.txt"), b"unrelated").expect("write unrelated file");
        drop(store);

        let reopened = test_store.open("tenant-a");
        assert_eq!(reopened.cleanup_abandoned_staging().expect("cleanup"), 1);
        assert!(!staging.join(".ingest-orphan.partial").exists());
        assert!(staging.join("keep.txt").exists());
        assert!(reopened.root.exists());
    }
}
