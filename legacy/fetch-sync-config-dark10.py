#!/usr/bin/env python3
"""Translate ~/.config/fastfetch/config.jsonc into ~/.config/fetch/config.

Only the module order and the display colours are carried over. The 3D fetch
keeps its own spin/animation defaults, so the look stays the stock fetch.
"""
import colorsys, json, os, re, sys

HOME = os.path.expanduser("~")
SRC = os.path.join(HOME, ".config/fastfetch/config.jsonc")
DST = os.path.join(HOME, ".config/fetch/config")

FIELDS = {
    "os", "host", "kernel", "uptime", "packages", "shell", "display", "wm",
    "displaymanager", "theme", "icons", "font", "cursor", "terminal", "cpu",
    "gpu", "memory", "swap", "disk", "ip", "battery", "powerprofile", "locale",
    "colors",
}

def strip_jsonc(text):
    out = []
    i = 0
    in_str = esc = line_comment = block_comment = False
    while i < len(text):
        c = text[i]
        n = text[i + 1] if i + 1 < len(text) else ""
        if line_comment:
            if c == "\n":
                line_comment = False
                out.append(c)
        elif block_comment:
            if c == "*" and n == "/":
                block_comment = False
                i += 1
        elif in_str:
            out.append(c)
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == '"':
                in_str = False
        else:
            if c == '"':
                in_str = True
                out.append(c)
            elif c == "/" and n == "/":
                line_comment = True
                i += 1
            elif c == "/" and n == "*":
                block_comment = True
                i += 1
            else:
                out.append(c)
        i += 1
    return "".join(out)

def load_config(path):
    try:
        raw = open(path, encoding="utf-8").read()
    except OSError:
        return None
    try:
        cleaned = re.sub(r",\s*([}\]])", r"\1", strip_jsonc(raw))
        return json.loads(cleaned)
    except json.JSONDecodeError as e:
        print(f"fetch: cannot parse {path}: {e}", file=sys.stderr)
        return None

def as_list(v):
    if v is None:
        return []
    return v if isinstance(v, list) else [v]


def darken_sgr(keys, factor=0.90):
    """Darken a truecolor SGR suffix (38;2;R;G;B) by reducing HSL lightness."""
    m = re.fullmatch(r"38;2;(\d+);(\d+);(\d+)", keys or "")
    if not m:
        return keys
    r, g, b = (int(x) / 255 for x in m.groups())
    h, l, s = colorsys.rgb_to_hls(r, g, b)
    r, g, b = colorsys.hls_to_rgb(h, l * factor, s)
    return f"38;2;{round(r * 255)};{round(g * 255)};{round(b * 255)}"

def main():
    cfg = load_config(SRC)
    if not cfg:
        return 1
    modules = cfg.get("modules") or []
    lines = []
    has_title = False
    for m in modules:
        if isinstance(m, str):
            kind = m
            opts = {}
        elif isinstance(m, dict):
            kind = m.get("type")
            opts = m
        else:
            continue
        if kind == "title":
            has_title = True
            continue
        if kind == "break":
            lines.append("---")
            continue
        if kind in ("de", "terminalfont", "media", "localip"):
            if kind == "localip":
                lines.append("ip")
            continue
        if kind == "disk":
            folders = as_list(opts.get("folders"))
            extra = []
            for f in folders:
                if not isinstance(f, str) or f in ("", "/"):
                    continue
                extra.append(f)
            if not extra:
                lines.append("disk")
            else:
                for f in extra:
                    lines.append(f"disk={f}")
            continue
        if kind == "custom":
            key = opts.get("key") or opts.get("format") or "Custom"
            lines.append(f"custom_{key}")
            continue
        if kind in FIELDS:
            lines.append(kind)
    # display colours: fastfetch stores a raw SGR suffix such as 38;2;R;G;B
    keys = (((cfg.get("display") or {}).get("color") or {}).get("keys") or "")
    settings = [
        "shading_mode=ascii",
        "height=20",
        "v_offset=4",
        "speed=1.0",
        "spin=xy",
        "size=1.0",
        "depth=1.0",
        "box=0",
        "v_alignment=top",
        "h_alignment=left",
        "separator=─",
        f"show_title={1 if has_title else 0}",
    ]
    if isinstance(keys, str) and re.fullmatch(r"[0-9;]+", keys or ""):
        settings.append(f"label_color={darken_sgr(keys, 0.90)}")
    body = [
        "# Generated from ~/.config/fastfetch/config.jsonc by",
        "# ~/.local/lib/fetch-sync-config.py. Edit the fastfetch config, then run",
        "# fetch --sync to regenerate this file.",
        "",
        *lines,
        "",
        "# stock 3D animation settings",
        *settings,
        "",
    ]
    os.makedirs(os.path.dirname(DST), exist_ok=True)
    with open(DST, "w", encoding="utf-8") as f:
        f.write("\n".join(body))
    return 0

if __name__ == "__main__":
    sys.exit(main())
