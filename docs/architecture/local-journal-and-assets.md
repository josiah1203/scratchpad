# Local journal and asset store

## Journal

`LocalJournal` in `crates/platform-runtime/src/journal.rs` provides append, read-by-ID, deterministic project paging, remote-ack state, and health queries. Appending validates a `CommandEnvelope` and commits its payload in SQLite before returning. The current prototype configures WAL and `synchronous=FULL`; `docs/adr/ADR-002-local-sqlite-journal.md` records this as proposed pending broader fault evidence.

`ReplicationState::Pending` means the operation is locally committed and has not been acknowledged remotely. Remote acknowledgment is an explicit separate transition. `CommandService` composes authorization, staged graph application, durable append, and in-memory graph publication; `replay_from` rebuilds from a caller-supplied base graph and cursor. This does not yet provide cloud sync, crash-safe startup orchestration, cross-version graph migration, or compaction.

## History and recovery

`LocalJournal::create_snapshot` stores a full graph checkpoint with snapshot-format, graph-schema, and runtime versions, a logical journal cursor, and a SHA-256 record checksum. Checkpointing validates every referenced asset. `create_named_version` creates an immutable project/name reference to a snapshot; `resolve_named_version` returns the exact graph and asset references after integrity checks. `recover_project` verifies the materialized graph checksum and cursor, tries snapshots newest-first when necessary, replays later operations, re-verifies assets, and repairs the materialized view only after reconstruction succeeds. Its report identifies the chosen source and skipped snapshots.

Create an initial checkpoint for imported/base project state before relying on history-based repair. No compaction API exists: operations, snapshots, named references, and source assets are retained. A v1 portable bundle exporter, verifier, and importer now exist as runtime prototypes, but graph-schema migration and production crash/power-loss/restore evidence remain open; see proposed [ADR-003](../adr/ADR-003-history-snapshot-recovery.md) and [G06 evidence](../evidence/G06-history-recovery.md).

## Assets

`LocalAssetStore` streams bytes into a temporary file, computes SHA-256, syncs the file, and atomically places it under a tenant/authorization-scope directory. Identical bytes reuse the local object within that scope. Another scope has an independent object path, so the local store does not perform cross-tenant deduplication. IDs use lowercase hexadecimal SHA-256 digests.

The graph carries asset identity, kind, length, provenance, rights, and optional derivation source/transform metadata. Source bytes remain immutable through this API. Integrity checks hash the stored object before it is copied out. Abandoned staging files use a private filename pattern and can be removed with an explicit maintenance call.

The caller must derive `scope_id` from authenticated policy and provide a separate local root for each protected device/account boundary as appropriate. Interrupted-ingest cleanup is an explicit maintenance call and requires a quiescent scope with no active writers; store open does not remove staging files. The store does not provide encryption at rest, remote object storage, quotas, or tenant authentication.
