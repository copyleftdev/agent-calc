#!/usr/bin/env bash

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

if ! git diff --quiet -- || ! git diff --cached --quiet --; then
    printf 'Refusing to package a dirty tracked worktree.\n' >&2
    exit 1
fi

HEAD_SHA="$(git rev-parse HEAD)"
ARCHIVE="${1:-/tmp/oddly-exact-sites-${HEAD_SHA:0:8}.tar.gz}"
if [[ "$ARCHIVE" != /* ]]; then
    ARCHIVE="$ROOT/$ARCHIVE"
fi

BUILD_ROOT="$(mktemp -d)"
trap 'rm -rf "$BUILD_ROOT"' EXIT

WRANGLER_OUT="$BUILD_ROOT/wrangler"
PACKAGE_ROOT="$BUILD_ROOT/package"

npm run build:worker
./node_modules/.bin/wrangler deploy --dry-run --outdir "$WRANGLER_OUT"

mapfile -t WASM_MODULES < <(
    find "$WRANGLER_OUT" -maxdepth 1 -type f -name '*.wasm' -print
)
if (( ${#WASM_MODULES[@]} != 1 )); then
    printf 'Expected exactly one bundled WASM module; found %s.\n' \
        "${#WASM_MODULES[@]}" >&2
    exit 1
fi

mkdir -p "$PACKAGE_ROOT/.open-next" "$PACKAGE_ROOT/.openai"
cp "$WRANGLER_OUT/worker.js" "$PACKAGE_ROOT/.open-next/worker.js"
cp "${WASM_MODULES[0]}" "$PACKAGE_ROOT/.open-next/"
cp -a .open-next/assets "$PACKAGE_ROOT/.open-next/assets"
cp .openai/hosting.json "$PACKAGE_ROOT/.openai/hosting.json"
cp wrangler.jsonc "$PACKAGE_ROOT/wrangler.jsonc"

tar -C "$PACKAGE_ROOT" -czf "$ARCHIVE" \
    .open-next \
    .openai/hosting.json \
    wrangler.jsonc

printf 'Sites archive: %s\n' "$ARCHIVE"
