#!/usr/bin/env bash
# iNiR color module: sync Herdr through the Rust ginir-theme binary.
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/module-runtime.sh"
COLOR_MODULE_ID="herdr"

BIN="$HOME/.local/bin/ginir-theme"
[[ -x "$BIN" ]] || exit 0

if "$BIN" apply --target herdr --no-reload >> "$STATE_DIR/user/generated/theming_modules.log" 2>&1; then
  log_module "herdr theme synced (Rust)"
else
  log_module "herdr Rust sync failed"
fi

if command -v herdr >/dev/null 2>&1 && pgrep -x herdr >/dev/null 2>&1; then
  herdr server reload-config >/dev/null 2>&1 || true
fi
