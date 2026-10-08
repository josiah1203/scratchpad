# ADR-004: Initial 2D renderer and GPU boundary

- Status: proposed
- Date: 2026-10-08
- Owners: Graphics/runtime lead
- Related: G11, G12, [architecture overview](../architecture/overview.md), [G11 evidence](../evidence/G11-renderer-architecture-spike.md)

## Context

The platform needs a high-volume 2D canvas without making the project graph, command reducer, or
durable journal depend on a client rendering API. G11 calls for evidence across vector, text, image,
zoom/pan, large scenes, GPU work, fallbacks, packaging, and target platforms. The current spike
isolates a Skia CPU raster path and a separate WGPU compute operation. It has only been run on one
Intel macOS machine, so it is not enough evidence for a production commitment.

## Decision

**Proposed for the next prototype stage:** use Skia as the first 2D raster API and mandatory CPU
fallback; keep WGPU available only for measured, workload-specific GPU operations. Do not build two
competing full-scene renderers. This is a provisional recommendation, not an accepted production
selection.

- The project graph and typed command runtime remain canonical. They produce read-only render
  inputs; renderer state never owns or mutates project semantics.
- Workspace/runtime code owns scene evaluation and invalidation contracts. Skia owns rasterization
  and its text drawing/shaping primitives; a complete typography/layout contract remains future
  work.
- The current baseline is Skia CPU raster. A WGPU operation is optional and must preserve a
  correctness-equivalent CPU path for supported workloads. The probe falls back to CPU when adapter
  discovery or device creation fails; this is not yet a renderer dispatch path, and post-initialization
  device loss is still surfaced as an error. No user-facing project can currently be made unreadable
  by that probe because it is isolated from project rendering.
- Do not enable a Skia GPU surface or unsafe platform-handle bridge until the safe binding and
  window/surface ownership boundary is explicitly reviewed. The spike crate forbids unsafe Rust.
- Add scene culling, retained state, or caches only with measurable invalidation rules. The 10,000
  tile benchmark currently traverses all tiles on every frame and does not model graph evaluation.

Accepting this ADR requires additional target-platform, packaging, fallback, text/layout, and
interaction evidence listed in the G11 evidence report.

## Alternatives considered

- **WGPU as the only full-scene renderer:** would provide one explicit GPU API, but this spike only
  measured a compute image operation; it did not establish vector path quality, font shaping,
  accessibility, or a reliable CPU rendering fallback.
- **Skia GPU rendering as the initial path:** Skia supports native GPU backends, but this spike did
  not validate a safe window/surface integration or cross-platform GPU lifecycle. The Rust Skia
  project's documented platform and prebuilt-binary matrix still requires per-target packaging
  checks ([rust-skia project](https://github.com/rust-skia/rust-skia)).
- **Skia CPU raster only:** simplest deployable baseline and exercised in the prototype, but the
  10,000-tile result shows the need to measure scene culling and representative workloads before
  accepting it as the only long-term rendering path.
- **Two full-scene backends:** rejected for the prototype because it doubles rendering semantics,
  golden coverage, and fallback complexity without evidence of a user-facing benefit.

## Consequences

The prototype has a real CPU raster fallback and a separate Metal compute probe. On the tested Mac,
Skia release p50 was 5.300 ms for 2,048 tiles and 14.558 ms for 10,000 tiles at 1280×800; WGPU RGB
inversion was 2.117 ms p50 for 1024×1024 including readback. These workloads are not comparable and
do not justify a GPU/full-scene choice. See [G11 evidence](../evidence/G11-renderer-architecture-spike.md).

The current Skia build selected a source-build fallback on the tested host, requiring Ninja and
network access; binary distribution size, licenses/notices, and cold-build behavior remain open.
The pinned `skia-safe` crate uses its documented text-layout feature set; platform font availability
must be tested beyond this host ([feature documentation](https://github.com/rust-skia/rust-skia/blob/master/skia-safe/README.md)).

## Validation and revisit trigger

Before accepting this ADR, run the same vector/text/image and interaction fixtures on macOS Intel and
Apple Silicon, Windows, Linux, and the intended mobile targets; test clean packaging and dependency
notices; exercise GPU-present and no-GPU/loss fallback; manually verify pan, zoom, resize, and input
latency; and define renderer invalidation plus cross-platform golden policy. Compare cold and warm
performance on representative project scenes, including culling. Revisit if these results show a
different backend materially improves correctness, supported platforms, or user-visible latency.
