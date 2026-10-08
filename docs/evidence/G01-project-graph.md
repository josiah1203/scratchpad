# G01 project-graph implementation evidence

- Date: 2026-10-08
- Status: partial; G01 remains planned and is not accepted.
- Contracts: graph schema v1 in `schemas/project-v1.schema.json`; proposed service-owned membership record v1 in `schemas/project-membership-v1.schema.json`; runtime graph validation in `crates/platform-runtime/src/graph.rs`.

## Implemented and exercised

- Project and node identities are UUID-shaped strings; asset IDs are lowercase SHA-256 identifiers.
- Runtime validation checks schema version, project/node identity, duplicate node and asset IDs, non-blank project/node/property labels and asset kind/provenance, node and asset references, and parent cycles.
- A test-only Draft 2020-12 validator executes the published graph schema against the checked-in fixture, invalid identities, unknown top-level fields, forward-compatible node fields, and a 2,052-node generated graph.
- The runtime round-trips unknown node fields; the schema rejects unknown project-envelope fields and invalid UUID-shaped IDs.
- Unsupported top-level data and graph schema versions fail rather than being silently discarded.
- `load_subgraph` returns roots plus parent/reference dependencies in source order. Its dependency walk now uses an ID index and visited set instead of repeatedly scanning all nodes.
- The large-graph regression test builds a valid graph with 2,048 unrelated nodes and confirms a Design-document load returns only the four reachable fixture nodes. A separate test verifies a two-node parent cycle is rejected with both cycle members.
- The current graph v1 has project metadata, nodes, relations, and asset references. A separate membership schema and synthetic fixture now define a current tenant/project/principal relationship without embedding capabilities or roles; authoritative persistence and invitation/role/audit semantics remain open. Named versions remain in a separate SQLite history table and are linked from the v1 portable bundle manifest by the exporter prototype.

## Proposed, not implemented

ADR-001 distinguishes the project aggregate from the content-graph envelope, separates service-authoritative membership from capability grants, and proposes content/history-only export by default. The exporter, verifier, importer, and manifest exist as prototypes; membership persistence, historical graph migrations, and broader compatibility evidence remain open.

## Checks run

```text
RUSTUP_TOOLCHAIN=stable cargo fmt --all -- --check
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test --workspace --all-targets --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test -p platform-runtime --test project_schema --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test -p platform-runtime --test project_membership_schema --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo build --workspace --locked --offline
CARGO_NET_OFFLINE=true RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target ./scripts/smoke-test.sh
jq empty schemas/project-v1.schema.json schemas/project-membership-v1.schema.json tests/fixtures/synthetic-campaign/project.json tests/fixtures/project-membership-v1/active-user.json
```

The workspace run passed 50 unit tests and 11 integration tests, including four graph-schema cases, two membership-schema cases, four media-fixture checks (PPM structure, PPM decoder, WAV, and Y4M), and one command-replay fixture. The focused graph and membership schema runs passed 4/4 and 2/2 tests. The build and both smoke tests passed. `jq empty` confirms JSON syntax only; semantic validation is provided by the schema integration tests.

## Open acceptance work

- Add versioned historical graph fixtures and explicit migration behavior before supporting another graph schema version. The runtime currently supports v1 only; no graph-schema migration framework exists.
- Implement authoritative membership persistence and its migration/audit behavior; define invitation and role-to-capability semantics. Complete portable-format compatibility/security review while preserving the tested content-only import behavior.
- Add fuzz/property coverage and storage-backed paging for very large projects under G12. The 2,048-node case is generated in memory and proves correctness, not a production performance limit.
- ADR-001 remains proposed. These gaps prevent a G01 completion or stable-contract claim.
