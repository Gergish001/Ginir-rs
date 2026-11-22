#!/bin/sh
# Ginir UI defaults: one dock, centered clock, 3-line-height mono bar.
# Run as the user; the shell config is backed up before it is rewritten.
set -eu

CONFIG="${GINIR_CONFIG:-${XDG_CONFIG_HOME:-$HOME/.config/ginir}/illogical-impulse/config.json}"
if [ ! -f "$CONFIG" ]; then
  echo "Ginir shell config not found: $CONFIG" >&2
  echo "Set GINIR_CONFIG=/path/to/illogical-impulse/config.json" >&2
  exit 1
fi

BACKUP_DIR="${XDG_BACKUP_DIR:-$HOME/gentoo-opt-backups}"
mkdir -p "$BACKUP_DIR"
BACKUP="$BACKUP_DIR/ginir-ui-defaults.$(date +%s).json"
cp -p "$CONFIG" "$BACKUP"
echo "backup: $BACKUP"

python3 - "$CONFIG" <<'PY'
import json, sys

path = sys.argv[1]
with open(path) as f:
    data = json.load(f)

bar = data.setdefault("material", {}).setdefault("bar", {})
bar["height"] = 56
bar["rightScrollAction"] = "none"

m3 = bar.setdefault("m3", {})
layouts = m3.setdefault("layouts", {})
layouts["leftLayout"] = ["leftSidebarButton", "media", "workspaces"]
layouts["middleLayout"] = ["clockWidget"]
# The system tray's tooltips clipped over the weather text; keep the weather
# capsule only. Tray stays available from the control center.
layouts["rightLayout"] = ["weatherBar"]
m3.setdefault("customLayouts", {}).update(
    {"leftLayout": [], "middleLayout": [], "rightLayout": []}
)
m3.setdefault("workspaces", {})["showAppIcons"] = False
m3["verbose"] = True
m3.setdefault("tooltips", {})["clickToShow"] = True

clock = m3.setdefault("clock", {})
clock["timeFontFamily"] = "JetBrainsMono Nerd Font"
clock["dateFontFamily"] = "JetBrainsMono Nerd Font"
clock["timePixelSize"] = 16
clock["datePixelSize"] = 16

typography = data.setdefault("appearance", {}).setdefault("typography", {})
typography["mainFont"] = "JetBrainsMono Nerd Font"
typography["titleFont"] = "JetBrainsMono Nerd Font"
typography["monospaceFont"] = "JetBrainsMono Nerd Font"

data.setdefault("material", {}).setdefault("style", {})["globalStyle"] = "ginir"

dock = data.setdefault("material", {}).setdefault("dock", {})
dock["enable"] = True
dock["position"] = "bottom"

with open(path, "w") as f:
    json.dump(data, f, indent=4, ensure_ascii=False)
    f.write("\n")

print(f"updated: {path}")
PY

if command -v sv >/dev/null 2>&1 && [ -d "$HOME/.config/service/inir" ]; then
  sv restart "$HOME/.config/service/inir" >/dev/null 2>&1 || true
  echo "restarted: ~/.config/service/inir"
elif command -v inir >/dev/null 2>&1; then
  inir restart >/dev/null 2>&1 || true
  echo "restarted: inir"
else
  echo "config written; restart Ginir to apply it"
fi
