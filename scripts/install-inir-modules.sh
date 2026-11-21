#!/bin/sh
# Install the Ginir-rs iNiR color modules and target manifests into a live iNiR tree.
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/integration/inir"
INIR="${INIR_DIR:-$HOME/.config/quickshell/inir}"
BACKUP="$HOME/gentoo-opt-backups/inir-modules.$(date +%s)"

if [ ! -d "$INIR/scripts/colors" ]; then
  echo "iNiR tree not found: $INIR" >&2
  echo "set INIR_DIR=/path/to/quickshell/inir" >&2
  exit 1
fi

install -d "$INIR/scripts/colors/modules" "$INIR/scripts/colors/targets"

for file in "$SRC"/modules/*.sh; do
  name="$(basename "$file")"
  target="$INIR/scripts/colors/modules/$name"
  if [ -f "$target" ] && ! cmp -s "$file" "$target"; then
    install -d "$BACKUP"
    cp -p "$target" "$BACKUP/$name"
    echo "backup: $BACKUP/$name"
  fi
  install -m755 "$file" "$target"
  echo "module: $name"
done

for file in "$SRC"/targets/*.json; do
  name="$(basename "$file")"
  install -m644 "$file" "$INIR/scripts/colors/targets/$name"
  echo "target: $name"
done

echo
echo "installed Ginir-rs modules into $INIR"
echo "make sure ~/.local/bin/inir-theme exists (./scripts/install.sh)"
