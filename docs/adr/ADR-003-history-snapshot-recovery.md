# ADR-003: Snapshot, named-version, and recovery contract

- Status: proposed
- Date: 2026-10-07
- Owners: Runtime/persistence lead
- Related: G04, G05, G06, G08

## Context

The journal and asset store provide durable operations and immutable bytes, but a materialized graph can still be damaged or unavailable. Recovery needs a validated checkpoint, a deterministic operation cursor, exact asset references, and human-readable version references. Compaction must not make the operation history or portable project unrecoverable.

## Decision

- A snapshot is an immutable full project-graph JSON image, with snapshot-format version, graph-schema version, runtime version, project and snapshot IDs, and an optional deterministic journal cursor. A SHA-256 record checksum covers these fields and the graph bytes, including the cursor.
- A named version is an immutable `(project, name) -> snapshot` reference. Names are unique within a project; creating a duplicate name or ID fails visibly. Rename/delete semantics are not implicit.
- The SQLite materialized view stores the graph image, journal cursor, and a checksum over both. A local command append updates the operation and its materialized view in the same transaction. The in-memory graph is published only after that transaction commits.
- Recovery validates the materialized checksum, graph invariants, cursor membership, and every referenced asset's digest and byte length. If invalid, it tries snapshots newest-first, replays later operations in deterministic journal order, verifies assets again, and writes the repaired materialized view only after the full reconstruction succeeds. Invalid snapshots are retained and reported; recovery never removes operations or assets.
- Until a portable export/import format and restore evidence prove a safe pruning boundary, compaction is disabled: retain all operations, snapshots, named-version references, and source assets. Storage pressure must be surfaced as a write failure, not handled by silent history loss.
- A workspace-neutral directory-bundle contract is documented in [`portable-project-format.md`](../architecture/portable-project-format.md) and `schemas/project-manifest-v1.schema.json`. A v1 exporter, verifier, and importer now exist as runtime prototypes. The bundle includes graph/history/version/asset references, excludes memberships and capability grants by default, and preserves operation actor IDs only as opaque provenance. This implementation is not yet an accepted portability or compaction guarantee.

The initial schema migration is SQLite `user_version` 1 to 2 and adds history tables transactionally without rewriting the operation table. Runtime version is provenance, not a hard compatibility gate; unsupported snapshot-format and graph-schema versions fail validation. The current implementation supports graph schema v1 only.

This remains a proposed implementation contract. Production durability still depends on platform-specific process-kill, abrupt-power-loss, WAL backup/restore, encrypted-local-storage, and asset restore evidence recorded as open work in ADR-002.

## Alternatives considered

- A snapshot containing only a cursor would require replaying the entire operation history and would not repair a damaged or unavailable materialized base.
- Updating the materialized view after committing the operation in a separate transaction leaves a crash window where the view claims a state that the journal does not atomically establish.
- Pruning operations once a newer checkpoint exists would break auditability, portable history, and named-version recovery until export/import retention guarantees are implemented and verified.
- Binding a named version directly to a mutable graph row would allow later edits to change what the name resolves to.

## Consequences

Snapshots duplicate graph metadata and add storage proportional to checkpoint frequency. Recovery reads every referenced asset to verify its digest, which may be expensive for large projects; future verification may use trusted immutable verification records only if their integrity and invalidation rules are specified. Named versions cannot currently be renamed or removed. The implementation intentionally has no compaction API.

## Validation and revisit trigger

Tests corrupt a materialized graph and alter a snapshot cursor to a different valid journal position, then verify fallback to an older snapshot, deterministic replay, asset integrity, and preserved operations. A separate migration test upgrades a v1 journal without losing its operation. A v1 bundle exporter/verifier/importer prototype now exists; before accepting this ADR, fault-test portable restore, resolve actor-ID privacy, and complete compatibility/security review. Revisit before enabling compaction, supporting another graph schema, adding branch mutation semantics, or making production recovery claims.
