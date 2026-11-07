#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# All four checks; stops at the first failure. Run before every commit.
set -euo pipefail
cd "$(dirname "$0")"
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
echo "check.sh: all OK"
