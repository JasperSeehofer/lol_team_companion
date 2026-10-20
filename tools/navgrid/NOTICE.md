# Notices

`tools/navgrid/` is licensed under the **GNU Affero General Public License,
version 3 only** (`AGPL-3.0-only`); the full text is in `LICENSE`.

Why "only": the region-layer layout and the flag and region enums follow
FrankTheBoxMonster's LoL-NGRID-converter, whose repository grants the AGPL
version 3 through its `LICENSE` file and states no "or any later version"
anywhere (no per-file headers, nothing in the README). The only "or (at your
option) any later version" wording in that repository is the unfilled template
in the licence's own appendix "How to Apply These Terms to Your New Programs",
which is not a grant by the author. So this tool takes the narrower reading.

The derived navgrid data (grids, rasters, GeoJSON) the tool produces from a
user's own install is output, not code: this licence does not cover it, and
Riot's ticket 2 decides whether any of it may be shared (see `README.md`).

## TheKillerey/MapgeoAddon (MIT)

- Repository: https://github.com/TheKillerey/MapgeoAddon
- Commit: `8d2f8bd9f57d064c47a5b9c5c7e6e55597412d76`
- Ported: from `wad_tool.py`, the WAD header and TOC reading of `parse_wad`,
  `read_entry_data` and the `ENTRY_*` storage-type codes (into
  `navgrid/wad.py`, rewritten without `bpy`, for WAD v3 only, using the
  `xxhash` package instead of `wad_links.py`'s pure-Python xxhash64); from
  `navgrid.py`, the v7 header layout, the 48-byte cell stride with the
  height at offset 0, the u16 flag array after the cells, and its flag names
  (into `navgrid/ngrid.py`, the `mapgeo` column of `FLAG_TABLE`).

```
MIT License

Copyright (c) 2026 TheKillerey

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

(The upstream LICENSE file also carries a disclaimer that the project is for
educational and modding purposes, that the rights to League of Legends belong
to Riot Games, Inc., and that it is not affiliated with Riot Games.)

## FrankTheBoxMonster/LoL-NGRID-converter (AGPL-3.0)

- Repository: https://github.com/FrankTheBoxMonster/LoL-NGRID-converter
- Commit: `92943ed2b2d5e82c86d680e69f53f247c89aefee`
- Licence: the repository's `LICENSE` (sha256
  `c685f13bb6c29676960eed800eebbb57b56811cf18ebc734275fb3692e69b39e`) is the
  unmodified GNU Affero General Public License, "Version 3, 19 November 2007".
  Its operative sentence for the version question (section 14): "If the
  Program does not specify a version number of the GNU Affero General Public
  License, you may choose any version ever published by the Free Software
  Foundation." The repository specifies version 3 (the file itself) and no
  "or any later version" grant; see above.
- Used (as facts, in an independent Python implementation, `navgrid/ngrid.py`):
  the full 48-byte v7 cell field list (`CELL_DTYPE`); the per-cell 4-byte
  region block and its nibble split (river; jungle quadrant / main region;
  nearest lane / POI; ring / SRX); the 8 x 132-byte unknown block; the height
  sample block; the 900 x (900 f32 + 2 i16) hint-node block; and the enums
  `VisionPathingFlags` (the `converter` column of `FLAG_TABLE`),
  `RiverRegionFlags`, `JungleQuadrantFlags`, `MainRegionFlags`,
  `NearestLaneFlags`, `POIFlags`, `RingFlags`, `UnknownSRXFlags` (from
  `NavGridCell.cs`, `NGridFileReader.cs`, `Program.cs`).
- Not used: the converter's converted outputs (`SR_2024/`, `.bmp`,
  `.LSGNGRID`). They were never fetched.

## Other sources (data, not code)

- CommunityDragon `map11.bin.json`, patch 16.20.8248524 (fetched 2026-10-20):
  the navgrid path names in `navgrid/wad.py` `CANDIDATE_PATHS`.
- Bayes Esports tower coordinates via Scout's `calibrate.py`
  (`origin/mission/M156-scout-replay-spike`, commit 2182417): the structure
  positions in `navgrid/structures.py`.

## Dependencies

`numpy` (BSD-3-Clause), `zstandard` (BSD-3-Clause), `xxhash` (BSD-2-Clause),
optional `shapely` (BSD-3-Clause), test-only `pytest` (MIT).
