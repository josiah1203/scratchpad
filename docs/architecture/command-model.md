# Typed command model v1

Every accepted project state change uses a `CommandEnvelope`. Human, AI, plugin, and service clients share this path; none may mutate canonical state through direct UI writes, private database paths, or unvalidated SQL.

## Envelope

The machine-readable contract is [`schemas/command-envelope-v1.schema.json`](../../schemas/command-envelope-v1.schema.json), with runtime validation in `crates/platform-runtime/src/command.rs`.

- `operation_id` is the idempotency key and stable identity of the operation.
- `project_id` and `target_id` bind the operation to project state.
- `schema_version` makes compatibility explicit.
- `actor` records the principal; AI and plugin actors must carry a run ID.
- `causal` records client sequence, Lamport ordering, and an optional parent operation.
- `command` is a typed, versioned command rather than arbitrary mutation data.

## Initial command families

The first contract supports create, property set/clear, move/reorder, delete, and restore. Property and move commands carry the previous state needed to produce a compensating command. Property state distinguishes an absent key from a key whose value is JSON `null`. The operation remains auditable; undo is a new compensating operation, not deletion or rewriting of history.

`OperationLedger` is a small in-memory duplicate-delivery helper. The first graph reducer supports create, property set/clear, deterministic sibling moves, and soft delete/restore, with explicit old-state preconditions where required. A failed application leaves the original graph unchanged.

`CommandService::commit_local` runs the capability/policy check, stages the graph change, atomically appends the operation and checksummed materialized graph, and exposes the in-memory graph only after commit. Identical retries return an already-committed receipt; a reused operation ID with a different payload is rejected. `CommandService::replay_from` applies deterministic journal pages to a supplied base/snapshot and swaps the graph only if the entire replay succeeds. `LocalJournal::recover_project` validates the materialized view and assets, then falls back to the newest valid snapshot and replays later operations when needed. Snapshot and named-version behavior is documented in [`local-journal-and-assets.md`](local-journal-and-assets.md); production recovery evidence and portable-history migration remain open.

## Boundaries

Validation checks envelope identity, attribution, ordering metadata, target agreement, and command-local invariants. Authorization and local journal sequencing are implemented in the first G07/G04 slices. Mutation conflict handling is limited to reducer preconditions; collaborative merge policy, sync, and schema migration remain future work.
