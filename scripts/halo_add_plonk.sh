#!/usr/bin/env bash
# Halo2 addition circuit: setup -> prove -> verify (external-user flow).
# Artifacts go to build/halo-add/ by default.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:-$ROOT/build/halo-add}"

cd "$ROOT/halo"
cargo run --release --bin halo_add_demo -- "$OUT_DIR"
