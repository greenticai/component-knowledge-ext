#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "=== fmt ==="
cargo fmt --all -- --check

echo "=== clippy ==="
cargo clippy --all-targets --locked -- -D warnings

echo "=== test ==="
cargo test --locked

echo "=== build extension wasm ==="
cargo component build --release --target wasm32-wasip2

echo "=== All checks passed ==="
