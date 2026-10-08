# G12 test and compatibility harness evidence

- Date: 2026-10-08
- Status: partial; G12 remains planned and is not accepted.
- Related contract: graph schema v1 in [`project-v1.schema.json`](../../schemas/project-v1.schema.json).
- Related goal: [G01](../../goals/00-foundation/G01-project-graph-schema.md).

## Implemented and exercised

- Added pinned test-only `jsonschema` 0.58.6 with default features disabled. The root workspace test tier now executes the graph's Draft 2020-12 schema rather than only checking selected schema fields.
- Added a synthetic [project-membership fixture](../../tests/fixtures/project-membership-v1/README.md) and proposed record schema. Tests validate the active record and reject invalid identities, embedded capability grants, and unsupported invitation status; membership persistence remains outside this schema harness.
- The integration tests validate the checked-in synthetic campaign fixture, reject malformed UUIDs and unknown project-envelope fields, and validate a generated 2,052-node graph.
- Unknown node fields remain schema-compatible and are verified through runtime parse/serialize round-trip. The fixture is synthetic and its asset record explicitly marks synthetic, test-only rights; it contains no imported media bytes.
- Added a [4×4 ASCII PPM checkerboard](../../tests/fixtures/synthetic-media/checkerboard-4x4.ppm), a [100 ms synthetic PCM WAV tone](../../tests/fixtures/synthetic-media/synthetic-tone-1khz-8khz-mono.wav), and a [four-frame 16×16 YUV4MPEG2 video fixture](../../tests/fixtures/synthetic-media/synthetic-gray-16x16-4frames-24fps.y4m), with a [fixture README](../../tests/fixtures/synthetic-media/README.md) documenting synthetic provenance and test-only use. Integration checks validate PPM structure and decode every pixel with the test-only PNM codec; they also verify WAV container/sample invariants and the Y4M stream header, frame boundaries, plane sizes, and exact luma/chroma samples.
- Added pinned test-only `image` 0.25.9 with default features disabled and only its PNM feature enabled. This exercises real P3 PPM decoding without adding an image codec to the runtime or enabling unrelated formats.
- Added a dependency-free WAV generator that refuses to overwrite its output. Regular CI compiles it, generates into a temporary path, and byte-compares the result with the checked-in fixture to detect nondeterminism or drift.
- Added a dependency-free Y4M generator that refuses to overwrite its output. Regular CI compiles it, regenerates the video fixture, and byte-compares it for deterministic header, frame timing, and pixel-plane data.
- Added a macOS AVFoundation decode check for the WAV fixture. It verifies the decoded rate, channels, frame count, and every PCM sample; the scheduled/manual extended workflow runs this system-native decoder check without adding a codec dependency to the product.
- Added a [four-operation replay fixture](../../tests/fixtures/replay-v1/README.md) and expected final graph. Its integration harness validates every envelope against command schema v1, commits the causal sequence through the authorized local command boundary using test-only project grants, replays from the base graph twice, and compares both results to the expected graph.
- Added a [pixel-exact CPU raster golden](../../tests/fixtures/golden-render/README.md) in the isolated Skia renderer experiment. Its local macOS test compares every RGB and alpha pixel with coordinate-specific diagnostics; the scheduled/manual extended workflow adds a Linux execution, still pending a hosted run.
- The existing root CI `cargo test --workspace --all-targets --locked` includes these tests without adding the native renderer experiment to the workspace.
- Added a separate weekly/manual GitHub Actions workflow for `cargo test --workspace --all-targets --release --locked`; the push/PR workflow remains the fast debug test tier.
- Added an isolated [coverage-guided fuzz workspace](../../fuzz/README.md) with graph and newline-delimited command-envelope targets. Both use checked-in synthetic fixtures as seed inputs; a separate bounded, weekly/manual Linux workflow keeps sanitizer-based fuzzing out of the fast local tier.
- Pinned the fuzz-only `cargo-fuzz`, `libfuzzer-sys`, and nightly toolchain versions. The root fast-CI setup now explicitly installs `rustfmt` and `clippy`, which its format/lint steps invoke.

## Checks run

