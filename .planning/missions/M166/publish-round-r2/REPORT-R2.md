# M166 — REPORT-R2: the navgrid tool on the real 16.21 grid

Date: 2026-10-27 (analysis run 2026-10-26). Branch: `mission/M166-scout-half-c-and-navgrid-prep`, pushed at
`a66c063` (the 5 R2 code commits), plus this report and the card.

**Inputs.** The input is the grid from Jasper's PC run on 2026-10-22:

- `aipath_srx_2.aimesh_ngrid`, 9,361,843 bytes, sha256 `b7551b91…bd93`;
- client 16.21.829.8620;
- the WAD entry was found by its candidate path (zstd, 1,954,053 → 9,361,843 bytes);
- the Match-V5 timeline of his game EUW1_8021305006.

All analysis ran in `~/.local/share/navgrid/r2/`, outside every git work tree. The scripts, outputs and
`R2-FACTS.md` (the full numbers) are there; the folder is local and can be deleted.

**Rulings that shaped R2.** Jasper decided on 2026-10-21:

- extract the grid with the Scout run;
- merge the tool only after this real-file check;
- keep the half-C thresholds as proposed;
- never put Scout run data in git.

**What this report carries.** Numbers and the tool's text output only. No grid, raster, GeoJSON, image,
polygon coordinates or cell dump is in any branch, in this report or on the card. Riot ticket 2 is still pending.

## Budget

- **R2 cost.**
  - The real-file executor used about 0.19 M subagent tokens.
  - The first report writer was cut off by the meter cap on 2026-10-26 at 22:27, before it wrote anything.
  - This finish-only resume adds about 0.1 M.
  - R2 is therefore about 0.3–0.4 M against the page's R2 estimate of about 0.6 M.
  - `76-meter.sh` is authoritative; the overseer raised the cap to 7.5 M for this finish.
- **Host.**
  - At load 55–60 the host was swapping: one pytest run took 42 min and one vectorise attempt stalled.
  - Both re-ran in seconds once the load fell below 20.
  - No cargo command ran.

## Least-confident decisions

1. **Walkability rule.** It is set to "none of 0x02|0x04" because of the SRX terrain codes and the bit pattern,
   not because of movement.
   - The two candidate rules differ only on the 136 base-gate cells.
   - No player position in the 340 per-minute samples landed on a gate cell, so the positions cannot separate
     them.
2. **0x04 "structure" vs MapgeoAddon's "BUSY".** The numbers fix where the bit sits (every turret, inhibitor,
   nexus and fountain footprint), but not which upstream name is right. Both names describe the same cells.
3. **POI 12 = Baron pit.** This rests on one Baron kill (28 units from the patch centroid) plus the river
   layer's BaronPit bit. POI 11 (four patches of 32–37 cells) has no known meaning.
4. **Transparent layer area.** It keeps only 0.77 of its cell area, because 1,432 of its 1,456 pieces are
   single cells and each keeps about 0.70 after smoothing and simplification.
   - The boundary error is still 9.4 units (under 0.2 cell).
   - I left it unchanged. A per-polygon tolerance cap would change every layer's output, and no test needs it.
5. **Still open:** bits 0x80 and 0x1000, and the 1,056-byte unknown block.
6. **Nexus and fountain-structure centres** are estimates from the grid, not reference positions.

## (1) Parse

The file parses completely: every byte is accounted for, there are 0 trailing bytes and `parse_report` is
empty.

- **Header.** v7.1, cell 50.0, counts 295 × 296 = 87,320 cells. Bounds min (-1.10, -71.24, 32.76) and max
  (14718.40, 184.52, 14792.21), with y as height. Each cell's x/z fields equal its indices.
- **Byte budget.** 39 + 48n + 2n + 4n + 1,056 + (16 + 591·593·4) + 900·(4·900+4) = 9,361,843.
- **Layers.** header, cells, flags, regions (river, jungle quadrant, main region, nearest lane, POI, ring, SRX),
  unknown_block, height_samples, hint_nodes.
- **Height samples.** 591 × 593 = (2·295+1) × (2·296+1), which confirms R1's assumption.
- **Hint nodes.** 900, which confirms the N·(4N+4) inference.
- **Unknown block.** 1,056 bytes, 282 of them non-zero, and not floats. Its meaning is open.
- **Flag histogram** (cells per bit):

  | Bit | Cells |
  |---|---|
  | 0x0001 | 2,129 |
  | 0x0002 | 33,460 |
  | 0x0004 | 1,092 |
  | 0x0040 | 2,911 |
  | 0x0080 | 23,023 |
  | 0x0400 | 70 |
  | 0x0800 | 66 |
  | 0x1000 | 76 |
  | no bits | 34,136 |

  Bits 0x0008, 0x0010, 0x0020, 0x0100, 0x0200 and above are unused. There are 19 distinct flag values.
