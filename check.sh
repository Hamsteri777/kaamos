#!/usr/bin/env bash
# All four checks; stops at the first failure. Run before every commit.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
echo "check.sh: all OK"
