#!/usr/bin/env python3
"""Generate opencode + herdr themes from the iNiR kitty palette.

opencode: darken 60%, saturation +50%
herdr:    darken 40%, saturation +20%

The kitty theme is the source of truth. The opencode theme is based on the
existing ~/.config/opencode/themes/inir.json structure, with its base colors
replaced from kitty and then transformed.

Default mode is "readable": text/accent colors get a minimum contrast against
the new background, so the theme stays usable. Use --mode all for a literal
transform of every color (may produce unreadable text).
"""
from __future__ import annotations

import argparse
import colorsys
import json
import re
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path

HOME = Path.home()
KITTY = HOME / ".config/kitty/theme.conf"
OPENCODE_TEMPLATE = HOME / ".config/opencode/themes/inir.json"
OPENCODE_OUT = HOME / ".config/opencode/themes/inir-dark60.json"
HERDR_CONFIG = HOME / ".config/herdr/config.toml"
APP_PALETTE = HOME / ".local/state/quickshell/user/generated/app-palette.json"
TERMINAL_JSON = HOME / ".local/state/quickshell/user/generated/terminal.json"
BACKUP_DIR = HOME / "gentoo-opt-backups"


def eprint(*a):
    print(*a, file=sys.stderr)


def parse_hex(s: str) -> tuple[int, int, int]:
    s = s.strip().lstrip("#")
    if len(s) == 3:
        s = "".join(c * 2 for c in s)
    return tuple(int(s[i:i + 2], 16) for i in (0, 2, 4))


def to_hex(rgb: tuple[int, int, int]) -> str:
    return "#%02x%02x%02x" % tuple(max(0, min(255, round(v))) for v in rgb)


def rgb_to_hsl(rgb):
    r, g, b = (v / 255 for v in rgb)
    return colorsys.rgb_to_hls(r, g, b)  # h, l, s


def hsl_to_rgb(h, l, s):
    r, g, b = colorsys.hls_to_rgb(h, max(0.0, min(1.0, l)), max(0.0, min(1.0, s)))
    return tuple(round(v * 255) for v in (r, g, b))


def luminance(rgb):
    def f(v):
        v /= 255
        return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4
    r, g, b = rgb
    return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)


def contrast(a, b):
    la, lb = luminance(a), luminance(b)
    hi, lo = max(la, lb), min(la, lb)
    return (hi + 0.05) / (lo + 0.05)


def mix(a, b, t):
    return tuple(round(a[i] * (1 - t) + b[i] * t) for i in range(3))


def ensure_contrast(rgb, bg, target):
    if contrast(rgb, bg) >= target:
        return rgb
    h, l, s = rgb_to_hsl(rgb)
    step = 0.01 if luminance(bg) < 0.5 else -0.01
    for _ in range(100):
        l = max(0.0, min(1.0, l + step))
        cand = hsl_to_rgb(h, l, s)
        if contrast(cand, bg) >= target:
            return cand
        if l in (0.0, 1.0):
            break
    return (255, 255, 255) if luminance(bg) < 0.5 else (0, 0, 0)


def transform(rgb, darken, sat, role="accent", bg=None, mode="readable"):
    h, l, s = rgb_to_hsl(rgb)
    l *= (1.0 - darken)
    s *= (1.0 + sat)
    out = hsl_to_rgb(h, l, s)
    if mode == "readable" and bg is not None:
        target = {"text": 4.5, "accent": 3.0, "muted": 2.5, "background": 0.0}.get(role, 0.0)
        if target:
            out = ensure_contrast(out, bg, target)
    return out


def parse_kitty(path: Path):
    colors = {}
    if not path.exists():
        raise SystemExit(f"missing kitty theme: {path}")
    for line in path.read_text(errors="ignore").splitlines():
        m = re.match(r"^\s*([A-Za-z0-9_]+)\s+(#[0-9A-Fa-f]{3,8})\s*$", line)
        if m and len(m.group(2)) in (4, 7):
            colors[m.group(1)] = m.group(2)
    if "background" not in colors or "foreground" not in colors:
        raise SystemExit("kitty theme has no background/foreground")
    return colors


def _pick(d, *keys, default=None):
    for key in keys:
        value = d.get(key)
        if isinstance(value, str) and re.fullmatch(r"#[0-9A-Fa-f]{6}", value):
            return value
    return default


