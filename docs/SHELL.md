# Ginir Rust shell (experimental)

`crates/ginir-shell` is a separate Rust-native shell, not a QML port. It uses
GTK4 for widgets and `gtk4-layer-shell` for Wayland layers. The QML iNiR tree is
the visual/behaviour reference and is **never written** by this crate: it only
reads iNiR's palette and config.

## Current slice

The top bar:

- niri workspaces and the focused window title;
- clock (`chrono`, local time);
- weather from `~/.local/state/quickshell/user/weather-cache.json`;
- battery and network status;
- colors from iNiR's `colors.json`, font and bar height from
  `~/.config/illogical-impulse/config.json`.

It is a layer-shell surface on the top layer with a fixed exclusive zone.

An optional dock follows the active iNiR family: `ii` uses
`material.dock.enable`, iRiS uses `iris.dock.enable`. The current original iNiR
config is iRiS with the dock off, so the shell shows the bar only; the dock
appears when its config turns it on. The side panels, control center, tray,
notifications, settings and overview are still the QML shell's job and are the
next modules.

## Prerequisites

- Rust 1.93+
- GTK 4.20 development headers
- `gtk4-layer-shell` 1.3 (Gentoo has no ebuild at the time of writing)

Build the C library once:

```sh
git clone https://github.com/wmww/gtk4-layer-shell /tmp/gtk4-layer-shell
cd /tmp/gtk4-layer-shell
meson setup build --prefix=/usr/local -Dvapi=false -Dsmoke-tests=false
meson compile -C build
sudo meson install -C build
sudo ldconfig
```

The repo's `.cargo/config.toml` adds `/usr/local/lib64/pkgconfig` to
`PKG_CONFIG_PATH` so the Rust sys crate finds it.

## Build and run

```sh
./scripts/install-shell.sh          # release build + ~/.local/bin/ginir-shell-rs
```

For a development build:

```sh
cargo build --manifest-path crates/ginir-shell/Cargo.toml
./crates/ginir-shell/target/debug/ginir-shell-rs
```

While testing against the QML bar, hide it first:

```sh
inir bar close        # with the original XDG paths
ginir-shell-rs
```

Restore it and stop the Rust bar:

```sh
pkill -x ginir-shell-rs
inir bar open
```

## Why a separate workspace

`crates/ginir-shell` declares its own `[workspace]` so `cargo test --workspace`
at the repo root does not need GTK/layer-shell. The theming crates keep their
small CI; the shell is built explicitly.