- **POI layer.** It uses values 11 and 12, which are not in the converter's enum. The converter's BaronPit
  value (3) has 0 cells.

## (2) Flag bits: settled on the real file

The strongest single test is the flag × SRX (the converter's terrain nibble) cross-tab:

| Flag value | SRX reading | Cells |
|---|---|---|
| 0x0 | Walkable | 34,136/34,136 |
| 0x1, 0x81 | Brush | 1,878/1,878 |
| 0x3, 0x83 | BrushWall | 251/251 |
| 0x2, 0x82 | Wall | 30,431/30,434 |
| 0x42, 0xc2 | TransparentWall | 1,681/1,683 |
| 0x46 | **Walkable** | 1,092/1,092 |
| 0x440 | BlueTeamOnly | 32/32 |
| 0x840 | RedTeamOnly | 28/28 |

**Structure references.** These are the 28 Bayes structures (22 turrets and 6 inhibitors) and the timeline's
81 structure events: 17 BUILDING_KILL, all within 17.8 units of the table, and 64 TURRET_PLATE_DESTROYED.

| Bit | Plan | MapgeoAddon | Converter | Structures with the bit within 1 / 2 / 3 cells | Bit cells within 300 / 600 units of a structure | Clusters | **Settled reading** |
|---|---|---|---|---|---|---|---|
| 0x01 | brush | HAS_GRASS | Brush | 0 / 0 / 0 of 28 | 0.000 / 0.000 | 40 (20–124 cells) | **brush** (all agree) |
| 0x02 | wall | NOT_PASSABLE | Wall | 28 / 28 / 28 | 0.025 / 0.068 | 87 | **not passable, structures included** (all agree) |
| 0x04 | structure wall | BUSY | StructureWall | 28 / 28 / 28 | 0.686 / 0.836 | 32: 28 at the table (centroid ≤14.2 units, median 7.5), 2 nexus, 2 fountain | **structure footprint** (plan/converter); always appears as 0x46 |
| 0x40 | transparent wall | SEE_THROUGH | TransparentWall | 28/28 (via 0x46) | 0.265 / 0.333 | 888 | **see-through** (MapgeoAddon); a transparent wall is 0x02\|0x40 without 0x04 |
| 0x80 | – | vision-related | Unknown128 ("sporadic") | 0 / 0 / 12 | 0.029 / 0.126 | 241 | **open**; it is not noise (neighbour agreement 0.849 against a 0.264 base rate; 80 % of river cells) |
| 0x400 | gate (blue) | – | BlueTeamOnly | 0 | 0 / 0 | 2, about 4,766 units from the blue fountain | **blue-only gate** |
| 0x800 | gate (red) | – | RedTeamOnly | 0 | 0 / 0 | 2, about 4,825 units from the red fountain | **red-only gate** |
| 0x1000 | gate vision | – | NeutralZoneVisibility | 0 | 0 / 0 | 4, on gate cells only | **open** |

**Who wins per bit.**

- 0x01 and 0x02: all three readings agree.
- 0x04: the plan's "structure wall" reading holds. MapgeoAddon's "BUSY" names the same cells.
- 0x40: MapgeoAddon's see-through reading wins over the plan's "transparent wall = 64". On its own, 0x40 marks
  a wall only together with 0x02.
- 0x400 and 0x800: the plan's and the converter's gate readings hold.
- 0x80 and 0x1000 stay unnamed.

**Nexus centres (estimates only).** These are the centroids of the 0x04 clusters nearest each base corner:
blue about (1549, 1658), red about (13240, 13235).

**Lane nibble and axis.**

- With x↔x and z↔y, the lane nibble matches 24/24 lane structures in the table, 17/17 timeline kills, and 6/6
  outer turrets on their lane's main region. Nexus turrets read as mid.
- With the axes swapped, those become 8/24, 7/17 and 2/6, so the axis convention is confirmed.

**Walkability rules compared.**

- R1's "none of 0x46" gives 53,724 walkable cells. MapgeoAddon's "not 0x02" gives 53,860. They differ only on
  the 136 gate cells.
- Both fountains are walkable under both rules.
- The 96 non-structure event positions are all on walkable cells under both rules.
- Of the 340 per-minute participant positions, 333 (0.979) are on walkable cells and 337 (0.991) within 1 cell,
  under both rules.
- No participant position lands on a gate cell.
- Settled as "none of 0x02|0x04" (see least-confident 1).

## (3) Vectorise and validate

**Vectorise.** `python -m navgrid vectorise <grid> --out ~/.local/share/navgrid/r2/out2`, after the fixes:

- 6.99 s, peak RSS 312,728 KB.
- No layer has an invalid polygon (shapely `is_valid` on every polygon), and every ring is closed.
- Hole counts equal the raw cell boundary's on every layer.

| Layer | Cells | Polygons (macro / vision) | Holes | Area kept (macro / vision) | Hausdorff, vision (units) |
|---|---|---|---|---|---|
| walkable | 53,860 | 1 / 1 | 86 (raw 86) | 0.999 / 1.001 | 9.4 |
| wall | 33,460 | 87 / 87 | 1 (raw 1) | 0.999 / 0.995 | 9.4 |
| brush | 2,129 | 40 / 40 | 0 | **0.993 / 0.981** | 9.4 |
| transparent | 1,683 | 1,456 / 1,456 | 0 | 0.771 / 0.768 | 9.4 |
| structures | 1,092 | 32 / 32 | 0 | 1.022 / 0.983 | 9.4 |
| gates | 136 | 4 / 4 | 0 | 0.960 / 0.958 | 9.4 |
| river | 3,030 | 2 / 2 | 0 | 0.999 / 1.000 | 9.4 |
| lanes | 26,672 | 3 / 3 | 0 | 0.997 / 0.998 | 9.4 |
| camps | 1,309 | 12 / 12 | 0 | 1.007 / 0.993 | 9.4 |
| pits | 700 | 2 / 2 | 0 | 1.007 / 0.998 | 9.3 |

The brush area kept is inside the 5 % bound at both levels. The Hausdorff distance is far inside the
one-cell (50 unit) bound.

**Validate.** `python -m navgrid validate <grid> --timeline <timeline>`: exit 0 in 0.21 s, `pass: true`.

- Structures: 28/28, with a maximum distance of 0.28 cells and lane match 28/28.
- Fountains walkable: 2/2.
- Structure-wall hypothesis: supported. 28/28 structures have bit 4 within 2 cells, and 0.686 of the bit's cells
  are within 300 units of a structure.
- Timeline: 81/81 events within 2 cells, and 81/81 in their lane. The maximum distance to a table structure is
  17.8 units.
- **Misses: none.**

## (4) Defects the real file exposed, fixed with synthetic tests

| Commit | What the real file showed | Fix | Synthetic test |
|---|---|---|---|
| `446b651` | R1's 0x46 mask turned the 136 base-gate cells (0x4x0) into walls | walkable = none of 0x02\|0x04; `FLAG_TABLE` gains a `settled` field per bit and keeps the other readings; `BLOCKING` 0x06, `BLOCKING_R1` 0x46 kept for reference; `synth` uses the 16.21 encoding | gate cells walkable |
| `6dcac43` | the transparent layer held every turret, inhibitor and gate (2,911 cells against SRX's 1,683) | transparent = 0x02\|0x40 without 0x04; new `structures` layer | structures and gates kept out of transparent |
| `b77a850` | the Baron pit was missing from the pits layer (on 16.21 it is POI 12; the converter's value 3 has 0 cells) | POI 11/12 added, `PIT_POI` = (3, 4, 12) | pits layer keeps a POI-12 patch |
| `309e0fd` | nexus turrets and timeline building events were not lane-checked | nexus counts as mid; the timeline's laneType and teamId are checked | lane check covers nexus and timeline |
| `a66c063` | two commit texts above said 1,683/1,683 | the agreement is 1,681 of 1,683 | (docs only) |

**Tests and guards.**

- `tools/navgrid/.venv/bin/python -m pytest tools/navgrid -q` → `64 passed in 48.90s`, up from 60 in R1.
- The derived-data guard (`-k riot_derived`) → `1 passed`.
- No real or derived data is in any commit. The worktree is clean apart from the verifier's untracked
  `VERIFY-R1.md`.

## Merge recommendation

**Merge the navgrid tool into main.** That means code and synthetic tests only: `tools/navgrid/`, `.gitignore`
and the mission folder.

- The real 16.21 file parses byte-exact.
- Validation passes on every structure and every timeline event.
- The four defects it exposed are fixed, each with a test.
- Nothing derived from the grid is in git.

The merge does **not** lift the ticket-2 rule. Until Riot answers:

- no grid, raster, GeoJSON or preview is committed, shipped or shared;
- the Rift Board vision layer does not ship on measured geometry.

## Next steps

1. **Overseer:** spawn the R2 verifier, then show the publish-round-r2 card.
2. **Jasper:** tap the merge question. After a PASS and a "Yes, merge", the overseer lands the branch.
3. **When Riot answers ticket 2:** if the answer is yes, a later round imports the vision-level GeoJSON into
   the Rift Board. If it is no, the tool stays a local check and the R2 outputs on brain are deleted.
