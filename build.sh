#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE"

echo "==> cargo component build"
cargo component build --release --target wasm32-wasip2

WASM_PATH="${CARGO_TARGET_DIR:-target}/wasm32-wasip2/release/greentic_knowledge_writer_extension.wasm"
if [ ! -f "$WASM_PATH" ]; then
  echo "ERROR: wasm not found at $WASM_PATH" >&2
  exit 1
fi

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

cp describe.json "$STAGE/"
cp "$WASM_PATH" "$STAGE/extension.wasm"

WASM_SHA256="$(shasum -a 256 "$STAGE/extension.wasm" | cut -d' ' -f1)"
sed "s/0000000000000000000000000000000000000000000000000000000000000000/${WASM_SHA256}/g" \
  "$STAGE/describe.json" > "$STAGE/describe.json.tmp"
mv "$STAGE/describe.json.tmp" "$STAGE/describe.json"

EXT_ID="$(jq -r .metadata.id "$STAGE/describe.json")"
EXT_VERSION="$(jq -r .metadata.version "$STAGE/describe.json")"
OUT="$HERE/${EXT_ID}-${EXT_VERSION}.gtxpack"
TMP_ZIP="$STAGE/../greentic_knowledge_writer_$$.zip"
(cd "$STAGE" && zip -r "$TMP_ZIP" .) > /dev/null
mv "$TMP_ZIP" "$OUT"

echo "==> built $OUT"
echo "==> size: $(du -h "$OUT" | cut -f1)"
