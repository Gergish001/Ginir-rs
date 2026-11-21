# Translating the README

The README uses a small language row under the badges:

```html
<a href="README.md">English</a> ·
<a href="docs/readme/README.ru.md">Русский</a> ·
<a href="docs/readme/TRANSLATING.md">help translate</a>
```

## Add a language

1. Copy `README.md` to `docs/readme/README.<code>.md`
   (for example `README.es.md`, `README.zh.md`).
2. Translate the prose. Keep code blocks, commands, paths, target ids and table
   ids unchanged.
3. Fix the relative links:
   - `docs/images/readme/mark.svg` → `../images/readme/mark.svg`
   - `LICENSE` → `../../LICENSE`
   - `rust-toolchain.toml` → `../../rust-toolchain.toml`
   - `upstream/inir/` → `../../upstream/inir/`
   - `crates/...`, `integration/...`, `packaging/...` → prefix `../../`
   - the language row's English link → `../../README.md`
4. Keep the explicit anchors on the section headings (`#targets`,
   `#build-and-install`, `#usage`, `#inir-integration`) so the link row keeps
   working in every language.
5. Add the new language to the row in `README.md` and in every existing
   translation.

## Current translations

| Language | File |
|---|---|
| English | `README.md` |
| Русский | `docs/readme/README.ru.md` |

If a translation falls behind, a short note in the translation pointing at the
English README is better than leaving stale commands.
