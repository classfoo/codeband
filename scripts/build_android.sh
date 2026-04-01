#!/usr/bin/env bash
set -euo pipefail

MANIFEST_PATH="apps/desktop/src-tauri/Cargo.toml"

npm run build:web
cargo tauri android build --manifest-path "$MANIFEST_PATH"
