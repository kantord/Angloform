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
if [ -n "$BINDGEN" ] && [ "$("$BINDGEN" --version)" != "wasm-bindgen $VERSION" ]; then
  # a system/PATH wasm-bindgen exists but doesn't match the version the
  # wasm-bindgen crate itself is pinned to (Cargo.lock) — trusting it
  # anyway produces exactly the "different bindgen format" error this
  # check exists to prevent. Fall through to the pinned download below.
  echo "ignoring $BINDGEN ($("$BINDGEN" --version), need $VERSION)"
  BINDGEN=""
fi
if [ -z "$BINDGEN" ]; then
  ARCH=$(uname -m)
  case "$ARCH" in
    x86_64) TRIPLE=x86_64-unknown-linux-musl;;
    aarch64) TRIPLE=aarch64-unknown-linux-musl;;
    *) echo "unsupported arch: $ARCH — install wasm-bindgen"; exit 1;;
  esac
  # no "v" prefix on the tag — wasm-bindgen/wasm-bindgen (renamed from
  # rustwasm/wasm-bindgen) tags releases as e.g. "0.2.126", not "v0.2.126"
  URL="https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$VERSION/wasm-bindgen-$VERSION-$TRIPLE.tar.gz"
  DEST=/tmp/wasm-bindgen-$VERSION
  if [ ! -x "$DEST/wasm-bindgen" ]; then
    rm -rf "$DEST"
    mkdir -p "$DEST"
    curl -sfL "$URL" | tar xz -C "$DEST" --strip-components=1
  fi
  BINDGEN="$DEST/wasm-bindgen"
fi

OUT=src/lib/wasm
mkdir -p "$OUT"
"$BINDGEN" --target web --out-dir "$OUT" "$REPO_ROOT/target/wasm32-unknown-unknown/release/angloform_wasm.wasm"
echo "wasm glue written to $OUT"