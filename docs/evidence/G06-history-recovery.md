# G06 implementation evidence

- Date: 2026-10-08
- Status: partial; G06 remains planned and is not accepted.
- Environment: Rust/Cargo 1.96.0, stable toolchain, offline dependency resolution.

## Implemented and exercised

- Full-graph snapshots record format, graph-schema, runtime, project, and cursor metadata and a SHA-256 integrity digest.
- Named versions are immutable project/name references; resolution returns the checked graph and its exact asset references.
- Journal schema v1 migrates transactionally to v2 without dropping existing operations.
- A local command commits its operation and checksummed materialized graph in one SQLite transaction. Recovery validates graph, cursor, and asset integrity; on failure it tries snapshots newest-first, replays later operations, and repairs materialized state only after successful reconstruction.
- History is append-only in this implementation. There is no compaction or asset-pruning API.
- Tests exercise v1 migration, stale-base and causal-order rejection, transaction rollback, missing-asset rejection, corrupted materialization, fallback from a corrupted snapshot cursor, replay, named-version resolution, and forced process termination after the local command API returns but before remote acknowledgment.

## Portable bundle implementation slice

The v1 manifest schema and directory layout are implemented by the `portable` runtime module. Export validates snapshots, writes the current graph and ordered operation envelopes, includes named-version references, closes over graph/snapshot assets and asset-ID-shaped command values, re-hashes copied blobs, and publishes without overwriting an existing destination. `verify_project_bundle` rejects malformed layouts, symlinks, extra files, size overages, digest errors, inconsistent references/cursors, and history that does not replay to the current graph. `import_project_bundle` verifies before mutation, ingests immutable blobs, then commits operation/snapshot/version/materialized rows in one SQLite transaction; tests cover replay after import, duplicate-project refusal, validation without side effects, and database rollback. Imported operations are locally pending; no access grants are imported.

Default resource limits are implemented but still need product/security review. Import can leave orphaned immutable blobs if a later asset or database write fails, and it requires the source directory to remain unchanged during validation/import. Export requires project writes to remain quiescent because reads do not share one SQLite snapshot, and recovery may repair materialized state. The actor-ID privacy policy still needs review and disclosure. These prototypes do not complete G06 or establish production portability.

## Checks run

All commands below passed:

```text
RUSTUP_TOOLCHAIN=stable cargo fmt --all -- --check
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo build --workspace --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test --workspace --all-targets --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test --workspace --all-targets --release --locked --offline
CARGO_NET_OFFLINE=true RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target ./scripts/smoke-test.sh
jq empty schemas/capabilities-v1.schema.json schemas/command-envelope-v1.schema.json schemas/history-v1.schema.json schemas/project-v1.schema.json schemas/project-membership-v1.schema.json schemas/project-manifest-v1.schema.json tests/fixtures/synthetic-campaign/project.json tests/fixtures/project-membership-v1/active-user.json
```

The workspace test command passed 50 unit tests and 11 integration tests with 0 failures, including ten portable-bundle tests. The portable schema test validates the actual exporter-produced manifest against Draft 2020-12 and confirms the schema rejects membership inclusion and unknown top-level fields. The crash/reopen test kills its child process after `CommandService::commit_local` returns, then verifies recovery of the materialized graph, the pending operation, and idempotent retry. The current smoke script ran two selected fixture/command tests; both passed. `jq empty` confirms the checked-in schemas are syntactically valid; broader adversarial/fuzz validation remains open.

## Open acceptance work

- Add supported graph-schema migrations and historical graph fixtures, then verify replay across those migrations.
- Add fuzz/property tests for hostile bundles and cross-platform verification of path and resource limits.
- Decide the operation actor-ID privacy/disclosure policy and review default import limits, source-directory race exposure, orphan-asset cleanup, and portable format compatibility before release.
- Extend process-kill coverage to in-flight transaction boundaries and supported release platforms; add abrupt-power-loss, WAL-aware backup/restore, and asset restore tests; decide local encryption and production retention policy.
- ADR-003 remains proposed. These gaps prevent a production durability or G06 completion claim.
