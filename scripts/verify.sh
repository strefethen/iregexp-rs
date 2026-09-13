#!/bin/sh
set -eu
cd "$(dirname "$0")/.."

python3 scripts/check-provenance.py
cargo fmt --check
cargo check --all-targets --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-features --locked
cargo test --release --all-features --locked
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features --locked
cargo +1.85.0 test --all-features --locked
cargo package --locked
cargo tree --locked
wc -l build.rs src/*.rs tests/*.rs
