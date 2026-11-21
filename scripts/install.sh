#!/bin/sh
set -eu

cd "$(dirname "$0")/.."
cargo build --release
install -d "$HOME/.local/bin"
install -m755 target/release/ginir-theme "$HOME/.local/bin/ginir-theme"
rm -f "$HOME/.local/bin/inir-theme"
printf 'installed %s\n' "$HOME/.local/bin/ginir-theme"

# `ginir` is a thin wrapper around the iNiR CLI (`inir`); `gi` is its short alias.
GINIR_OK=0
if [ -e "$HOME/.local/bin/ginir" ] && ! grep -q 'Run the iNiR shell CLI under the Ginir name' "$HOME/.local/bin/ginir" 2>/dev/null; then
  printf 'ginir exists and is not ours; leaving it alone\n'
elif command -v ginir >/dev/null 2>&1 && [ "$(command -v ginir)" != "$HOME/.local/bin/ginir" ]; then
  printf 'ginir is already provided by %s; skipping the wrapper\n' "$(command -v ginir)"
else
  install -m755 scripts/ginir "$HOME/.local/bin/ginir"
  GINIR_OK=1
  printf 'installed %s\n' "$HOME/.local/bin/ginir"
fi

if [ "$GINIR_OK" -eq 1 ]; then
  if [ -e "$HOME/.local/bin/gi" ] && ! grep -q 'Short alias for ginir' "$HOME/.local/bin/gi" 2>/dev/null; then
    printf 'gi exists and is not ours; leaving it alone\n'
  elif command -v gi >/dev/null 2>&1 && [ "$(command -v gi)" != "$HOME/.local/bin/gi" ]; then
    printf 'gi is already provided by %s; skipping the short alias\n' "$(command -v gi)"
  else
    install -m755 scripts/gi "$HOME/.local/bin/gi"
    printf 'installed %s\n' "$HOME/.local/bin/gi"
  fi
fi

printf 'run: ginir-theme doctor && ginir-theme apply --all --dry-run\n'
printf 'shell: ginir <command> (short: gi)\n'
