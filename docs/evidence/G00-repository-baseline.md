# G00 repository-baseline evidence

- Date: 2026-10-08
- Status: partial; G00 remains in progress and is not accepted.
- Toolchain: Rust/Cargo 1.96.0, matching `rust-toolchain.toml`.

## Implemented and exercised

- The repository documents prerequisites, common commands, ownership, dependency policy, architecture, and a synthetic project fixture.
- GitHub Actions is configured for pushes to any branch and pull requests. Its workflow runs formatting, all-features Clippy, build, and workspace tests with Rust 1.96.0.
- The pinned toolchain was confirmed locally as Rust/Cargo 1.96.0. An empty Cargo target directory was used for a fresh offline workspace compile using the locally cached, locked dependencies.
- The regular CI workflow's additional steps were exercised locally: fuzz-workspace metadata/format checks and deterministic WAV/Y4M generation matched their checked-in fixtures byte-for-byte.
- Temporary local probes verified that `cargo fmt --check` rejects malformed formatting and the workspace test command fails on a failing test. Both probes were removed before final verification.

## Checks run

An isolated source copy was created without Git metadata or the existing build directory, and the following local CI-equivalent commands passed from that copy:

```text
rsync -a --exclude='/.git/' --exclude='/target/' /Users/josiah/chromakopia/ /private/tmp/chromakopia-g00-checkout.EWjcKi/
RUSTUP_TOOLCHAIN=stable cargo fmt --all -- --check
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-g00-checkout.EWjcKi-target cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-g00-checkout.EWjcKi-target cargo build --workspace --locked --offline
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-g00-checkout.EWjcKi-target cargo test --workspace --all-targets --locked --offline
CARGO_NET_OFFLINE=true RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo CARGO_TARGET_DIR=/private/tmp/chromakopia-g00-checkout.EWjcKi-target ./scripts/smoke-test.sh
RUSTUP_TOOLCHAIN=stable CARGO_HOME=/private/tmp/chromakopia-schema-cargo cargo metadata --manifest-path fuzz/Cargo.toml --locked --offline --no-deps --format-version 1
RUSTUP_TOOLCHAIN=stable cargo fmt --manifest-path fuzz/Cargo.toml --all -- --check
```

The media-generation steps in `.github/workflows/ci.yml` also passed locally:

```text
RUSTUP_TOOLCHAIN=stable rustfmt --edition 2024 --check scripts/generate-synthetic-wav.rs
RUSTUP_TOOLCHAIN=stable rustc --edition=2024 -D warnings scripts/generate-synthetic-wav.rs -o /private/tmp/chromakopia-g00-checkout.EWjcKi/generate-synthetic-wav
/private/tmp/chromakopia-g00-checkout.EWjcKi/generate-synthetic-wav /private/tmp/chromakopia-g00-checkout.EWjcKi/synthetic-tone.wav
cmp /private/tmp/chromakopia-g00-checkout.EWjcKi/synthetic-tone.wav tests/fixtures/synthetic-media/synthetic-tone-1khz-8khz-mono.wav
RUSTUP_TOOLCHAIN=stable rustfmt --edition 2024 --check scripts/generate-synthetic-y4m.rs
RUSTUP_TOOLCHAIN=stable rustc --edition=2024 -D warnings scripts/generate-synthetic-y4m.rs -o /private/tmp/chromakopia-g00-checkout.EWjcKi/generate-synthetic-y4m
/private/tmp/chromakopia-g00-checkout.EWjcKi/generate-synthetic-y4m /private/tmp/chromakopia-g00-checkout.EWjcKi/synthetic-video.y4m
cmp /private/tmp/chromakopia-g00-checkout.EWjcKi/synthetic-video.y4m tests/fixtures/synthetic-media/synthetic-gray-16x16-4frames-24fps.y4m
```

The isolated source-copy run passed 50 unit tests and 11 integration tests with 0 failures. The workspace build, strict all-features Clippy, formatter, smoke script, fuzz metadata/format checks, and media generation/fixture comparisons all passed. The smoke script ran its two named fixture/command tests, both passing. This is a clean source-copy check, not a Git clone or hosted Actions run.

Expected-negative local checks:

- `RUSTUP_TOOLCHAIN=stable cargo fmt --all -- --check` exited nonzero and showed the required formatting diff while a temporary malformed function was present.
- The temporary failing-test probe exited nonzero and showed the intentional failure; it was removed before the passing checks above.

These exercise the same local commands used by CI, but they do not prove that a hosted GitHub Actions run occurred.

## Open acceptance work

- The current `main` branch has no commit and the checkout has no configured Git remote; the repository files are untracked. Therefore a clean Git checkout and hosted default-branch CI run could not be verified, and there is no committed baseline for reviewers to reproduce.
- The pinned toolchain is already installed locally; installation on a fresh machine was not exercised. The locked build used the existing offline dependency cache.
- The sandbox could not write to the user's Rustup home to install the pinned toolchain; the already-installed `stable` toolchain was verified as exactly Rust/Cargo 1.96.0 and used for the clean-target run.
- CI workflow behavior was inspected and its local gates were exercised, but GitHub-hosted scheduling, permissions, and action execution remain unverified.

These gaps prevent a G00 completion claim.