def parse_inir(app_palette: Path, terminal_json: Path):
    """Build a kitty-like palette from iNiR's generated app palette + terminal.json.

    These files are written before the theming modules run, so a herdr module
    can use them without racing the kitty target.
    """
    if not app_palette.exists() or not terminal_json.exists():
        raise SystemExit(f"missing iNiR palette: {app_palette} or {terminal_json}")
    p = json.loads(app_palette.read_text())
    t = json.loads(terminal_json.read_text())

    bg = _pick(p, "surface", "background", "app_background", default="#131014")
    fg = _pick(p, "on_surface", "foreground", "app_foreground", default="#efebe4")
    accent = _pick(p, "app_accent", "primary", default="#b78cda")
    sel_bg = _pick(p, "primary_container", "surface_variant", "app_selection", default=bg)
    inactive_bg = _pick(p, "surface_container_high", "surface_container", "app_card_bg", default=bg)
    inactive_fg = _pick(p, "on_surface_variant", "app_subtext", default=fg)
    outline = _pick(p, "outline", "outline_variant", default="#847c84")

    colors = {
        "background": bg,
        "foreground": fg,
        "selection_foreground": fg,
        "selection_background": sel_bg,
        "cursor": fg,
        "cursor_text_color": bg,
        "url_color": t.get("term4", accent),
        "active_border_color": accent,
        "inactive_border_color": outline,
        "bell_border_color": t.get("term1", "#f3a1ab"),
        "active_tab_foreground": _pick(p, "on_primary", "on_accent", default=bg),
        "active_tab_background": accent,
        "inactive_tab_foreground": inactive_fg,
        "inactive_tab_background": inactive_bg,
        "tab_bar_background": bg,
    }
    for i in range(16):
        key = f"term{i}"
        if key in t:
            colors[f"color{i}"] = t[key]
    return colors


def classify_def(key: str) -> str:
    k = key.lower()
    if k.startswith("m3inverse") or k.startswith("m3onprimary"):
        return "keep"
    if "onsurface" in k:
        return "text"
    if "container" in k:
        return "background"
    if "surface" in k or "shadow" in k:
        return "background"
    if "outline" in k:
        return "muted"
    return "accent"


def classify_theme(key: str) -> str:
    k = key.lower()
    if k in ("thinkingopacity",):
        return "skip"
    if any(x in k for x in ("background", "panel", "menu", "element")) or k.endswith("bg"):
        return "background"
    if k == "text" or k.endswith("text") or "textmuted" in k:
        return "text" if k != "textmuted" else "muted"
    if "border" in k or "outline" in k or k in ("diffhunkheader", "difflinenumber"):
        return "muted"
    if "syntaxcomment" in k or "syntaxoperator" in k or "syntaxpunctuation" in k:
        return "muted"
    return "accent"


