#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-$(uname -s)}"

case "$TARGET" in
  Linux)
    ./scripts/build_linux.sh
    ;;
  Darwin)
    ./scripts/build_macos.sh
    ;;
  Windows_NT|MINGW*|MSYS*|CYGWIN*)
    ./scripts/build_windows.sh
    ;;
  ios|iOS)
    ./scripts/build_ios.sh
    ;;
  android|Android)
    ./scripts/build_android.sh
    ;;
  *)
    echo "Unsupported target: $TARGET"
    echo "Use one of: Linux, Darwin, Windows_NT, ios, android"
    exit 1
    ;;
esac
