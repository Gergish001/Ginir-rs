# iNiR → Rust porting status

This repository ports the parts of iNiR that are not the QML UI: palette maths,
the generators that write other apps' configs, and a separate native shell.
The QML shell remains the reference (646 QML files, ~170k lines); the Rust shell
does not rewrite it in place.

## Native Rust shell

`crates/ginir-shell` is a separate GTK4 + `gtk4-layer-shell` workspace. The first
slice is the top bar (niri workspaces, focused window, clock, weather, battery,
network) in iNiR's palette and font. It reads iNiR's config and generated colors;
it never writes the QML tree. Next: dock, side panels, control center, tray,
notifications, settings, overview.

## Rust targets

| target | what it writes |
|---|---|
| `herdr` | `~/.config/herdr/config.toml` |
| `opencode` | `~/.config/opencode/themes/inir.json` |
| `niri` | Niri focus-ring colour |
| `umbriel` | `~/.local/state/quickshell/user/generated/umbriel-colors.toml` |
| `firefox` | Firefox-family `user.js` managed block |
| `chrome` | Chromium policy + profile colour scheme |
| `limusic` | `~/.config/limusic/matugen.css` |
| `fetch3d` | `~/.config/fetch/config` |
| `launchers` | `~/.local/bin/cmx`, `~/.local/bin/lavax` |

## Delegated (already Rust/upstream)

`terminals`, `editors`, `zed`, `spicetify`, `sddm`, `claude-code`, `gtk-kde`
call `inir-accel` or iNiR scripts that already exist. Porting them again would
duplicate upstream logic.

## Still Bash in iNiR

`cava`, `steam`, `pear-desktop` are self-contained app generators not yet ported.

## Rules for a new target

1. Read only the generated palette/state; never launch the app for theming.
2. Keep a managed block or a marked file; back up before writing.
3. `--dry-run` must report the same paths without touching them.
4. Add a Rust test for the file shape and the disabled/removal path.
