#!/usr/bin/env bash
# iNiR color module: 3D fetch label colour (not fastfetch).
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/module-runtime.sh"
COLOR_MODULE_ID="fetch3d"

BIN="$HOME/.local/bin/inir-theme"
[[ -x "$BIN" ]] || exit 0

if "$BIN" apply --target fetch3d --no-reload >> "$STATE_DIR/user/generated/theming_modules.log" 2>&1; then
  log_module "3D fetch label color synced (Rust)"
else
  log_module "3D fetch Rust sync failed"
fi
