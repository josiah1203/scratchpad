# G11 renderer architecture spike evidence

- Date: 2026-10-08
- Status: partial; G11 remains planned and is not accepted.
- Host: macOS 26.0, x86_64, Intel Iris Plus Graphics, Metal; Rust/Cargo 1.96.0.
- Proposal: [ADR-004](../adr/ADR-004-renderer-and-gpu-boundary.md).
- Prototype: [`experiments/renderer-spike/`](../../experiments/renderer-spike/README.md).

## Implemented and exercised

- An isolated Skia CPU-raster canvas draws rounded vector cards, cubic paths, visible text using a
  platform typeface, and generated image tiles. It renders into caller-owned RGBA pixels and does
  not read or mutate the project graph or command state.
- A `winit` + `softbuffer` window prototype presents the raster output with drag-to-pan and
  wheel-to-zoom input. It entered the macOS event loop without initialization errors; the check
  stopped it after startup, so manual pan/zoom interaction was not verified.
- A separate WGPU compute probe inverts RGB in a synthetic 1024×1024 image. It ran on the Intel Iris
  Plus through Metal and its mapped output matched the CPU reference.
- The WGPU probe now uses a deterministic CPU fallback when adapter discovery or device creation
  fails. `RENDERER_SPIKE_FORCE_CPU=1` provides an explicit fallback test path. Failures after GPU
  initialization, including device loss during a submitted workload, are still reported as errors.
  A normal sandboxed run exercised adapter-unavailable detection and completed in CPU fallback with
  `reference_match=true`; an outside-sandbox run still selected Metal.
- Pan/zoom transforms are shared camera operations with tests for pointer-anchored zoom, pan deltas,
  and zoom limits. Native pointer interaction was not manually driven during this check.
- Added a pixel-exact 8×8 CPU raster golden over integer-aligned vector shapes, with a readable PPM
  fixture and first-differing-coordinate diagnostics. It passes locally on macOS Intel; a scheduled
  Linux run is configured but has not yet executed, so cross-platform determinism remains unproven.
- The experiment pins direct crate versions in its own lockfile and sets `unsafe_code = "forbid"`.
  The root workspace excludes it so the default workspace CI does not acquire a Skia native build.

The platform-specific frame at [`G11-renderer-spike-1280x800-macos-intel.png`](G11-renderer-spike-1280x800-macos-intel.png)
is a visual record of the 2,048-tile Skia scene on this host; it is not a cross-platform golden.

## Measurements

Release-profile measurements on the host above. Skia uses a 1280×800 CPU raster target, 8 warmup
frames, then 40 measured frames. WGPU uses 1 warmup run and 7 measured runs; each timing includes
command submission and mapped readback. The two rows are different operations and must not be read
as a direct Skia-versus-WGPU comparison.

| Path / workload | p50 | p95 | Correctness / notes |
|---|---:|---:|---|
| Skia CPU raster, 256 tiles | 2.784 ms | 4.382 ms | checksum `8ca2a1566a97a273` |
| Skia CPU raster, 2,048 tiles | 5.300 ms | 7.745 ms | same visible viewport and checksum |
| Skia CPU raster, 10,000 tiles | 14.558 ms | 16.918 ms | same visible viewport; currently iterates all scene tiles without culling |
| WGPU Metal, 1024×1024 RGBA8 RGB inversion | 2.117 ms | 2.649 ms | `reference_match=true`; readback included |

Recreate these paths with the commands in the experiment README. The benchmark writes the PNG
fixture to the path passed as its optional argument.

## Checks run

```text
RUSTUP_TOOLCHAIN=stable cargo fmt --manifest-path experiments/renderer-spike/Cargo.toml -- --check
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-renderer-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-renderer-target cargo clippy --manifest-path experiments/renderer-spike/Cargo.toml --all-targets --all-features --locked --offline -- -D warnings
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-renderer-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-renderer-target cargo test --manifest-path experiments/renderer-spike/Cargo.toml --all-features --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-renderer-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-renderer-target cargo test --manifest-path experiments/renderer-spike/Cargo.toml --locked --lib cpu_raster_matches_the_checked_in_pixel_golden
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-renderer-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-renderer-target cargo run --manifest-path experiments/renderer-spike/Cargo.toml --release --bin skia-benchmark -- docs/evidence/G11-renderer-spike-1280x800-macos-intel.png
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-renderer-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-renderer-target cargo run --manifest-path experiments/renderer-spike/Cargo.toml --release --features wgpu-probe --bin wgpu-compute
RENDERER_SPIKE_FORCE_CPU=1 RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-renderer-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-renderer-target cargo run --manifest-path experiments/renderer-spike/Cargo.toml --all-features --locked --offline --bin wgpu-compute
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-renderer-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-renderer-target cargo run --manifest-path experiments/renderer-spike/Cargo.toml --all-features --locked --offline --bin wgpu-compute
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-renderer-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-renderer-target cargo run --manifest-path experiments/renderer-spike/Cargo.toml --all-features --bin skia-canvas
```

Formatting, strict Clippy, and all-feature tests passed; the suite passed 7 library tests and 1 GPU
probe fallback test. The forced CPU fallback ran end-to-end with `reference_match=true`. Both release
benchmarks ran successfully. The WGPU probe and interactive window needed execution outside the
filesystem sandbox to access Metal and the native window server. The Skia source build used Ninja
after the matching prebuilt archive URL returned 404 on this host.

## Open acceptance work

- Only macOS x86_64 was run. Windows, Linux, Apple Silicon, iOS/iPadOS, and Android packaging,
  bindings, runtime, and fallback behavior remain untested. The architecture decision is therefore
  still proposed, not a production platform guarantee.
- The Skia path is CPU-only. No Skia Metal/Vulkan GPU surface was prototyped; the WGPU compute probe
  is independent and does not render the scene. Adapter/device-initialization fallback exists only
  in the probe; GPU device-loss fallback and a product rendering dispatch are not implemented.
- The interactive binary was started but not manually exercised for pan, zoom, resize, or long-run
  presentation. No frame-time pacing or input-latency study was performed.
- The synthetic scene is not produced by the project graph. Scene evaluation, invalidation,
  caching, culling, hit testing, real image decoding/color management, complex text shaping/layout,
  and renderer restart behavior remain unimplemented.
- No cold-machine packaging, binary-size, transitive license/notice, or clean-room build review was
  completed. The current build may compile Skia from source and needs Ninja plus network access.
- Cross-platform deterministic golden-image policy and representative real-project fixtures remain
  open. The current vector-only golden has passed on macOS Intel; Linux execution is configured in
  the extended workflow, but no second-platform result exists yet.

These gaps prevent a G11 completion or production renderer-selection claim.
