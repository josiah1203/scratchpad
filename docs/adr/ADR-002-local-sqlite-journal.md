# ADR-002: Local SQLite journal configuration

- Status: proposed
- Date: 2026-10-07
- Owners: Persistence lead
- Related: G04, G06, G08, G13

## Context

Acknowledged edits must survive application restart and remain available offline. The repository guide names SQLite WAL as the initial candidate, but the production durability claim needs fault-injection evidence on supported filesystems and an explicit encrypted-local-storage decision.

## Decision

For the initial runtime prototype, use SQLite in WAL mode with `synchronous=FULL`, a five-second busy timeout, and a versioned schema. Append validates the typed command, writes in an immediate transaction, commits, then returns local success. The sync acknowledgment flag is separate and cannot be inferred from local commit success.

This is a proposed configuration, not a production durability promise. Local database encryption and recovery after abrupt power loss remain unresolved. The journal health API reports the active mode, synchronous setting, schema version, SQLite version, and quick integrity check.

## Alternatives considered

- Cloud-first persistence would block or lose offline work and would not meet the local acknowledgment boundary.
- A custom append-only file format would add locking, indexing, migration, corruption, and recovery code that SQLite already provides.
- SQLite rollback-journal mode remains a candidate if WAL behavior or platform support fails validation.

## Consequences

The runtime has a local SQLite dependency and a journal schema migration boundary. WAL sidecar files must be kept with the database during backup/copy; supported backup paths need a SQLite-aware snapshot procedure. Filesystem and storage-device behavior still bound durability.

## Validation and revisit trigger

Current checks cover local append/commit ordering, duplicate delivery, operation-ID and client-sequence collisions, database growth limits, injected transaction failure, reopen recovery, WAL/FULL mode, corrupt-file diagnostics, and a subprocess forced-termination test after the authorized local command API returns but before remote acknowledgment. That test verifies the materialized graph and operation recover, the operation remains remotely pending, and an identical retry is idempotent. Before accepting this ADR, broaden process-kill coverage to in-flight transaction boundaries and supported release platforms, add abrupt-power-loss fault tests, exercise WAL-aware backup/restore, decide encrypted storage, and measure write latency under representative operation sizes. Revisit if these tests fail or storage requirements change.
