#!/usr/bin/env bash
# Build the wasm linter (crates/wasm) and generate the wasm-bindgen glue
#into src/lib/wasm, which Vite bundles. Run from the web/ directory.
set -euo pipefail
cd "$(dirname "$0")/.."

REPO_ROOT="$(cd .. && pwd)"
VERSION=0.2.126

CARGO_TARGET_DIR="$REPO_ROOT/target" cargo build --release -p angloform-wasm \
  --target wasm32-unknown-unknown --manifest-path "$REPO_ROOT/Cargo.toml"

BINDGEN=${WASM_BINDGEN:-$(command -v wasm-bindgen || true)}
if [ -z "$BINDGEN" ]; then
  ARCH=$(uname -m)
  case "$ARCH" in
    x86_64) TRIPLE=x86_64-unknown-linux-musl;;
    aarch64) TRIPLE=aarch64-unknown-linux-musl;;
    *) echo "unsupported arch: $ARCH — install wasm-bindgen"; exit 1;;
  esac
  URL="https://github.com/rustwasm/wasm-bindgen/releases/download/v$VERSION/wasm-bindgen-$VERSION-$TRIPLE.tar.gz"
  DEST=/tmp/wasm-bindgen-$VERSION
  if [ ! -x "$DEST/wasm-bindgen" ]; then
    rm -rf "$DEST"
    mkdir -p "$DEST"
    curl -L "$URL" | tar xz -C "$DEST" --strip-components=1
  fi
  BINDGEN="$DEST/wasm-bindgen"
fi

OUT=src/lib/wasm
mkdir -p "$OUT"
"$BINDGEN" --target web --out-dir "$OUT" "$REPO_ROOT/target/wasm32-unknown-unknown/release/angloform_wasm.wasm"
echo "wasm glue written to $OUT"