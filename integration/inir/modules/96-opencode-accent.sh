#!/usr/bin/env bash
# iNiR color module: keep the active OpenCode inir theme's accent in sync.
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/module-runtime.sh"
COLOR_MODULE_ID="opencode-accent"

BIN="$HOME/.local/bin/ginir-theme"
[[ -x "$BIN" ]] || exit 0

if "$BIN" apply --target opencode --no-reload >> "$STATE_DIR/user/generated/theming_modules.log" 2>&1; then
  log_module "opencode accent synced (Rust)"
else
  log_module "opencode accent Rust sync failed"
fi
