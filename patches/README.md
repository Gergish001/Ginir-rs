# Patches

Everything in `../integration/inir/` is a Ginir-rs addition to iNiR: target
manifests and the color modules that call the Rust `ginir-theme` binary. The
`upstream/inir/` tree itself is left untouched.

- `fetch-icons-dark-label.patch` — icons and the darkened label colour for the
  **3D fetch** renderer (`fetch.c`). This is not fastfetch and does not touch it.

The Python/Bash implementations that were replaced by the Rust crate are kept in
`../legacy/` for reference. fastfetch, btop and yazi are intentionally unchanged.
