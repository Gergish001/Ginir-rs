# Patches

These are the non-Rust pieces that still need a patch in the consuming application.

- `fetch-icons-dark-label.patch` — icons and the darkened label colour for the
  **3D fetch** renderer (`fetch.c`). This is not fastfetch and does not touch it.
- `05-niri-rust-wrapper.patch` — adds the iNiR `05-niri.sh` module that calls
  `inir-theme apply --target niri` (the upstream fork's `niri.json` target
  expects this module).

The Python/Bash implementations that were replaced by the Rust crate are kept in
`../legacy/` for reference.
