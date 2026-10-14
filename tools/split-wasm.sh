#!/usr/bin/env bash
# Split each dist/pkg-*/ftt_bg.wasm into parts of at most $MB MiB, for static
# hosts with a per-file size cap. The loader joins them (parts.json).
#   tools/split-wasm.sh dist 12
set -euo pipefail
DIST="${1:-dist}"; MB="${2:-12}"
for pkg in "$DIST"/pkg-*; do
  [ -f "$pkg/ftt_bg.wasm" ] || continue
  ( cd "$pkg"
    split -b "${MB}M" -d -a 1 --additional-suffix=.wasm ftt_bg.wasm ftt_bg.part
    rm ftt_bg.wasm
    printf '{"parts":[%s]}\n' "$(ls ftt_bg.part*.wasm | sed 's/.*/"&"/' | paste -sd,)" > parts.json
    cat parts.json )
done
