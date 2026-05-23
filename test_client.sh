#!/usr/bin/env bash
set -e

echo "=== Ferrite Client Auto Test ==="

rm -f ferrite.log

export RUST_LOG=ferrite_client=debug

timeout 15 cargo run --bin ferrite-client -- --auto-connect 2>&1 || true

echo ""
echo "=== ferrite.log ==="
cat ferrite.log
