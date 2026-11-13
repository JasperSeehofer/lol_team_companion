# C paper: contrast of the proposed tokens

`paper-tokens.css` remaps the Survey token names under `.paper` (DESIGN 3.2 form). Grounds are neutral-cool (hue 195-200, chroma 0.004 to 0.008): no cream, parchment or sepia. Ratios are WCAG 2.x on the 8-bit sRGB values (OKLCH converted and gamut-clipped by `../tools/oklch.py`; the alpha `accent-soft` is composited over `page`). AA: 4.5 for text, 3 for large text, control edges and marks. Regenerate with `python3 -I G/tools/oklch.py pairs G/c/paper-tokens.css` (exit 1 on a failure).

| Foreground | Background | Hex fg / bg | Ratio | Needs | Use | Result |
|---|---|---|---:|---:|---|---|
| `primary` | `page` | `#141c1d` / `#f7fbfb` | 16.60 | 4.5 | body text | pass |
| `primary` | `ink-1` | `#141c1d` / `#f1f5f6` | 15.76 | 4.5 | text on raised surface | pass |
| `primary` | `ink-3` | `#141c1d` / `#dfe6e7` | 13.69 | 4.5 | text on the highest step | pass |
| `secondary` | `page` | `#3d4748` / `#f7fbfb` | 9.19 | 4.5 | secondary text, marks beside labels | pass |
| `secondary` | `ink-2` | `#3d4748` / `#e9efef` | 8.23 | 4.5 | secondary text on a monogram | pass |
| `muted` | `page` | `#586363` / `#f7fbfb` | 5.96 | 4.5 | captions, meta | pass |
| `muted` | `ink-1` | `#586363` / `#f1f5f6` | 5.66 | 4.5 | captions on raised surface | pass |
| `muted` | `ink-3` | `#586363` / `#dfe6e7` | 4.91 | 4.5 | captions on the highest step | pass |
| `outline` | `page` | `#6b7778` / `#f7fbfb` | 4.44 | 3 | control edge | pass |
| `outline` | `ink-2` | `#6b7778` / `#e9efef` | 3.98 | 3 | control edge on a field | pass |
| `rule` | `page` | `#cbd3d3` / `#f7fbfb` | 1.46 | 1.2 | hairline, decorative | pass |
| `accent` | `page` | `#5d5b0a` / `#f7fbfb` | 6.79 | 4.5 | chalk as text: map label | pass |
| `accent` | `ink-1` | `#5d5b0a` / `#f1f5f6` | 6.45 | 4.5 | chalk as text on raised surface | pass |
| `accent-ink` | `accent` | `#f7fbfb` / `#5d5b0a` | 6.79 | 4.5 | primary button label | pass |
| `accent-ink` | `accent-hover` | `#f7fbfb` / `#4c4b02` | 8.70 | 4.5 | primary button hovered | pass |
| `primary` | `accent-soft` | `#141c1d` / `#e8ebe3` | 14.36 | 4.5 | text on a selected row | pass |
| `side-blue` | `page` | `#0a71b1` / `#f7fbfb` | 5.02 | 3 | side mark and line | pass |
| `side-red` | `page` | `#b6443a` / `#f7fbfb` | 5.20 | 3 | side mark and line | pass |
| `side-blue` | `ink-3` | `#0a71b1` / `#dfe6e7` | 4.14 | 3 | side mark on the highest step | pass |
| `side-red` | `ink-3` | `#b6443a` / `#dfe6e7` | 4.28 | 3 | side mark on the highest step | pass |
| `primary` | `side-blue-deep` | `#141c1d` / `#dbeaf8` | 14.12 | 4.5 | side badge text | pass |
| `primary` | `side-red-deep` | `#141c1d` / `#f8e2df` | 13.95 | 4.5 | side badge text | pass |
| `win` | `page` | `#026e5b` / `#f7fbfb` | 5.95 | 3 | win mark and border | pass |
| `win-ink` | `win` | `#f7fbfb` / `#026e5b` | 5.95 | 4.5 | win badge text | pass |
| `loss` | `page` | `#586363` / `#f7fbfb` | 5.96 | 3 | loss mark and border | pass |
| `success` | `page` | `#156f41` / `#f7fbfb` | 5.96 | 4.5 | status text | pass |
| `warning` | `page` | `#8d5901` / `#f7fbfb` | 5.66 | 4.5 | status text | pass |
| `danger` | `page` | `#af2938` / `#f7fbfb` | 6.30 | 4.5 | status text | pass |
| `info` | `page` | `#126686` / `#f7fbfb` | 6.16 | 4.5 | status text | pass |
| `accent-ink` | `danger-deep` | `#f7fbfb` / `#8d182c` | 8.76 | 4.5 | danger button hovered; on paper the label must be paper, not primary | pass |

All 30 pairs pass. Notes for a real paper theme:

- **Chalk.** Pale chalk (dark theme `#e7e798`) is 1.2:1 on paper, so the spot colour becomes a deep olive (`#5d5b0a`, 6.8:1) with paper text on it for the primary button. It still reads as the chalk's hue, not as warning amber.
- **Danger hover.** The dark theme hovers the danger button to `danger-deep` with `primary` text; on paper `primary` on `danger-deep` is near-black on deep red and fails, so the label must switch to `accent-ink` (paper), 8.8:1. That is a rule change, not only a token remap.
- **Sides and results** keep their meaning: blue 5.0:1, red 5.2:1, win 6.0:1 as marks on `page`, still 4.1:1 or more on `ink-3`; `-deep` becomes a pale tint behind primary text (14:1). Loss stays the muted grey, never red.
- **Rule** is a decorative hairline (1.5:1); everything that bounds a control uses `outline` (4.4:1).
