#!/usr/bin/env bash
# Sync the Herdr UI palette with the current iNiR/kitty theme.
# Runs as an iNiR color module; reads app-palette.json + terminal.json,
# which are generated before the theming modules (no race with kitty).
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/module-runtime.sh"
COLOR_MODULE_ID="herdr"

PALETTE="$STATE_DIR/user/generated/app-palette.json"
TERMINAL="$STATE_DIR/user/generated/terminal.json"
THEME_BIN="$HOME/.local/bin/theme-from-kitty"

[[ -f "$PALETTE" && -f "$TERMINAL" && -x "$THEME_BIN" ]] || exit 0

accent=$(jq -r '.app_accent // .primary // empty' "$PALETTE" 2>/dev/null || true)
args=(--source inir --only-herdr --app-palette "$PALETTE" --terminal-json "$TERMINAL" --no-reload)
if [[ "$accent" =~ ^#[0-9A-Fa-f]{6}$ ]]; then
  args+=(--herdr-accent "$accent" --herdr-accent-exact)
fi

if "$THEME_BIN" "${args[@]}" >> "$STATE_DIR/user/generated/theming_modules.log" 2>&1; then
  log_module "herdr theme synced${accent:+ (accent $accent)}"
else
  log_module "herdr theme sync failed"
fi

if command -v herdr >/dev/null 2>&1 && pgrep -x herdr >/dev/null 2>&1; then
  herdr server reload-config >/dev/null 2>&1 || true
fi
