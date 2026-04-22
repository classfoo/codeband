#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-}"

if [[ -z "$MODE" ]]; then
  echo "Usage: bash ./scripts/harness_tdd.sh <red|green|refactor>"
  exit 1
fi

run_cmd() {
  local cmd="$1"
  echo ">> $cmd"
  bash -lc "$cmd"
}

run_red() {
  if [[ -z "${TDD_CMD:-}" ]]; then
    echo "TDD red phase requires a failing test command."
    echo "Example:"
    echo "  TDD_CMD='cargo test -p server employee_creation_requires_name' bash ./scripts/harness_tdd.sh red"
    exit 1
  fi

  set +e
  run_cmd "$TDD_CMD"
  local code=$?
  set -e

  if [[ $code -eq 0 ]]; then
    echo "Expected RED (failing) but command passed."
    exit 1
  fi
  echo "RED confirmed: failing test captured."
}

run_green() {
  if [[ -z "${TDD_CMD:-}" ]]; then
    echo "TDD green phase requires a target test command."
    echo "Example:"
    echo "  TDD_CMD='cargo test -p server employee_creation_requires_name' bash ./scripts/harness_tdd.sh green"
    exit 1
  fi
  run_cmd "$TDD_CMD"
  echo "GREEN confirmed: target test now passes."
}

run_refactor() {
  run_cmd "cargo +\${RUST_TOOLCHAIN:-stable} test -p server"
  run_cmd "cargo +\${RUST_TOOLCHAIN:-stable} check -p server"
  run_cmd "npm --workspace @kaisha/web run typecheck"
  if npm --workspace @kaisha/web run | rg -q 'test'; then
    run_cmd "npm --workspace @kaisha/web run test -- --run"
  fi
  echo "Refactor safety checks passed."
}

case "$MODE" in
  red)
    run_red
    ;;
  green)
    run_green
    ;;
  refactor)
    run_refactor
    ;;
  *)
    echo "Unknown mode: $MODE"
    exit 1
    ;;
esac