def build_opencode(kitty, template, darken, sat, mode):
    bg = transform(parse_hex(kitty["background"]), darken, sat, "background", mode=mode)
    fg = parse_hex(kitty["foreground"])
    primary = parse_hex(kitty.get("active_border_color", kitty.get("color2", "#35b28f")))
    error = parse_hex(kitty.get("color1", "#ff9f8b"))
    success = parse_hex(kitty.get("color2", "#8ac77f"))
    outline = parse_hex(kitty.get("color8", "#92938c"))
    variant = parse_hex(kitty.get("color7", "#adaea6"))
    container = parse_hex(kitty.get("inactive_tab_background", kitty["background"]))

    base = {
        "m3Surface": parse_hex(kitty["background"]),
        "m3OnSurface": fg,
        "m3Primary": primary,
        "m3Error": error,
        "m3Success": success,
        "m3Outline": outline,
        "m3OnSurfaceVariant": variant,
        "m3SurfaceContainer": container,
        "m3SurfaceContainerHigh": mix(parse_hex(kitty["background"]), fg, 0.10),
        "m3SurfaceContainerHighest": mix(parse_hex(kitty["background"]), fg, 0.14),
        "m3SurfaceContainerLow": mix(parse_hex(kitty["background"]), fg, 0.04),
        "m3SurfaceDim": transform(parse_hex(kitty["background"]), 0.15, 0.0, "background", mode="all"),
        "m3SurfaceBright": mix(parse_hex(kitty["background"]), fg, 0.06),
        "m3PrimaryContainer": mix(primary, parse_hex(kitty["background"]), 0.80),
        "m3OnPrimary": parse_hex(kitty["background"]),
        "m3OnPrimaryContainer": parse_hex(kitty.get("color10", "#9ed395")),
        "m3ErrorContainer": mix(error, parse_hex(kitty["background"]), 0.80),
        "m3SuccessContainer": mix(success, parse_hex(kitty["background"]), 0.80),
        "m3OutlineVariant": parse_hex(kitty.get("inactive_border_color", "#92938c")),
        "m3InverseSurface": fg,
        "m3InverseOnSurface": parse_hex(kitty["background"]),
        "m3InversePrimary": primary,
        "m3Shadow": (0, 0, 0),
    }
    ansi = {
        "ansiBlue": "color4", "ansiBrightBlue": "color12",
        "ansiBrightCyan": "color14", "ansiBrightGreen": "color10",
        "ansiBrightMagenta": "color13", "ansiBrightRed": "color9",
        "ansiBrightYellow": "color11", "ansiCyan": "color6",
        "ansiGreen": "color2", "ansiMagenta": "color5",
        "ansiRed": "color1", "ansiYellow": "color3",
    }
    for name, key in ansi.items():
        if key in kitty:
            base[name] = parse_hex(kitty[key])

    out = json.loads(json.dumps(template))
    defs = out.setdefault("defs", {})
    for key, value in list(defs.items()):
        if key in base:
            rgb = base[key]
        elif isinstance(value, str) and value.startswith("#"):
            rgb = parse_hex(value)
        else:
            continue
        role = classify_def(key)
        if role == "keep":
            defs[key] = to_hex(rgb)
        else:
            defs[key] = to_hex(transform(rgb, darken, sat, role, bg, mode))

    theme = out.get("theme", {})
    for key, value in list(theme.items()):
        if not isinstance(value, dict):
            continue
        dark = value.get("dark")
        if isinstance(dark, str) and dark.startswith("#"):
            role = classify_theme(key)
            if role == "skip":
                continue
            value["dark"] = to_hex(transform(parse_hex(dark), darken, sat, role, bg, mode))
        # light variant is intentionally left as-is so light mode stays usable
    return out, bg, defs


def herdr_palette(kitty, darken, sat, mode, accent_override=None, accent_exact=False):
    bg = transform(parse_hex(kitty["background"]), darken, sat, "background", mode=mode)
    roles = {
        "accent": ("active_border_color", "accent"),
        "panel_bg": ("background", "background"),
        "sidebar_bg": ("background", "background"),
        "active_row_bg": ("inactive_tab_background", "background"),
        "selection_bg": ("selection_background", "muted"),
        "surface0": ("inactive_tab_background", "background"),
        "surface1": ("selection_background", "muted"),
        "surface_dim": ("background", "background"),
        "overlay0": ("color8", "muted"),
        "overlay1": ("color7", "muted"),
        "text": ("foreground", "text"),
        "subtext0": ("color7", "text"),
        "mauve": ("color5", "accent"),
        "green": ("color2", "accent"),
        "yellow": ("color3", "accent"),
        "red": ("color1", "accent"),
        "blue": ("color4", "accent"),
        "teal": ("color6", "accent"),
        "peach": ("color11", "accent"),
    }
    out = {}
    for token, (key, role) in roles.items():
        if token == "accent" and accent_override:
            if accent_exact:
                out[token] = to_hex(parse_hex(accent_override))
            else:
                out[token] = to_hex(
                    transform(parse_hex(accent_override), darken, sat, "accent", bg, mode)
                )
            continue
        src = kitty.get(key, kitty["background"])
        out[token] = to_hex(transform(parse_hex(src), darken, sat, role, bg, mode))
    return out


