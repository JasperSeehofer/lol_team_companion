# Hand-drawn marks: from reMarkable to `public/marks/`

`marks-template.pdf` (reMarkable 2 size, 1404 x 1872 px at 226 dpi) holds nine 16-unit grid cells and one Rift cell, all in one light grey. Rebuild it with `python3 .planning/missions/M151/remarkable/build-template.py`.

1. Copy the PDF to the tablet (desktop app or my.remarkable.com) and draw with the Fineliner, black, one size. Stay inside the dashed live area (1 to 15); use the keylines (circle d12, square 11, diamond, triangle) as guides.
2. Export the page as **vector** (Share > SVG, or PDF; never PNG).
3. Drop the grey template layer: in the SVG keep only the black paths (the template lines are grey `0.8`, the strokes black), or delete the template group in Inkscape.
4. Crop each cell: set the `viewBox` to the cell's 16-unit grid (the cell's outer grid square), so the grid maps to `0 0 16 16`.
5. Clean the paths: the tablet writes many short segments. Simplify (Inkscape Path > Simplify, small threshold), then snap the points to whole or half units where the drawing allows. Keep round ends.
6. Normalise the stroke: `stroke-width="1.5"` in the 16 grid, `fill="none"`, `stroke="currentColor"`, `stroke-linecap="round"`, `stroke-linejoin="round"`; remove every colour, transform and `style` attribute.
7. Add `<title>` with the word (Top, Jungle, Mid, Bot, Support, Dragon, Baron, Herald, Tower).
8. Write `public/marks/<name>.svg` (width/height 16) and `<name>-24.svg` (width/height 24, `stroke-width="1.33333"` = 2 px).
9. Put the same path data into `MARKS` in `src/pages/design_marks.rs` and into the table in `.planning/missions/M151/gallery/tools/build-marks.py`; the unit test `mark_files_match_the_table` fails until all three agree.
10. Check at 16 px on the contact sheet (`gallery/marks.html`, rebuild with `build-marks.py`): beside its word, against the other eight for weight, and against the grid spec rules (no corner L, no diagonal lane bar, no wings, helmet, antlers or claw).