```text
RUSTUP_TOOLCHAIN=stable cargo fmt --all -- --check
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test --workspace --all-targets --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test -p platform-runtime --test project_schema --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test -p platform-runtime --test fixture_media --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test -p platform-runtime --test operation_replay --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo build --workspace --locked --offline
CARGO_NET_OFFLINE=true RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target ./scripts/smoke-test.sh
file tests/fixtures/synthetic-media/checkerboard-4x4.ppm
rustfmt +stable --edition 2024 --check scripts/generate-synthetic-wav.rs crates/platform-runtime/tests/fixture_media.rs
verify_dir=$(mktemp -d /private/tmp/chromakopia-wav.XXXXXX)
rustc +stable --edition=2024 -D warnings scripts/generate-synthetic-wav.rs -o "$verify_dir/generator"
"$verify_dir/generator" "$verify_dir/generated.wav"
cmp "$verify_dir/generated.wav" tests/fixtures/synthetic-media/synthetic-tone-1khz-8khz-mono.wav
file tests/fixtures/synthetic-media/synthetic-tone-1khz-8khz-mono.wav
rustfmt +stable --edition 2024 --check scripts/generate-synthetic-y4m.rs
y4m_dir=$(mktemp -d /private/tmp/chromakopia-y4m.XXXXXX)
rustc +stable --edition=2024 -D warnings scripts/generate-synthetic-y4m.rs -o "$y4m_dir/generator"
"$y4m_dir/generator" "$y4m_dir/generated.y4m"
cmp "$y4m_dir/generated.y4m" tests/fixtures/synthetic-media/synthetic-gray-16x16-4frames-24fps.y4m
cache_dir=$(mktemp -d /private/tmp/chromakopia-swift.XXXXXX)
swift -module-cache-path "$cache_dir/modules" scripts/verify-synthetic-wav.swift tests/fixtures/synthetic-media/synthetic-tone-1khz-8khz-mono.wav
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test -p platform-runtime --test fixture_media --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/Users/josiah/chromakopia/target cargo test --workspace --all-targets --release --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-fuzz-cargo cargo metadata --manifest-path fuzz/Cargo.toml --locked --offline --no-deps --format-version 1
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-fuzz-cargo cargo fmt --manifest-path fuzz/Cargo.toml --all -- --check
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-renderer-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-renderer-target cargo clippy --manifest-path experiments/renderer-spike/Cargo.toml --all-targets --all-features --locked -- -D warnings
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-renderer-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-renderer-target cargo test --manifest-path experiments/renderer-spike/Cargo.toml --all-features --locked
ruby -e 'require "yaml"; ARGV.each { |path| YAML.load_file(path) }' .github/workflows/ci.yml .github/workflows/extended.yml .github/workflows/fuzz.yml
```

Formatting, strict Clippy, build, and smoke checks passed. Debug and release-profile workspace runs each passed 50 unit tests and 11 integration tests; the focused graph-schema, membership-schema, media-fixture, and replay suites passed 4/4, 2/2, 4/4, and 1/1 respectively. Both media generators compiled with warnings denied and reproduced their checked-in WAV and Y4M fixtures byte-for-byte. `file` recognized the WAV as mono 8 kHz 16-bit PCM. The native AVFoundation check decoded the WAV and matched all 800 PCM samples. The PPM fixture decodes through `image`'s PNM codec and every pixel matches its expected RGB value. The Y4M fixture is structurally validated but has not been exercised through a video decoder. The GitHub Actions workflows have not run because this repository has no remote.

The fuzz source formatting, isolated workspace metadata/lockfile, and workflow YAML syntax checks pass. The fuzz targets have not yet been compiled or run: this macOS host has no nightly toolchain or `cargo-fuzz`, and `libfuzzer-sys` requires Linux sanitizer support. The Linux workflow is configured to compile and run each target for up to 60 seconds, seeded from the corresponding synthetic fixture; its first hosted execution remains required evidence.

## Open acceptance work

- Add versioned historical schema fixtures and migration compatibility tests when a second graph schema version is proposed; v1 is currently the only supported version.
- Exercise Y4M through an approved video decoder; execute the renderer golden on Linux and define a broader cross-platform tolerance policy; execute and grow the parser/contract fuzz campaigns, preserving minimized regressions.
- Run the scheduled/manual extended workflow on GitHub after a remote is configured; local release-profile tests pass, but hosted scheduling remains unverified.
- Run this workflow on a clean hosted default-branch checkout; the repository currently has no commit or remote.

These gaps prevent a G12 completion claim.
