# Legacy implementations

Reference copies of the scripts this crate replaces. They are **not** used by
`inir-theme`; they document the colour maths and the original iNiR integration.

- `theme-from-kitty.py` — original kitty/iNiR palette parser, transforms and
  herdr/opencode generators.
- `fetch-sync-config-dark10.py` — patched iNiR fastfetch -> 3D fetch config
  generator with the label colour darkened by 10%.
- `95-herdr-python.sh` — the iNiR herdr module before it called the Rust binary.

Fastfetch, btop and yazi are intentionally not modified by this project.
