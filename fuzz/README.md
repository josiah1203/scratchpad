# Coverage-guided parser fuzzing

This isolated workspace contains `cargo-fuzz` targets for the shared project graph parser/invariants
and the typed command-envelope parser/validator. The command target accepts newline-delimited
envelopes so the checked-in replay sequence can seed it directly. Both targets treat malformed input
as ordinary rejected input; a crash, sanitizer finding, or hang is a fuzz failure.

`libfuzzer-sys` relies on LLVM sanitizer support and this configuration is run on Linux, not the
current macOS development host. Tool versions are pinned: nightly Rust `nightly-2026-06-09`,
`cargo-fuzz` 0.13.2, and `libfuzzer-sys` 0.4.13. The fuzz crate has its own lockfile and is excluded
from the stable root workspace test tier.

On Linux, install the pinned toolchain and runner, then run bounded campaigns from the repository
root:

```sh
rustup toolchain install nightly-2026-06-09 --profile minimal
cargo +nightly-2026-06-09 install cargo-fuzz --version 0.13.2 --locked
cargo +nightly-2026-06-09 fuzz run project_graph -- -max_total_time=60 -max_len=262144 tests/fixtures/synthetic-campaign/project.json
cargo +nightly-2026-06-09 fuzz run command_envelope -- -max_total_time=60 -max_len=262144 tests/fixtures/replay-v1/operations.jsonl
```

Generated build output and crash artifacts are ignored. Keep any minimized crash-inducing input as a
versioned regression fixture under `tests/fixtures/`, with synthetic or licensed provenance, and
add a deterministic regression test before removing it from the fuzz artifacts.
