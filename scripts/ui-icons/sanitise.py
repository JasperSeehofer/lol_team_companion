#!/usr/bin/env python3
"""Sanitise an upstream SVG icon for the app's /img/ui/ route (M152).

Usage: python3 -I scripts/ui-icons/sanitise.py IN.svg OUT.svg

OUT is the committed copy, assets/ui/position/NAME.svg (never under
public/, which is served as plain static files without the route's CSP).

Keeps only the elements svg, g, path, circle and rect, and only the
attributes fill, fill-rule, opacity, stroke, d, cx, cy, r, x, y, width,
height, viewBox and transform. Everything else is dropped: processing
instructions (the upstream glow.css stylesheet link), scripts, styles,
ids, classes, foreignObject, xlink and every href, and event handlers.
A <polygon> is not kept as such: its points become an equivalent <path>
(M ... Z), since the upstream position icons use both. Every fill and
stroke colour becomes currentColor, so the icon is recoloured by tokens
(as a CSS mask or inline) and never shows its client colours.

Standard library only, so the committed copies are reproducible.
"""

import re
import sys
import xml.etree.ElementTree as ET

SVG_NS = "http://www.w3.org/2000/svg"
KEEP = {"svg", "g", "path", "circle", "rect"}
ATTRS = {
    "fill", "fill-rule", "opacity", "stroke", "d", "cx", "cy", "r",
    "x", "y", "width", "height", "viewBox", "transform",
}
NUMBER = re.compile(r"^-?[0-9.]+$")
SAFE_VALUE = re.compile(r"^[A-Za-z0-9 ,.\-()#]*$")


def local(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def polygon_to_d(points: str) -> str:
    nums = [n for n in re.split(r"[\s,]+", points.strip()) if n]
    if len(nums) < 4 or len(nums) % 2 or not all(NUMBER.match(n) for n in nums):
        raise ValueError(f"bad polygon points: {points!r}")
    pairs = [f"{nums[i]} {nums[i + 1]}" for i in range(0, len(nums), 2)]
    return "M" + pairs[0] + "L" + "L".join(pairs[1:]) + "Z"


def clean(el: ET.Element) -> ET.Element | None:
    name = local(el.tag)
    attrib = dict(el.attrib)
    if name == "polygon":
        name = "path"
        attrib["d"] = polygon_to_d(attrib.pop("points", ""))
    if name not in KEEP:
        return None
    out = ET.Element(name)
    for k, v in attrib.items():
        if "}" in k or k not in ATTRS:
            continue  # namespaced (xlink:href), id, class, style, on*, ...
        if not SAFE_VALUE.match(v):
            continue  # url(...) references and anything odd are dropped
        if "url" in v.lower():
            continue
        if k in ("fill", "stroke") and v.strip().lower() != "none":
            v = "currentColor"
        out.set(k, v)
    if name == "svg":
        out.set("xmlns", SVG_NS)
    for child in el:
        c = clean(child)
        if c is not None:
            out.append(c)
    return out


def sanitise(data: bytes) -> bytes:
    # ElementTree never fetches external entities or DTDs; processing
    # instructions and comments are skipped by the default parser.
    root = ET.fromstring(data)
    if local(root.tag) != "svg":
        raise ValueError("root element is not <svg>")
    out = clean(root)
    ET.register_namespace("", SVG_NS)
    text = ET.tostring(out, encoding="unicode", short_empty_elements=True)
    # ElementTree would write the namespace twice if we left both.
    text = text.replace(f' xmlns:ns0="{SVG_NS}"', "")
    return (text + "\n").encode("utf-8")


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    with open(sys.argv[1], "rb") as f:
        data = f.read()
    result = sanitise(data)
    lowered = result.lower()
    for bad in (b"<script", b"href", b"style", b"foreignobject", b" on", b"url("):
        if bad in lowered:
            raise SystemExit(f"sanitised output still contains {bad!r}")
    with open(sys.argv[2], "wb") as f:
        f.write(result)
    return 0


if __name__ == "__main__":
    sys.exit(main())