def update_herdr(palette, darken, sat, apply):
    if not HERDR_CONFIG.exists():
        eprint(f"herdr config not found: {HERDR_CONFIG}")
        return
    text = HERDR_CONFIG.read_text()
    lines = text.splitlines()
    header = "[theme.custom]"
    start = None
    for i, line in enumerate(lines):
        if line.strip() == header:
            start = i
            break
    block = [header, f"# generated from iNiR/kitty palette: darken={darken:.2f}, sat=+{sat:.2f}"]
    block += [f'{k} = "{v}"' for k, v in palette.items()]
    if start is None:
        if lines and lines[-1].strip():
            lines.append("")
        lines.extend(block)
    else:
        end = len(lines)
        for j in range(start + 1, len(lines)):
            if lines[j].lstrip().startswith("["):
                end = j
                break
        lines = lines[:start] + block + ([""] if end < len(lines) else []) + lines[end:]
    new_text = "\n".join(lines).rstrip() + "\n"
    if new_text == text:
        eprint("herdr: no changes")
        return
    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    backup = BACKUP_DIR / f"herdr-config-before-theme-{datetime.now():%F-%H%M%S}.toml"
    shutil.copy2(HERDR_CONFIG, backup)
    HERDR_CONFIG.write_text(new_text)
    eprint(f"herdr: updated {HERDR_CONFIG} (backup {backup})")
    if apply:
        for cmd in (["herdr", "config", "check"], ["herdr", "server", "reload-config"]):
            p = subprocess.run(cmd, capture_output=True, text=True)
            if p.returncode != 0:
                eprint(f"{' '.join(cmd)} failed: {p.stderr.strip() or p.stdout.strip()}")


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--source", choices=("kitty", "inir"), default="kitty",
                    help="read colors from kitty theme.conf or from iNiR app-palette.json + terminal.json")
    ap.add_argument("--kitty", type=Path, default=KITTY)
    ap.add_argument("--app-palette", type=Path, default=APP_PALETTE)
    ap.add_argument("--terminal-json", type=Path, default=TERMINAL_JSON)
    ap.add_argument("--opencode-template", type=Path, default=OPENCODE_TEMPLATE)
    ap.add_argument("--opencode-out", type=Path, default=OPENCODE_OUT)
    ap.add_argument("--opencode-darken", type=float, default=0.60)
    ap.add_argument("--opencode-sat", type=float, default=0.50)
    ap.add_argument("--herdr-darken", type=float, default=0.40)
    ap.add_argument("--herdr-sat", type=float, default=0.20)
    ap.add_argument("--herdr-accent", default=None,
                    help="override herdr accent, e.g. the iNiR app_accent #b78cda")
    ap.add_argument("--herdr-accent-exact", action="store_true",
                    help="use --herdr-accent without darkening/saturation")
    ap.add_argument("--only-herdr", action="store_true", help="skip opencode theme generation")
    ap.add_argument("--mode", choices=("readable", "all"), default="readable")
    ap.add_argument("--no-herdr", action="store_true", help="do not touch herdr config")
    ap.add_argument("--no-reload", action="store_true", help="do not reload herdr")
    args = ap.parse_args()

    if args.herdr_accent and not re.fullmatch(r"#[0-9A-Fa-f]{6}", args.herdr_accent):
        raise SystemExit("--herdr-accent must be #RRGGBB")

    if args.source == "inir":
        kitty = parse_inir(args.app_palette, args.terminal_json)
    else:
        kitty = parse_kitty(args.kitty)

    if not args.only_herdr:
        template = json.loads(args.opencode_template.read_text())
        theme, bg, defs = build_opencode(kitty, template, args.opencode_darken, args.opencode_sat, args.mode)
        args.opencode_out.parent.mkdir(parents=True, exist_ok=True)
        args.opencode_out.write_text(json.dumps(theme, indent=2) + "\n")
        print(f"opencode: wrote {args.opencode_out}")

        print(f"  surface  {to_hex(bg)}  (from {kitty['background']})")
        for key in ("m3OnSurface", "m3Primary", "m3Error", "ansiGreen"):
            if key in defs:
                print(f"  {key:12s} {defs[key]}")

    palette = herdr_palette(
        kitty, args.herdr_darken, args.herdr_sat, args.mode,
        accent_override=args.herdr_accent,
        accent_exact=args.herdr_accent_exact,
    )
    print("herdr palette:")
    for k, v in palette.items():
        print(f"  {k:14s} {v}")
    if not args.no_herdr:
        update_herdr(palette, args.herdr_darken, args.herdr_sat, not args.no_reload)


if __name__ == "__main__":
    main()
