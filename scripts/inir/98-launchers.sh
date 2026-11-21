#!/usr/bin/env bash
# iNiR color module: cmatrix/lavat wrappers with the current accent.
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/module-runtime.sh"
COLOR_MODULE_ID="launchers"

BIN="$HOME/.local/bin/inir-theme"
[[ -x "$BIN" ]] || exit 0

if "$BIN" apply --target launchers --no-reload >> "$STATE_DIR/user/generated/theming_modules.log" 2>&1; then
  log_module "cmatrix/lavat wrappers synced (Rust)"
else
  log_module "cmatrix/lavat Rust sync failed"
fi
