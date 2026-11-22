#!/usr/bin/env bash
# iNiR color module: Firefox-family ui.* system colours through ginir-theme.
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/module-runtime.sh"
COLOR_MODULE_ID="firefox"

BIN="$HOME/.local/bin/ginir-theme"
[[ -x "$BIN" ]] || exit 0

if "$BIN" apply --target firefox --no-reload >> "$STATE_DIR/user/generated/theming_modules.log" 2>&1; then
  log_module "Firefox system colours synced (Rust)"
else
  log_module "Firefox system colours Rust sync failed"
fi
