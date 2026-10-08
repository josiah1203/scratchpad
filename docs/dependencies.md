# Direct dependency record

Versions are pinned exactly in package manifests and their corresponding lockfiles. This records direct dependencies used by the shared runtime, its tests, and the isolated fuzz workspace; test- and fuzz-only dependencies are labeled explicitly. It is not a substitute for a generated transitive inventory, vulnerability scan, or release-specific license review.

| Package | Version / features | License | Use and review notes |
|---|---|---|---|
| `serde` | 1.0.229, `derive` | MIT OR Apache-2.0 | Typed, versioned JSON data structures. No local modifications. |
| `serde_json` | 1.0.151 | MIT OR Apache-2.0 | Portable graph and command encoding. No local modifications. |
| `rusqlite` | 0.32.1, `bundled` | MIT | SQLite access for the local journal. The bundled SQLite amalgamation is public domain; `libsqlite3-sys` (MIT) and build dependencies are included in the lockfile. WAL behavior and backup handling need platform tests. |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 | SHA-256 content identity and integrity checks. No local modifications. |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | Same-filesystem staging and no-clobber asset installation. Temporary contents are removed on failed ingestion. |
| `jsonschema` | 0.58.6, defaults disabled; dev dependency | MIT | Draft 2020-12 validation of checked-in graph schemas and compatibility fixtures. Network/file reference features are disabled; see the [crate documentation](https://docs.rs/jsonschema/0.58.6/jsonschema/). |
| `image` | 0.25.9, defaults disabled, `pnm`; dev dependency | MIT OR Apache-2.0 | Decode the synthetic PPM compatibility fixture through the PNM codec only. No production/runtime dependency and no other image codecs enabled. |
| `libfuzzer-sys` | 0.4.13; isolated fuzz workspace only | MIT OR Apache-2.0 | Coverage-guided parser fuzzing for project graphs and command envelopes. Depends on LLVM sanitizer support; the configured runner is Linux-only. |

The fuzz workflow installs `cargo-fuzz` 0.13.2 (MIT OR Apache-2.0) as a pinned development tool
and uses nightly Rust `nightly-2026-06-09`; neither is a runtime dependency. See the
[fuzz workspace guide](../fuzz/README.md).

Before shipping binaries, generate and review the complete transitive license inventory, check advisories and update cadence, record build features and notices, and repeat the review when versions or features change. Media codecs, fonts, PDF support, and graphics libraries require their own feature/configuration and distribution review before adoption.

## Experimental renderer spike

These exact versions are pinned only in `experiments/renderer-spike/Cargo.toml` and its separate
`Cargo.lock`. The experiment is excluded from the root workspace and is not a production dependency.

| Package | Version / features | Declared license | Experimental use / open review |
|---|---|---|---|
| `skia-safe` | 0.153.3; `binary-cache`, `embed-icudtl`, `textlayout`; defaults disabled | MIT | CPU raster, system font matching, text drawing, and image/path APIs. Skia native archive/build, bundled notices, font behavior, and distribution size require review. |
| `wgpu` | 30.0.1; optional | MIT OR Apache-2.0 | Separate compute-only Metal probe; no scene renderer. GPU backend/driver and fallback matrix remain unvalidated. |
| `winit` | 0.30.13; optional | Apache-2.0 | Native interactive-window event loop. Platform packaging/bindings remain unvalidated. |
| `softbuffer` | 0.4.8; optional | MIT OR Apache-2.0 | CPU pixel presentation for the Skia raster prototype. |
| `pollster` | 0.4.0; optional | Apache-2.0 OR MIT | Small synchronous executor for the probe binary only. |

This is a direct-dependency record, not a full native/transitive license inventory. The Skia source
fallback requires a C/C++ toolchain, Python 3, Ninja, and network access on a cold build; confirm
prebuilt archive availability, build reproducibility, notices, and target support before adopting
this stack in a shipped binary. See the [G11 evidence](evidence/G11-renderer-architecture-spike.md)
and proposed [ADR-004](adr/ADR-004-renderer-and-gpu-boundary.md).
