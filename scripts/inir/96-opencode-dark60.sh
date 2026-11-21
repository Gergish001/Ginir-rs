#!/usr/bin/env bash
# iNiR color module: OpenCode inir-dark60 theme through the Rust binary.
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/module-runtime.sh"
COLOR_MODULE_ID="opencode-dark60"

BIN="$HOME/.local/bin/inir-theme"
[[ -x "$BIN" ]] || exit 0

if "$BIN" apply --target opencode --no-reload >> "$STATE_DIR/user/generated/theming_modules.log" 2>&1; then
  log_module "opencode dark60 synced (Rust)"
else
  log_module "opencode dark60 Rust sync failed"
fi
