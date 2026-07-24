#!/usr/bin/env bash
#
# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.
#

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
  arrow-ipc)
    cargo tree \
      -p arrow-ipc \
      --target wasm32-unknown-unknown \
      --no-default-features \
      --features lz4,zstd \
      --edges normal,build \
      --locked \
      >"$tree_output"
    ;;
  *)
    echo "unknown WASM dependency policy profile: $profile" >&2
    exit 2
    ;;
esac

if grep -En '(^|[[:space:]])zstd-sys v' "$tree_output" >/dev/null; then
  echo "$profile WASM dependency policy failed: zstd-sys is active" >&2
  grep -En '(^|[[:space:]])zstd-sys v' "$tree_output" >&2
  exit 1
fi

echo "$profile WASM dependency policy passed"
