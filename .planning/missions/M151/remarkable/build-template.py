#!/usr/bin/env python3
"""Write marks-template.pdf: a reMarkable 2 drawing template for the nine marks.

Page: 1404 x 1872 px at 226 dpi = 447.29 x 596.39 pt. Everything is drawn in
one light grey (0.80) so the owner's black strokes separate cleanly on export.
Nine cells, each a 16-unit grid with the live area (1 to 15) and the keylines
of references.md "Mark grid spec" dashed, plus one larger Rift cell (100-unit
square, the river diagonal hinted). Python stdlib only; raw PDF 1.4.

Run: python3 .planning/missions/M151/remarkable/build-template.py
"""
import math
import os

W, H = 1404 / 226 * 72, 1872 / 226 * 72
GREY = "0.8 G 0.8 g"
MARGIN, GAP, COLS = 20.0, 10.0, 4
CELL = (W - 2 * MARGIN - (COLS - 1) * GAP) / COLS  # ~96.3 pt
LABEL = 11.0  # room under each grid for its name
NAMES = ["Top", "Jungle", "Mid", "Bot", "Support", "Dragon", "Baron", "Herald", "Tower"]

ops = [GREY]


def line(x1, y1, x2, y2, w, dash=None):
    ops.append(f"{w} w {('[' + ' '.join(map(str, dash)) + '] 0 d') if dash else '[] 0 d'} "
               f"{x1:.2f} {y1:.2f} m {x2:.2f} {y2:.2f} l S")


def rect(x, y, w, h, lw, dash=None):
    ops.append(f"{lw} w {('[' + ' '.join(map(str, dash)) + '] 0 d') if dash else '[] 0 d'} "
               f"{x:.2f} {y:.2f} {w:.2f} {h:.2f} re S")


def poly(pts, lw, dash=None):
    d = ('[' + ' '.join(map(str, dash)) + '] 0 d') if dash else '[] 0 d'
    p = " ".join(f"{x:.2f} {y:.2f} {'m' if i == 0 else 'l'}" for i, (x, y) in enumerate(pts))
    ops.append(f"{lw} w {d} {p} h S")


def circle(cx, cy, r, lw, dash=None):
    k = 0.5523 * r
    d = ('[' + ' '.join(map(str, dash)) + '] 0 d') if dash else '[] 0 d'
    ops.append(
        f"{lw} w {d} {cx + r:.2f} {cy:.2f} m "
        f"{cx + r:.2f} {cy + k:.2f} {cx + k:.2f} {cy + r:.2f} {cx:.2f} {cy + r:.2f} c "
        f"{cx - k:.2f} {cy + r:.2f} {cx - r:.2f} {cy + k:.2f} {cx - r:.2f} {cy:.2f} c "
        f"{cx - r:.2f} {cy - k:.2f} {cx - k:.2f} {cy - r:.2f} {cx:.2f} {cy - r:.2f} c "
        f"{cx + k:.2f} {cy - r:.2f} {cx + r:.2f} {cy - k:.2f} {cx + r:.2f} {cy:.2f} c S")


def text(x, y, size, s):
    s = s.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)")
    ops.append(f"BT /F1 {size} Tf {x:.2f} {y:.2f} Td ({s}) Tj ET")


def mark_cell(x, top, name):
    """A 16-unit grid whose top-left corner is (x, top) in page coordinates."""
    u = CELL / 16
    gy = top - CELL  # bottom of the grid
    P = lambda gx, gyu: (x + gx * u, top - gyu * u)  # grid units, y down
    for i in range(17):
        lw = 0.6 if i in (0, 8, 16) else 0.3
        line(x + i * u, gy, x + i * u, top, lw)
        line(x, top - i * u, x + CELL, top - i * u, lw)
    rect(*P(1, 15), 14 * u, 14 * u, 0.6, [2, 2])                      # live area
    circle(*P(8, 8), 6 * u, 0.5, [3, 2])                               # circle d12
    rect(*P(2.5, 13.5), 11 * u, 11 * u, 0.5, [3, 2])                   # square 11
    poly([P(8, 2), P(14, 8), P(8, 14), P(2, 8)], 0.5, [3, 2])          # diamond
    poly([P(2, 13.25), P(8, 2.75), P(14, 13.25)], 0.5, [1, 2])         # triangle
    text(x, gy - LABEL + 2, 8, name)


# Title
top = H - MARGIN
text(MARGIN, top - 10, 10, "Team Companion marks: draw inside the dashed live area, one stroke weight")
top -= 22

# Rift cell: 2 x 2 cells at the top left
rift = 2 * CELL + GAP
rx, ry = MARGIN, top - rift
rect(rx, ry, rift, rift, 0.8)
for i in range(1, 10):  # 10-unit grid, faint
    line(rx + i * rift / 10, ry, rx + i * rift / 10, ry + rift, 0.25, [1, 3])
    line(rx, ry + i * rift / 10, rx + rift, ry + i * rift / 10, 0.25, [1, 3])
s = rift / 100
off = 1.5 * math.sqrt(2) * s  # two banks, 3 units apart
for o in (-off, off):  # river from (8,8) towards (92,92), y down
    line(rx + 8 * s + o, ry + rift - 8 * s + o, rx + 92 * s + o, ry + rift - 92 * s + o, 0.6, [4, 4])
text(rx, ry - LABEL + 2, 8, "Rift  (100 x 100, river hinted)")

# Nine mark cells: two beside the Rift, then rows of four
slots = [(2, 0), (3, 0), (2, 1), (3, 1), (0, 2), (1, 2), (2, 2), (3, 2), (0, 3)]
row_h = CELL + LABEL + GAP
for name, (c, r) in zip(NAMES, slots):
    cx = MARGIN + c * (CELL + GAP)
    ctop = top - r * row_h
    mark_cell(cx, ctop, name)

# Notes in the free space of the last row
nx = MARGIN + CELL + GAP
ny = top - 3 * row_h - 12
for i, s in enumerate([
    "Keylines (dashed): circle d12, square 11, diamond, triangle.",
    "Live area 1 to 15. Round ends. Stroke only, no fills.",
    "Motif: surveying instruments and map symbols.",
    "Never a corner L, a diagonal lane bar, wings, a helmet,",
    "antlers or a claw. Export as vector: see IMPORT.md.",
]):
    text(nx, ny - i * 12, 8, s)

content = "\n".join(ops).encode("latin-1")
objs = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {W:.2f} {H:.2f}] "
    f"/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>".encode(),
    b"<< /Length %d >>\nstream\n" % len(content) + content + b"\nendstream",
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
]
out = bytearray(b"%PDF-1.4\n")
offsets = []
for i, o in enumerate(objs, 1):
    offsets.append(len(out))
    out += b"%d 0 obj\n" % i + o + b"\nendobj\n"
xref = len(out)
out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objs) + 1)
out += b"".join(b"%010d 00000 n \n" % off for off in offsets)
out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objs) + 1, xref)
path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "marks-template.pdf")
with open(path, "wb") as f:
    f.write(out)
print(f"wrote {path} ({W:.2f} x {H:.2f} pt)")
