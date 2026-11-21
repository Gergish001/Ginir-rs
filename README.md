# Ginir-rs

Rust-first theming and backend layer built on top of [iNiR](https://github.com/snowarch/iNiR):
palette maths, accent parsing/transformation and generators that push the
derived accent into other applications.

This is a new project that started from iNiR; the GitHub repository is a fork
only because that is how it was created. The original iNiR QML tree is kept
untouched in [`upstream/inir/`](upstream/inir/) as reference.

> **What "Rust-first" means here.** The theming/backend logic — reading iNiR's
> palette, transforming the accent, writing app configs, reloading the app — is
> Rust (`inir-theme`). The iNiR QML widgets are not reimplemented: they still run
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
install -m755 target/release/inir-theme ~/.local/bin/inir-theme
# or
./scripts/install.sh
```

Requires Rust 1.85+; no Qt, no Python.

## Usage

```sh
inir-theme list
inir-theme doctor
inir-theme apply --target herdr
inir-theme apply --all --dry-run
inir-theme apply --all
```

Source selection is automatic: iNiR's generated
`app-palette.json` + `terminal.json`, falling back to
`~/.config/kitty/theme.conf`. Override with:

```sh
inir-theme apply --all \
  --app-palette ~/.local/state/quickshell/user/generated/app-palette.json \
  --terminal-json ~/.local/state/quickshell/user/generated/terminal.json \
  --accent '#b78cda'
```

Every write is backed up to `~/gentoo-opt-backups/`. `--dry-run` only reports.

## iNiR integration

`integration/inir/` holds the color modules that call the Rust binary and their
target manifests. Install them into a live iNiR tree with:

```sh
./scripts/install.sh                 # build + install ~/.local/bin/inir-theme
./scripts/install-inir-modules.sh    # copy modules/targets, backing up conflicts
inir theme doctor
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
crates/inir-theme-core/         accent parsing, colour maths, palette, targets
crates/inir-theme-cli/          inir-theme binary
crates/inir-theme-core/assets/  OpenCode template (derived from iNiR)
integration/inir/               iNiR modules + target manifests (additive)
patches/                        external C patches outside Rust
legacy/                         original Python/Bash implementations
scripts/install.sh              local install of the binary
scripts/install-inir-modules.sh install modules into a live iNiR tree
upstream/inir/                  untouched iNiR QML reference tree
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

The OpenCode template in `crates/inir-theme-core/assets/` is derived from iNiR's
generated theme and remains under the iNiR license. The fetch patch is for the
separate 3D `fetch` source.

## Roadmap

1. Port more accent consumers (lazygit, GTK/Qt accents, other terminals) into
   `inir-theme-core`, each as a separate target.
2. Move the generators into `inir-accel` upstream so the shell gets one Rust
   binary instead of a Python helper plus Bash modules.
3. Add a Hub colour-theme/plugin so users install the targets without a fork.
4. Ship the Gentoo overlay for `inir-theme`, `qs-rs`, `inir-accel`, `inir-cli`
   and iNiR.
