#!/usr/bin/env bash
# Regenerates the embedded web runtime artifacts in `web-runtime/`, which
# `rune build --target web` emits into a project's web bundle.
#
# Prerequisites:
#   - rustup target add wasm32-unknown-unknown
#   - wasm-bindgen-cli matching the resolved `wasm-bindgen` crate version
#     (currently 0.2.108): cargo install -f wasm-bindgen-cli --version 0.2.108
#   - jco installed (for the preview2-shim source): npm i -g @bytecodealliance/jco
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> Building rune-host web cdylib (release)"
cargo build --target wasm32-unknown-unknown -p rune-host --lib --release

echo "==> wasm-bindgen -> web-runtime/"
wasm-bindgen target/wasm32-unknown-unknown/release/web.wasm \
  --out-dir web-runtime --target web --no-typescript

echo "==> Copying preview2-shim browser lib"
SHIM="$(find "$(npm root -g)" -type d -name preview2-shim 2>/dev/null | head -1)"
if [ -z "$SHIM" ]; then
  echo "preview2-shim not found; install jco: npm i -g @bytecodealliance/jco" >&2
  exit 1
fi
rm -rf web-runtime/shim && mkdir -p web-runtime/shim/lib
for d in browser common io synckit; do
  [ -d "$SHIM/lib/$d" ] && cp -r "$SHIM/lib/$d" "web-runtime/shim/lib/$d" || true
done

echo "==> Done. web-runtime/ contents:"
ls -la web-runtime/
