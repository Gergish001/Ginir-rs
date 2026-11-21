#!/bin/sh
set -eu

cd "$(dirname "$0")/.."
cargo build --release
install -d "$HOME/.local/bin"
install -m755 target/release/inir-theme "$HOME/.local/bin/inir-theme"
printf 'installed %s\n' "$HOME/.local/bin/inir-theme"
printf 'run: inir-theme doctor && inir-theme apply --all --dry-run\n'
