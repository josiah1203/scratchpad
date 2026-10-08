# Unified Creative Platform

This repository is the implementation starting point for a unified creative platform: one durable project system with specialized Design, Image, Video, and later workspace modules.

The project guide in [`AGENTS.md`](AGENTS.md) is the architectural source of truth. The milestone index is [`goals/README.md`](goals/README.md). No goal is accepted yet; implementation readiness depends on dependencies, evidence, and required ADRs.

## Product and architecture

- [Working product brief](docs/product/brief.md) — initial target, workflow, scope, and validation questions.
- [Architecture overview](docs/architecture/overview.md) — component boundaries, data flow, trust boundaries, failure behavior, and implementation status.
- [Portable project bundle v1](docs/architecture/portable-project-format.md) — manifest, file layout, integrity rules, and current export/verification/import boundaries.
- [Architecture decision records](docs/adr/README.md) and [detailed architecture notes](docs/architecture/).

## Current baseline

G00 establishes a reproducible Rust workspace and a synthetic project fixture. The runtime now contains initial slices of the project graph, typed command reducer, local SQLite journal and replay boundary, scoped content-addressed asset store, and capability policy. A subprocess test force-terminates a process after an authorized local command returns, then verifies recovery of both the materialized graph and pending operation ([G04 evidence](docs/evidence/G04-local-journal.md)); this does not prove in-transaction crash or power-loss behavior. These are foundation prototypes—not complete history, sync, recovery, or production durability implementations.

## Prerequisites

- Rust 1.96.0 with `rustfmt` and `clippy` (the repository pins this in [`rust-toolchain.toml`](rust-toolchain.toml)).
- Git.

Rustup users can install the pinned toolchain with:

```sh
rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy
```

## Common commands

Run these from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --locked
./scripts/smoke-test.sh
```

The smoke test is deliberately small. Schema, media-fixture, and replay integration tests run in the fast suite. Coverage-guided parser fuzzing is isolated in the [fuzz workspace](fuzz/README.md) and scheduled/manual [fuzz workflow](.github/workflows/fuzz.yml). An experimental Skia CPU pixel golden runs in the scheduled/manual [extended verification workflow](.github/workflows/extended.yml); its Linux result is pending. Crash, network-fault, security, and operational checks remain future gates as their foundation goals land.

The fast test tier above runs on pushes and pull requests. The [extended verification workflow](.github/workflows/extended.yml) runs the full workspace test suite in release mode weekly and on manual dispatch; run the same tier locally with:

```sh
cargo test --workspace --all-targets --release --locked
```

## Repository map

```text
crates/                         Shared Rust runtime crates
tests/fixtures/                 Synthetic and licensed test fixtures
docs/adr/                       Architecture decision records and index
docs/product/                   Product brief and scope
docs/architecture/              Architecture overview and component contracts
docs/evidence/                  Goal-specific verification and known limitations
goals/                          Planned milestones and dependency gates
scripts/                        Reproducible local verification helpers
.github/                        CI and repository automation
```

The target architecture described in `AGENTS.md` may grow to include `apps/`, `engines/`, `services/`, `schemas/`, `sdk/`, and `infrastructure/`. Existing code should not be moved merely to match a diagram; each consequential boundary requires evidence and, where noted, an ADR.

## Working agreements

- Read `AGENTS.md`, `goals/README.md`, the selected goal, and its dependency goals before editing.
- Treat the project graph as canonical. Accepted state changes use the shared typed command path.
- Preserve local durability, offline behavior, editability, permissions, history, and asset identity as first-class contracts.
- Keep shared contract ownership with one lead and assign parallel work to disjoint areas.
- Update schemas, fixtures, docs, ADRs, and tests with behavior changes.
- Report exact verification commands, results, limitations, and follow-up decisions.

## Status

The synthetic WAV has a local AVFoundation decode check that verifies every sample, and the scheduled extended workflow repeats that check on macOS. A test-only PNM decoder also verifies every synthetic PPM pixel; Y4M video decoding remains open, and hosted workflows cannot run until a remote is configured.

G12 now also includes a reproducible four-frame YUV4MPEG2 fixture for downstream video-ingest work. Its structure is tested, but it has not yet been run through a real video decoder.

The repository has initial G00/G01/G03/G04/G05/G07 foundation slices plus partial G11 and G12 work. G00's local clean-target, build, test, smoke, and failure-probe evidence is recorded in [`docs/evidence/G00-repository-baseline.md`](docs/evidence/G00-repository-baseline.md); hosted CI and a clean Git checkout remain unverified because this workspace has no commit or remote. The command reducer applies graph preconditions; the local command service authorizes and atomically journals an operation with its materialized graph. G01 has indexed partial dependency loading, generated large-graph regression coverage, and test-only Draft 2020-12 validation for graph and proposed membership schemas; authoritative membership persistence, historical migrations, and invitation/role/audit semantics remain open ([evidence](docs/evidence/G01-project-graph.md)). G06 has checksummed snapshots, immutable named-version references, asset verification, v1-to-v2 journal migration, fallback/replay repair for corrupted materialized state, and a v1 portable-bundle exporter/verifier/importer prototype. Its generated manifest is validated against the published JSON Schema, but graph-schema migrations, bundle fuzzing, privacy/security review, production crash/restore evidence, and a production retention policy remain open. Partial implementation checks and limitations are recorded in [`docs/evidence/G06-history-recovery.md`](docs/evidence/G06-history-recovery.md). G11 has an isolated Skia CPU-raster/window spike, a CPU pixel-golden test, and a separate WGPU Metal compute probe; production renderer choice remains proposed pending platform, fallback, interaction, packaging, and text/layout evidence ([evidence](docs/evidence/G11-renderer-architecture-spike.md), [ADR-004](docs/adr/ADR-004-renderer-and-gpu-boundary.md)). G12 now has semantic v1 schema checks, rights-labeled synthetic PPM and PCM WAV fixtures (with test-only PPM decode coverage), a synthetic vector golden fixture, fixture-driven command replay, and scheduled/manual release, renderer-golden, and Linux fuzz tiers. The WAV generator is deterministic and checked against its fixture in regular CI. The fuzz targets are defined but not executed, and the Linux golden run is pending; hosted execution remains unavailable without a configured remote. Historical migrations, Y4M video decoding, and cross-platform golden policy remain open ([evidence](docs/evidence/G12-test-compatibility-harness.md)). Remaining foundation work includes these gates, cloud-store evaluation, module contracts, sync, and the G14 end-to-end proof before broad editor work.
