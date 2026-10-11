# Fonts

All four families are self-hosted (no font CDN) and licensed under the SIL Open Font License 1.1. Each folder holds the licence text as `OFL.txt`.

| Family | Role | Files | Upstream | Version |
|--------|------|-------|----------|---------|
| Familjen Grotesk | display (page titles, section heads) | variable `wght` 400-700, upright | github.com/Familjen-Sthlm/Familjen-Grotesk | 2.002 |
| Public Sans | UI and text | variable `wght` 100-900, upright and italic | github.com/uswds/public-sans | 2.001 |
| JetBrains Mono | figures, coordinates, overlines, keycaps | variable `wght` 100-800, upright | github.com/JetBrains/JetBrainsMono | 2.211 |
| Source Serif 4 | surveyor's notes (annotations only) | variable `wght` 200-900, italic only | github.com/adobe-fonts/source-serif | 4.004 |

The files are the unmodified Google Fonts subsets as packaged by Fontsource (`@fontsource-variable/*` 5.3.x on npm): one `latin` and one `latin-ext` file per style, loaded with `unicode-range` so the `latin-ext` file (German, Polish, Turkish and other European letters) downloads only when a page needs it. They are byte-for-byte as distributed, which matters for Source Serif 4: its licence reserves the font name "Source", which applies to modified versions only.

Checked in each file: `tnum` (tabular figures) in Familjen Grotesk, Public Sans and Source Serif 4; JetBrains Mono is monospaced.
