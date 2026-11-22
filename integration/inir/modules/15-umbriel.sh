#!/usr/bin/env bash
# iNiR color module: Umbriel borders, tabs and panels through ginir-theme.
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/module-runtime.sh"
COLOR_MODULE_ID="umbriel"

BIN="$HOME/.local/bin/ginir-theme"
[[ -x "$BIN" ]] || exit 0

if "$BIN" apply --target umbriel --no-reload >> "$STATE_DIR/user/generated/theming_modules.log" 2>&1; then
  log_module "Umbriel colours synced (Rust)"
else
  log_module "Umbriel colours Rust sync failed"
fi
