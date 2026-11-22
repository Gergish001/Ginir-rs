#!/usr/bin/env bash
# iNiR color module: LiMusic's live matugen.css through ginir-theme.
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/module-runtime.sh"
COLOR_MODULE_ID="limusic"

BIN="$HOME/.local/bin/ginir-theme"
[[ -x "$BIN" ]] || exit 0

if "$BIN" apply --target limusic --no-reload >> "$STATE_DIR/user/generated/theming_modules.log" 2>&1; then
  log_module "LiMusic colours synced (Rust)"
else
  log_module "LiMusic colours Rust sync failed"
fi
