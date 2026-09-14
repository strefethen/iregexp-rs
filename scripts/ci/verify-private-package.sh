#!/usr/bin/env bash
# Fails unless Cargo.toml keeps publish = false. iregexp-rs is distributed
# from GitHub releases only and is never published to crates.io.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if ! grep -qE '^publish[[:space:]]*=[[:space:]]*false[[:space:]]*$' "$repo_root/Cargo.toml"; then
  echo "ERROR: Cargo.toml must set publish = false." >&2
  echo "This crate is distributed from GitHub and is never published to crates.io." >&2
  exit 1
fi

echo "Cargo.toml publish = false verified."
