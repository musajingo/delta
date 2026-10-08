#!/usr/bin/env bash

# Use before committing.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

MSRV="1.94"

# Check that a toolchain is present.
require_toolchain() {
  local tc="$1"; shift
  if ! rustup toolchain list | grep -q "^$tc"; then
    echo "Toolchain '$tc' is missing. Install it with:" >&2
    echo "  rustup toolchain install $tc $*" >&2
    exit 1
  fi
}

require_toolchain stable --component rustfmt --component clippy
require_toolchain "$MSRV"

run() {
  echo "==> $*"
  "$@"
}

run cargo +stable fmt --check
run cargo +stable clippy --all-features --all-targets -- -D warnings
run cargo +stable test --all-features
run env RUSTDOCFLAGS="-D warnings" cargo +stable doc --all-features --no-deps

for features in "" "serde" "utoipa" "sqlx"; do
  run cargo +stable test --no-default-features ${features:+--features "$features"} --all-targets
done

echo "==> Checking MSRV ($MSRV)..."
run cargo +"$MSRV" check --all-features

echo "All checks passed."
