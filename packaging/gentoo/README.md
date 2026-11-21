# Gentoo notes

`ginir-theme` itself is a plain Rust binary:

```sh
cargo build --release
install -m755 target/release/ginir-theme ~/.local/bin/ginir-theme
```

For the full iNiR stack a Gentoo overlay is needed. The intended ebuilds are:

| Package | Source | Notes |
|---|---|---|
| `gui-apps/qs-rs` | `snowarch/qs-rs` | Rust + Qt 6/LayerShellQt, or the prebuilt Qt bundle |
| `dev-util/inir-accel` | `snowarch/inir-accel` | Rust; optional `inir-qml-plugin` |
| `dev-util/inir-cli` | `snowarch/inir-cli` | Go |
| `gui-apps/inir` | `snowarch/iNiR` | QML payload, targets, runit/OpenRC user service |
| `dev-util/ginir-theme` | this repo | Rust; no Qt |

The hard part is qs-rs: it needs Qt 6 (the upstream bundle uses 6.12) and
LayerShellQt, or the prebuilt bundle. iNiR itself currently supports Arch,
Fedora, Debian, Ubuntu, Void and NixOS; Gentoo needs its own service and paths.
