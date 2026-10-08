#!/usr/bin/env bash
set -euo pipefail

cargo test --package platform-runtime --lib tests::sample_fixture_is_a_valid_graph --locked
cargo test --package platform-runtime --lib command::tests::local_command_boundary_commits_then_replays_and_deduplicates --locked
