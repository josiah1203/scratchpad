# Renderer architecture spike

This is an isolated G11 experiment, not a shared-runtime or product renderer. It is excluded from
the root Cargo workspace so ordinary workspace checks do not implicitly build Skia's native
library.

The Skia path rasterizes a synthetic 2D canvas (rounded vector cards, paths, labels, and generated
image tiles) into CPU RGBA pixels. The optional interactive binary presents those pixels through
`winit` and `softbuffer`; drag pans and the wheel zooms. The independent WGPU probe runs a
1024×1024 compute operation and verifies readback against a CPU reference. It is not a WGPU
full-scene renderer and is not a Skia GPU surface. The crate forbids unsafe Rust.

The Skia test suite includes an exact 8×8 CPU-raster comparison against the synthetic
[`skia-vector-8x8.ppm`](../../tests/fixtures/golden-render/skia-vector-8x8.ppm) fixture. It uses
integer-aligned solid rectangles, so it avoids system-font and GPU variation. The scheduled/manual
extended workflow runs this check on Linux as a second platform; until that hosted run succeeds, the
golden is verified only on the current macOS Intel host and is not a cross-platform renderer promise.

## Prerequisites

- Rust 1.96.0 with Cargo, rustfmt, and Clippy.
- A C/C++ toolchain, Python 3, and Ninja for a Skia source build.
- Network access on a cold build if Skia's matching prebuilt archive is unavailable.
- A native windowing environment for the interactive canvas; a supported WGPU adapter for the GPU
  probe.

Skia's build script may download prebuilt archives and falls back to compiling Skia if the exact
platform/feature archive is missing. The first source build can take several minutes and use
substantial temporary storage. Check the pinned `Cargo.lock`; do not update dependencies as part of
benchmarking.

## Run

From the repository root:

```sh
RUSTUP_TOOLCHAIN=stable cargo fmt --manifest-path experiments/renderer-spike/Cargo.toml -- --check
RUSTUP_TOOLCHAIN=stable cargo clippy --manifest-path experiments/renderer-spike/Cargo.toml --all-targets --all-features --locked -- -D warnings
RUSTUP_TOOLCHAIN=stable cargo test --manifest-path experiments/renderer-spike/Cargo.toml --all-features --locked
RUSTUP_TOOLCHAIN=stable cargo run --manifest-path experiments/renderer-spike/Cargo.toml --release --bin skia-benchmark -- /tmp/renderer-spike.png
RUSTUP_TOOLCHAIN=stable cargo run --manifest-path experiments/renderer-spike/Cargo.toml --release --features wgpu-probe --bin wgpu-compute
RENDERER_SPIKE_FORCE_CPU=1 RUSTUP_TOOLCHAIN=stable cargo run --manifest-path experiments/renderer-spike/Cargo.toml --features wgpu-probe --bin wgpu-compute
RUSTUP_TOOLCHAIN=stable cargo run --manifest-path experiments/renderer-spike/Cargo.toml --all-features --bin skia-canvas
```

The benchmark uses the same 1280×800 output and 8 warmup / 40 measured frames for 256, 2,048, and
10,000 tiles. The WGPU timing includes submission and mapped readback. Treat the two timings as
separate workloads, not a head-to-head renderer comparison. The WGPU probe falls back to its CPU
reference when adapter/device initialization is unavailable; set `RENDERER_SPIKE_FORCE_CPU=1` to
exercise that path explicitly. See
[`../../docs/evidence/G11-renderer-architecture-spike.md`](../../docs/evidence/G11-renderer-architecture-spike.md)
for the measured host, results, and open acceptance work.
