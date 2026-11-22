#!/bin/sh
# Build ginir-shell-rs and install it in the user's bin directory.
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SHELL="$ROOT/crates/ginir-shell"

if ! pkg-config --exists gtk4-layer-shell-0 2>/dev/null; then
  PKG_CONFIG_PATH="${PKG_CONFIG_PATH:-}:/usr/local/lib64/pkgconfig:/usr/lib64/pkgconfig:/usr/share/pkgconfig" \
    pkg-config --exists gtk4-layer-shell-0 2>/dev/null || {
      echo "gtk4-layer-shell 1.3 is missing; see docs/SHELL.md" >&2
      exit 1
    }
fi

cargo build --release --manifest-path "$SHELL/Cargo.toml"
install -Dm755 "$SHELL/target/release/ginir-shell-rs" "$HOME/.local/bin/ginir-shell-rs"
printf 'installed %s\n' "$HOME/.local/bin/ginir-shell-rs"
printf 'run: ginir-shell-rs   (hide the QML bar with: inir bar close)\n'
