# G04 local-journal evidence

- Date: 2026-10-08
- Status: partial; G04 remains planned and is not accepted.
- Related decision: proposed [ADR-002](../adr/ADR-002-local-sqlite-journal.md).

## Implemented and exercised

- `LocalJournal` uses SQLite WAL mode with `synchronous=FULL` and returns local append success only after the transaction commits. Remote acknowledgment is stored independently from local durability.
- A subprocess regression test force-terminates a child after the authorized `CommandService::commit_local` call returns and before any remote acknowledgment. Reopening recovers the checksummed materialized graph and exact operation as `Pending`; retrying the same envelope returns `AlreadyCommitted`.
- Existing tests cover configured journal health, transaction rollback, duplicate delivery, sequence/operation collisions, schema migration, database-growth failure, and corrupt-file diagnostics.

## Checks run

```text
RUSTUP_TOOLCHAIN=stable cargo fmt --all -- --check
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test -p platform-runtime command::tests::killed_process_recovers_local_command_graph_and_pending_operation --locked --offline -- --exact --nocapture
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test --workspace --all-targets --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test --workspace --all-targets --release --locked --offline
CARGO_NET_OFFLINE=true RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target ./scripts/smoke-test.sh
```

All listed checks passed. The focused forced-termination test covers process termination after the local command API returns, then verifies recovery of the graph and pending operation; it does not simulate a kill inside the SQLite transaction or abrupt power loss.

## Open acceptance work

- Exercise pre-commit and in-flight transaction termination, WAL-aware backup/restore, and asset restore on supported platforms.
- Decide encrypted local storage and measure latency under representative operation sizes before accepting ADR-002 or making production durability claims.
- Cloud replication and recovery of sync state remain future G08/G14 work.
