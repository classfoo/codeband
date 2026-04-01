#!/usr/bin/env bash
set -euo pipefail

MANIFEST_PATH="apps/desktop/src-tauri/Cargo.toml"

cargo tauri ios init --manifest-path "$MANIFEST_PATH"
