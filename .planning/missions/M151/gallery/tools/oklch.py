#!/usr/bin/env python3
"""oklch.py: OKLCH -> sRGB hex and WCAG 2.x contrast, standard library only.

  python3 -I oklch.py hex 0.180 0.018 200            # -> #0b1517 (gamut-clipped)
  python3 -I oklch.py pairs FILE.css                  # contrast table for the
      pairs listed in FILE.css as `/* pair: fg-token on bg-token, need N */`
      against the oklch() custom properties defined in the same file.

Alpha colours are composited over the `page` token of the same file.
"""
import math
import re
import sys


def oklch_to_linear(l, c, h):
    a, b = c * math.cos(math.radians(h)), c * math.sin(math.radians(h))
    l_ = l + 0.3963377774 * a + 0.2158037573 * b
    m_ = l - 0.1055613458 * a - 0.0638541728 * b
    s_ = l - 0.0894841775 * a - 1.2914855480 * b
    l3, m3, s3 = l_ ** 3, m_ ** 3, s_ ** 3
    r = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3
    g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3
    bl = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3
    return [min(1.0, max(0.0, v)) for v in (r, g, bl)]


def enc(v):
    return 12.92 * v if v <= 0.0031308 else 1.055 * v ** (1 / 2.4) - 0.055


def dec(v):
    return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4


def to_hex(lin):
    return "#" + "".join(f"{round(enc(v) * 255):02x}" for v in lin)


def lum(lin):
    # WCAG uses the 8-bit-rounded sRGB value; round-trip through it.
    r, g, b = (dec(round(enc(v) * 255) / 255) for v in lin)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def ratio(a, b):
    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


TOKEN = re.compile(r"--color-([\w-]+)\s*:\s*oklch\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)\s*(?:/\s*([\d.]+))?\s*\)")
PAIR = re.compile(r"pair:\s*([\w-]+)\s+on\s+([\w-]+),\s*need\s*([\d.]+)\s*(?:\(([^)]*)\))?")


def load(path):
    text = open(path, encoding="utf-8").read()
    raw = {m[1]: (float(m[2]), float(m[3]), float(m[4]), float(m[5]) if m[5] else 1.0) for m in TOKEN.finditer(text)}
    page = oklch_to_linear(*raw["page"][:3])
    col = {}
    for k, (l, c, h, a) in raw.items():
        lin = oklch_to_linear(l, c, h)
        if a < 1:  # composite in sRGB-encoded space, as browsers do
            lin = [dec(enc(x) * a + enc(p) * (1 - a)) for x, p in zip(lin, page)]
        col[k] = lin
    return text, raw, col


def main():
    if len(sys.argv) >= 5 and sys.argv[1] == "hex":
        print(to_hex(oklch_to_linear(*map(float, sys.argv[2:5]))))
        return
    if len(sys.argv) == 3 and sys.argv[1] == "pairs":
        text, raw, col = load(sys.argv[2])
        print("| Foreground | Background | Hex fg / bg | Ratio | Needs | Use | Result |")
        print("|---|---|---|---:|---:|---|---|")
        bad = 0
        for fg, bg, need, use in PAIR.findall(text):
            r = ratio(col[fg], col[bg])
            ok = r >= float(need)
            bad += not ok
            print(f"| `{fg}` | `{bg}` | `{to_hex(col[fg])}` / `{to_hex(col[bg])}` | {r:.2f} | {need} | {use} | {'pass' if ok else 'FAIL'} |")
        sys.exit(1 if bad else 0)
    print(__doc__)
    sys.exit(2)


if __name__ == "__main__":
    main()
