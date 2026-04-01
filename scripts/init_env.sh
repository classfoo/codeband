#!/usr/bin/env bash
set -euo pipefail

# One-shot environment bootstrap for Codeband.
# Supports: Linux (apt), macOS (Homebrew), Windows (Chocolatey in Git Bash).

AUTO_YES=0
DRY_RUN=0
WITH_MOBILE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --yes|-y)
      AUTO_YES=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --with-mobile)
      WITH_MOBILE=1
      shift
      ;;
    *)
      echo "Unknown argument: $1"
      echo "Usage: bash ./scripts/init_env.sh [--yes] [--dry-run] [--with-mobile]"
      exit 1
      ;;
  esac
done

log() { echo "[init] $*"; }
warn() { echo "[init][warn] $*"; }

run_cmd() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "[dry-run] $*"
  else
    eval "$@"
  fi
}

ask() {
  local message="$1"
  if [[ "$AUTO_YES" -eq 1 ]]; then
    return 0
  fi
  read -r -p "$message [y/N]: " reply
  [[ "$reply" =~ ^[Yy]$ ]]
}

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

OS="$(uname -s)"

install_linux_apt() {
  if ! has_cmd sudo; then
    warn "sudo not found; cannot auto-install apt dependencies."
    return 1
  fi

  run_cmd "sudo apt-get update"
  run_cmd "sudo apt-get install -y curl git build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf"

  if ! has_cmd node || ! has_cmd npm; then
    run_cmd "sudo apt-get install -y nodejs npm"
  fi

  if [[ "$WITH_MOBILE" -eq 1 ]]; then
    run_cmd "sudo apt-get install -y openjdk-17-jdk"
    warn "Android SDK/NDK are not auto-installed here. Install Android Studio, then configure ANDROID_HOME and sdkmanager packages."
  fi
}

install_macos_brew() {
  if ! has_cmd brew; then
    warn "Homebrew is missing. Install from https://brew.sh first."
    return 1
  fi

  run_cmd "brew update"
  run_cmd "brew install curl git node rustup-init"

  if [[ "$WITH_MOBILE" -eq 1 ]]; then
    run_cmd "brew install --cask android-studio"
    warn "Open Android Studio once to install SDK/NDK and accept licenses."
    warn "For iOS builds, install Xcode from App Store and run: xcode-select --install"
  fi
}

install_windows_choco() {
  if ! has_cmd choco; then
    warn "Chocolatey is missing. Install from https://chocolatey.org/install first."
    return 1
  fi

  run_cmd "choco install -y git curl nodejs rustup.install"

  if [[ "$WITH_MOBILE" -eq 1 ]]; then
    run_cmd "choco install -y openjdk17 androidstudio"
    warn "Install Android SDK/NDK from Android Studio SDK Manager after launch."
  fi
}

ensure_rust() {
  if has_cmd rustup && has_cmd cargo; then
    log "Rust toolchain already installed"
    return
  fi

  if [[ "$OS" == "Linux" || "$OS" == "Darwin" ]]; then
    run_cmd "curl https://sh.rustup.rs -sSf | sh -s -- -y"
    if [[ "$DRY_RUN" -eq 0 ]]; then
      # shellcheck disable=SC1091
      source "$HOME/.cargo/env"
    fi
  else
    warn "Please install rustup/cargo manually for this OS."
  fi
}

ensure_tauri_cli() {
  if has_cmd cargo-tauri; then
    log "cargo-tauri already installed"
  else
    run_cmd "cargo install tauri-cli"
  fi
}

ensure_node_modules() {
  run_cmd "npm install"
}

ensure_rust_targets() {
  run_cmd "rustup target add x86_64-unknown-linux-gnu || true"
  run_cmd "rustup target add x86_64-apple-darwin aarch64-apple-darwin || true"
  run_cmd "rustup target add x86_64-pc-windows-msvc || true"

  if [[ "$WITH_MOBILE" -eq 1 ]]; then
    run_cmd "rustup target add aarch64-apple-ios aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android || true"
  fi
}

verify_project() {
  run_cmd "npm run check:web"
  run_cmd "cargo check -p domain -p application -p server"
}

main() {
  log "Detected OS: $OS"

  if ask "Proceed with environment bootstrap?"; then
    case "$OS" in
      Linux)
        install_linux_apt || true
        ;;
      Darwin)
        install_macos_brew || true
        ;;
      MINGW*|MSYS*|CYGWIN*|Windows_NT)
        install_windows_choco || true
        ;;
      *)
        warn "Unsupported OS for automatic package manager install: $OS"
        ;;
    esac

    ensure_rust

    if [[ "$DRY_RUN" -eq 0 && -f "$HOME/.cargo/env" ]]; then
      # shellcheck disable=SC1091
      source "$HOME/.cargo/env"
    fi

    ensure_tauri_cli
    ensure_node_modules
    ensure_rust_targets
    verify_project

    log "Environment initialization completed."
  else
    log "Initialization cancelled by user."
  fi
}

main
