#!/usr/bin/env bash
set -euo pipefail

MANIFEST_PATH="apps/desktop/src-tauri/Cargo.toml"

cargo tauri android init --manifest-path "$MANIFEST_PATH"
