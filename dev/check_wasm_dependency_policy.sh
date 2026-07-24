#!/usr/bin/env bash

set -euo pipefail

profile="${1:-parquet}"

if [[ ! -f Cargo.lock ]]; then
  cargo generate-lockfile
fi

tree_output="$(mktemp)"
trap 'rm -f "$tree_output"' EXIT

case "$profile" in
  parquet)
    cargo tree \
      -p parquet \
      --target wasm32-unknown-unknown \
      --no-default-features \
      --features arrow,async,object_store,snap,brotli,flate2-zlib-rs,lz4,zstd,base64,simdutf8 \
      --edges normal,build \
      --locked \
      >"$tree_output"
    ;;
  *)
    echo "unknown WASM dependency policy profile: $profile" >&2
    exit 2
    ;;
esac

if rg -n '(^|[[:space:]])zstd-sys v' "$tree_output" >/dev/null; then
  echo "$profile WASM dependency policy failed: zstd-sys is active" >&2
  rg -n '(^|[[:space:]])zstd-sys v' "$tree_output" >&2
  exit 1
fi

echo "$profile WASM dependency policy passed"
