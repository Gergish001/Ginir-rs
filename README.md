<p align="center">
  <img src="docs/images/readme/mark.svg" alt="" width="56">
</p>

<h1 align="center">Ginir-rs</h1>

<p align="center">
  Rust-first theming and backend layer for <a href="https://github.com/snowarch/iNiR">iNiR</a> on Gentoo.<br>
  Parse the accent once, transform it per app, keep every surface in sync.
</p>

<p align="center">
  <a href="https://github.com/Gergish001/Ginir-rs/actions/workflows/rust.yml"><img src="https://github.com/Gergish001/Ginir-rs/actions/workflows/rust.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://github.com/Gergish001/Ginir-rs/stargazers"><img src="https://img.shields.io/github/stars/Gergish001/Ginir-rs?style=flat-square&color=111111" alt="Stars"></a>
  <a href="https://github.com/Gergish001/Ginir-rs/releases"><img src="https://img.shields.io/badge/version-0.1.0-111111?style=flat-square" alt="Version 0.1.0"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-111111?style=flat-square" alt="GPL-3.0"></a>
  <a href="rust-toolchain.toml"><img src="https://img.shields.io/badge/rust-1.85%2B-111111?style=flat-square&logo=rust&logoColor=white" alt="Rust 1.85+"></a>
</p>

<p align="center">
  <a href="#targets">Targets</a> ·
  <a href="#build-and-install">Install</a> ·
  <a href="#usage">Usage</a> ·
  <a href="#inir-integration">iNiR integration</a> ·
  <a href="https://github.com/snowarch/iNiR">Upstream</a> ·
  <a href="https://github.com/Gergish001/Ginir-rs/issues">Issues</a>
</p>

<p align="center">
  <sub>
    <a href="README.md">English</a> ·
    <a href="docs/readme/README.ru.md">Русский</a> ·
    <a href="docs/readme/TRANSLATING.md">help translate</a>
  </sub>
</p>

Ginir-rs is a new Rust project that started from iNiR; the GitHub repository is a
fork only because that is how it was created. It covers palette maths, accent
parsing/transformation, and generators that push the derived accent into other
applications. The original iNiR QML tree is kept untouched in
[`upstream/inir/`](upstream/inir/) as reference.

> **What "Rust-first" means here.** The theming/backend logic — reading iNiR's
> palette, transforming the accent, writing app configs, reloading the app — is
> Rust (`ginir-theme`). The iNiR QML widgets are not reimplemented: they still run
> on `qs-rs`, a separate upstream project. Rewriting those widgets would be a new
> shell, not a port.

## Targets

Each target parses the current iNiR accent and writes it into another app,
with a per-app transform.

| id | What it writes | Transform |
|---|---|---|
| `herdr` | `~/.config/herdr/config.toml` `[theme.custom]` | exact iNiR accent, darker surfaces |
| `opencode` | `~/.config/opencode/themes/inir.json` `m3Tertiary` | exact iNiR accent (active theme patch) |
| `niri` | Niri `focus-ring active-color` in `config.kdl` + `config.d/*.kdl` | −20% lightness, saturation ×1.34 |
| `fetch3d` | `~/.config/fetch/config` `label_color` | −10% lightness (3D fetch, **not** fastfetch) |
| `launchers` | `~/.local/bin/cmx`, `~/.local/bin/lavax` | nearest ANSI / accent + teal |

`fetch3d` is the custom 3D fetch renderer. **fastfetch, btop and yazi are not
touched by this project** and get no patches or config changes.

## Build and install

```sh
cargo build --release
install -m755 target/release/ginir-theme ~/.local/bin/ginir-theme
# or
./scripts/install.sh
```

Requires Rust 1.85+; no Qt, no Python.

`scripts/install.sh` installs `ginir-theme` and the `ginir`/`gi` wrappers
around the iNiR CLI (`inir`). `ginir-theme` is the Rust tool; `ginir` (short:
`gi`) is the shell command. `gi` is skipped if that name is already taken.

## Usage

```sh
ginir-theme list
ginir-theme doctor
ginir-theme apply --target herdr
ginir-theme apply --all --dry-run
ginir-theme apply --all
```

Source selection is automatic: iNiR's generated
`app-palette.json` + `terminal.json`, falling back to
`~/.config/kitty/theme.conf`. Override with:

```sh
ginir-theme apply --all \
  --app-palette ~/.local/state/quickshell/user/generated/app-palette.json \
  --terminal-json ~/.local/state/quickshell/user/generated/terminal.json \
  --accent '#b78cda'
```

Every write is backed up to `~/gentoo-opt-backups/`. `--dry-run` only reports.

## iNiR integration

`integration/inir/` holds the color modules that call the Rust binary and their
target manifests. Install them into a live iNiR tree with:

```sh
./scripts/install.sh                  # build + install ginir-theme, ginir, gi
./scripts/install-ginir-modules.sh    # copy modules/targets, backing up conflicts
ginir theme doctor
```

| module | target manifest | Rust target |
|---|---|---|
| `05-niri.sh` | `niri.json` | `niri` |
| `95-herdr.sh` | `herdr.json` | `herdr` |
| `96-opencode-accent.sh` | `opencode-accent.json` | `opencode` |
| `97-fetch3d.sh` | `fetch3d.json` | `fetch3d` |
| `98-launchers.sh` | `launchers.json` | `launchers` |

The modules read `app-palette.json` and `terminal.json`, which iNiR writes before
the theming modules run, so there is no race with the kitty target.

## Layout

```
crates/ginir-theme-core/          accent parsing, colour maths, palette, targets
crates/ginir-theme-cli/           ginir-theme binary
crates/ginir-theme-core/assets/   OpenCode template (derived from iNiR)
integration/inir/                 iNiR modules + target manifests (additive)
patches/                          external C patches outside Rust
legacy/                           original Python/Bash implementations
scripts/install.sh                install ginir-theme, ginir and gi
scripts/install-ginir-modules.sh  install modules into a live iNiR tree
scripts/ginir, scripts/gi         wrappers around the iNiR CLI
upstream/inir/                    untouched iNiR QML reference tree
```

## Gentoo

The Rust binary installs with `cargo`/`scripts/install.sh` today. The full stack
needs a Gentoo overlay:

- `gui-apps/qs-rs` (Rust + Qt 6/LayerShellQt, or the prebuilt Qt bundle)
- `dev-util/inir-accel` (Rust)
- `dev-util/inir-cli` (Go)
- `gui-apps/inir` (QML payload + modules + runit/OpenRC service)

Those pieces are separate upstream repositories; see `packaging/gentoo/README.md`.
A full iNiR ebuild is not included yet, because qs-rs/Qt packaging is the hard
part, not this CLI.

## Relationship to upstream and licenses

- iNiR: GPL-3.0
- qs-rs: LGPL-3.0
- inir-accel: LGPL-3.0
- inir-cli: GPL-3.0
- this project: **GPL-3.0**

The OpenCode template in `crates/ginir-theme-core/assets/` is derived from iNiR's
generated theme and remains under the iNiR license. The fetch patch is for the
separate 3D `fetch` source.

## Roadmap

1. Port more accent consumers (lazygit, GTK/Qt accents, other terminals) into
   `ginir-theme-core`, each as a separate target.
2. Move the generators into `inir-accel` upstream so the shell gets one Rust
   binary instead of a Python helper plus Bash modules.
3. Add a Hub colour-theme/plugin so users install the targets without a fork.
4. Ship the Gentoo overlay for `ginir-theme`, `qs-rs`, `inir-accel`, `inir-cli`
   and iNiR.
