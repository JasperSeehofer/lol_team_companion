# Rift schematic: measurement note

The drawn Rift (`RiftSketch`, `src/components/ui/rift.rs`) is an original schematic of the map. It is built from
numbers typed by hand into one file; no part of it is a picture of the map, and no program ever read a
picture to make it. This note says where every number came from (M152, scope item 2).

## Files

| File | What it is |
|---|---|
| `scripts/rift-geometry/rift-source.json` | The authored source: walls, river, ground edge, lanes, bushes, pits, objectives, camps, nexuses, inhibitors, towers. Every feature carries a `source`. |
| `scripts/rift-geometry/build.py` | The generator (Python standard library only). Checks the layers against each other, then writes the two outputs below. `--check` fails when an output is stale. |
| `scripts/rift-geometry/test_build.py` | Generator tests: deterministic output, committed outputs current, counts, fixture positions used as recorded, no image library in the directory. |
| `assets/rift/map11-geometry.json` | Generated: every feature in game units with its source, the counts and the check results. |
| `src/components/ui/rift_geometry.rs` | Generated (`// @generated`): SVG path data for the sketch and survey levels, in drawing units. |
| `scripts/og/build.py`, `scripts/og/capture.mjs` | The link-preview cards `public/og/{landing,share,default}.png` (recipe card F): `build.py` writes three 1200x630 pages from `rift_geometry.rs` (the sketch's own path data), the `input.css` tokens and the fonts in `public/fonts`; `capture.mjs` serves them on a loopback port, captures them with Playwright, reduces each to 64 colours (ImageMagick) and fails unless each is 1200x630, under 100 KB and fetched nothing else. Run `PLAYWRIGHT_CORE=<path to playwright-core> [CHROMIUM=<executable>] node scripts/og/capture.mjs` after the geometry changes. |

To change the drawing, edit the source file, then run `python3 -I scripts/rift-geometry/build.py` and
`python3 -I scripts/rift-geometry/test_build.py`.

## Coordinates and calibration

- **Game units:** 0 to 15000 on both axes, y pointing north, the frame of Match-V5 positions
  (`src/models/sand_table.rs`).
- **Drawing units** (the recipe cards' frame): `x_d = x / 150`, `y_d = 100 - y / 150`, a 100 x 100 sheet.
- **Reference frame:** both reference images use the frame of the old match-history map, game x from -120 to
  14870 and game y from -120 to 14980 across the image:
  `pixel_x = (x + 120) * N / 14990`, `pixel_y = N * (1 - (y + 120) / 15100)`, N the image width (3708 or 512).
  On the 3708 px image that is 0.247 px per game unit (about 4 units per pixel); on the 512 px minimap 29 units
  per pixel.
- **Checks of the calibration** (each passes; `build.py` asserts the first):
  - the fixture's Dragon kill (9866, 4414) lies 98 units from the measured dragon-pit centre (9940, 4350), inside
    the 490-unit pit;
  - the fixture's tower kill (10504, 1029) lies inside the measured bottom-lane band (y 640 to 1650);
  - the fixture's turret plate (13866, 4505) lies inside the measured right-hand lane band (x 13105 to 14140);
  - the measured Baron and Dragon pit centres are point-symmetric about (7395, 7472), the same centre the
    reported structure pairs give within 60 units;
  - the base emblems on the 3708 px image sit on the reported nexus-tower pairs.

## Method

- **References, kept outside the repository and never committed or shipped:**
  - `map11.png` from CommunityDragon's match-history plugin (3708 px). It is a stylised art map: its walls
    follow the game's, but it shows no river and draws each base as an emblem. It gave the nexus positions (the
    centres of the circle and square emblems) and served as the cross-check for the calibration.
  - The Data Dragon minimap `map11.png` of patch 16.20.1 (512 px), Riot's own published schematic and an
    approved asset under the Developer Portal policies. It shows walls, river and lanes as flat areas and was the
    reference for the outlines.
- **Measuring:** a crop of a reference was enlarged with a labelled grid every 250 game units drawn over it, and
  the coordinates of each outline's corners were read off by eye and typed into the source file, at 4 to 8
  game units per screen pixel. Nothing was sampled, thresholded, edge-detected, contoured or vectorised: every
  vertex is a number written after looking. Wall outlines have 5 to 38 vertices (bushes 4 to 5).
- **Checking:** the authored polygons were drawn over a dimmed copy of the reference, the misfits were read off
  the same way and the numbers retyped (two walls were corrected: the north edge of the Baron pit wall and the
  east edge of the red side's base wall, both concave curves first read as straight). These check images stay
  outside the repository too.
- **Consistency checks in the generator:** no structure or camp inside a wall, no wall overlapping another, no
  bush overlapping a wall or the river, every tower within 900 units of its lane's centreline (the worst is
  614, the blue top inhibitor tower, which stands on the base platform), the fixture Dragon inside the
  measured pit.

## Sources

| Source | Meaning | Features |
|---|---|---|
| `fixture:<file>:<event>@<timestamp>` | A Match-V5 event position in a repo fixture, used as recorded | 3: the Dragon objective, the blue bottom outer tower, the red bottom outer tower |
| `measured:<reference and crop>` | Read by eye off a gridded crop (`dd` = Data Dragon minimap, `m11` = `map11.png`) | walls (46), river, ground edge, lanes (3), pits (2), Baron/Herald/Grubs (one pit), camps (14), nexuses (2) |
| `reported` | The fixed position Match-V5 reports for that structure in every `BUILDING_KILL` event, entered from the API's published values. Neither reference shows towers or inhibitors, so these are not measurements; the blue and red pairs agree with each other under point symmetry within 60 units, and every one stands on its measured lane or base platform | towers (20), inhibitors (6) |
| `approximate` | Placed by game knowledge in open ground. **Neither reference shows brush**, so the bushes are not measured: their places and shapes are a sketch of where the game's brush is, the red side's mirror the blue side's through the map centre, and the generator only proves they sit in open ground | bushes (24) |

The turret-plate fixture event says team 100 but its position is the red side's bottom outer tower; the position
is what is used.

## Deviations from the recipe card (for the atelier)

- **No hatch at the sketch level (critique-r1-surfaces.md ruling 4, fix 1).** Hatch is survey-only. The earlier
  sketch hatch (card A as amended by critique-r1-rift.md fixes 1 to 6: walls merged within 2 units, simplified,
  filtered by area and width, pitch 5.0) left **13 lines**: measured walls are 4 to 7 units wide, so only 6 merged
  masses passed, and the sheet carried 4.47 % lit pixels at 1280 and 6.41 % at 390 with the figure set by the
  frame, lanes and labels rather than the pitch. Because the sketch never outlines a wall, those 13 lines had no
  visible mass: they read as arbitrary bars (at 390 the left stack read as text lines), not as terrain. Hatch only
  reads as terrain inside an outline, and outlines belong to the survey level, so the generator no longer emits
  `SKETCH_HATCH` and the sketch is frame, lanes, river, bases, Baron and Dragon, bush dots and the one chalk label.
  The quadrant hatch of R0 does not come back (it had no relation to measured walls). The 4.5 to 6 % lit-pixel
  target of card A now applies to the survey level only.
- **River centreline (card A, critique fix 7).** The rule is applied: the R1-a cubics are kept unless the
  measured centreline (the middle of the two banks, pits left out) deviates more than 1.5 units. It deviates up
  to 3.8 units, so the generator refits the same two cubics (smooth join at the mid crossing, then a
  deterministic search on the handles). The best two cubics stay up to 2.2 units off (about 330 game units, a
  third of the river's width), mostly at the top mouth, where the river bends towards the top lane. The card
  allows no third cubic, so this residual stays; it is recorded here and in the geometry JSON's checks
  (`river_dev_r1a`, `river_dev_fit`).
- **Sketch budget.** The sketch's path data is 3.1 KB, within the card's 4 KB estimate.
- **Match Day inset (critique fix 8; critique-r1-surfaces.md fix 2).** The inset draws no frame path, but at
  88 px the top and bottom lanes, whose measured outer corners are chamfered, closed into an octagon that read as
  a frame (an X in a box). The inset therefore draws its own lanes (`INSET_LANES`): the same snapped lanes with the
  two diagonal corner bends left out, so the corners stay open like the sheet's frame. Lanes, the river with its
  gap (centreline broken where the mid distance is under 5) and the bases are all it draws.
- **Share strip (critique fix 9).** Centred on (50, 50); the card's 40 x 10.7 (below `md`) and 50 x 9.4 (from
  `md`) are each widened by 3 units, to 43 x 11.53 and 53 x 9.96, so the strokes run to the edge.
- **Lanes.** The measured top and bottom lanes cut their outer corner on a diagonal (the playable ground is
  chamfered there), so the sketch's lanes are chamfered too rather than right-angled.

## Elevation

The Rift is flat except for the river basin: the river and its two pits sit one step below the ground, and every
lane, jungle path and base shares one level. The geometry JSON records this as a stub (the river polygon carries
`"elev": -1`, everything else the default 0) and nothing is rendered from it. Contours would not help: a
one-step basin has exactly one contour, and that contour is the river's outline, which both levels already draw
(the banks in the sketch, the measured outline in the survey). A second line beside the bank would only double
it, and an "index contour" with a label would claim a relief the map does not have. So there are no contours.

## What this is not

It is not a copy of Riot's map. No pixel of a Riot image is in the repository or in the build: the references
stay in a scratch directory, and the generator reads only the authored numbers (its directory has no image
library, a test checks that). There is no texture, no terrain colour, no Riot logo and no Riot icon on the
drawing: structures and camps are ink glyphs of our own (dots, rings, diamonds and the side shapes), Baron and
Dragon are written as words. The exact-outline request waits on Riot's answer about map reuse (the mission's
open legal question); until then the outlines are measured and redrawn by hand, not traced.

## Every feature and its source

Generated from `assets/rift/map11-geometry.json` (127 rows).

| Id | Kind | Source | What |
|---|---|---|---|
| `boundary` | boundary | `measured:dd crops h_0_0, k2, h_7500_7500, h_7500_0` | edge of the playable ground, lanes and bases included |
| `wall-b01` | wall | `measured:dd crop k1` | top lane, beside blue base |
| `wall-b02` | wall | `measured:dd crop k1` | top lane, gromp wall |
| `wall-b03` | wall | `measured:dd crop k1` | blue top-side jungle, long blade |
| `wall-b04` | wall | `measured:dd crop k2` | top river mouth, blue side |
| `wall-b05` | wall | `measured:dd crop k1` | top river bank, blue side |
| `wall-b06` | wall | `measured:dd crop k1` | blue sentinel crescent |
| `wall-b07` | wall | `measured:dd crop k1` | wolves crescent, blue |
| `wall-b08` | wall | `measured:dd crops h_0_0 and k1` | blue top-side jungle, base wall |
| `wall-b09` | wall | `measured:dd centre crop` | mid lane blade, blue top side |
| `wall-b10` | wall | `measured:dd centre crop` | mid river wall, blue top side |
| `wall-b11` | wall | `measured:dd crop h_0_0` | mid lane, blue bot side, by the base |
| `wall-b12` | wall | `measured:dd crop g_5000_0` | blue bot-side jungle, blade |
| `wall-b13` | wall | `measured:dd centre crop` | raptor wall, blue |
| `wall-b14` | wall | `measured:dd crop g_5000_0` | bot lane, beside blue base |
| `wall-b15` | wall | `measured:dd crop g_5000_0` | bot lane dome, blue |
| `wall-b16` | wall | `measured:dd crop g_5000_0` | bot lane wedge, blue |
| `wall-b17` | wall | `measured:dd crops g_5000_0 and h_7500_0` | krug wall, blue |
| `wall-b18` | wall | `measured:dd crop g_5000_0` | red brambleback crescent, blue side |
| `wall-b19` | wall | `measured:dd centre crop` | mid river wall, blue bot side |
| `wall-b20` | wall | `measured:dd crop h_7500_0` | bot lane wall by the river, blue |
| `wall-b21` | wall | `measured:dd crops h_7500_0 and r4` | dragon pit wall |
| `wall-b22` | wall | `measured:dd crops h_7500_0 and r4` | bot river mouth, blue side |
| `wall-b23` | wall | `measured:dd crop k2` | top lane alcove |
| `wall-r01` | wall | `measured:dd crop k2` | top river mouth, red side |
| `wall-r02` | wall | `measured:dd crops k3, r5 and q1` | baron pit wall |
| `wall-r03` | wall | `measured:dd crops k3 and r3` | krug wall, red |
| `wall-r04` | wall | `measured:dd crop r3` | top lane wall, red |
| `wall-r05` | wall | `measured:dd crop r3` | top lane dome, red |
| `wall-r06` | wall | `measured:dd crops r3 and r2` | top lane, beside red base |
| `wall-r07` | wall | `measured:dd crop r5` | red brambleback crescent, red side |
| `wall-r08` | wall | `measured:dd crop r5` | red top-side jungle, blade |
| `wall-r09` | wall | `measured:dd crop r5` | raptor wall, red |
| `wall-r10` | wall | `measured:dd crop r5` | mid river wall, red top side |
| `wall-r11` | wall | `measured:dd crop r2` | mid lane, red top side, by the base |
| `wall-r12` | wall | `measured:dd crops r2, r1 and q3` | red top-side jungle, base wall |
| `wall-r13` | wall | `measured:dd crop r2` | wolves crescent, red |
| `wall-r14` | wall | `measured:dd crop r2 and centre crop` | mid lane blade, red bot side |
| `wall-r15` | wall | `measured:dd centre crop` | mid river wall, red bot side |
| `wall-r16` | wall | `measured:dd crop r1` | red sentinel crescent |
| `wall-r17` | wall | `measured:dd crops r1 and h_7500_7500` | bot lane, beside red base |
| `wall-r18` | wall | `measured:dd crop r1` | bot lane, gromp wall, red |
| `wall-r19` | wall | `measured:dd crops r1 and r4` | red bot-side jungle, long blade |
| `wall-r20` | wall | `measured:dd crop r1` | bot river bank, red side |
| `wall-r21` | wall | `measured:dd crop r4` | bot lane, red, by the river |
| `wall-r22` | wall | `measured:dd crop r4` | bot river mouth, red side |
| `wall-r23` | wall | `measured:dd crops r4 and h_7500_0` | bot lane alcove |
| `river` | river | `measured:dd crops k2, k1, centre, h_7500_0, r4, r1, r5, k3` | water, both pits included; south-west bank from the top mouth to the bottom mouth, then the north-east bank back |
| `lane-top` | lane | `measured:dd crops h_0_7500 and k2 (middle of the band between the map edge and the first walls)` |  |
| `lane-mid` | lane | `measured:dd centre crop (axis of the mid band, through the centre)` |  |
| `lane-bot` | lane | `measured:dd crops h_7500_0 and r1 (middle of the band between the map edge and the first walls)` |  |
| `bush-b01` | bush | `approximate` | top lane, river mouth |
| `bush-b02` | bush | `approximate` | top lane side |
| `bush-b03` | bush | `approximate` | blue sentinel entrance |
| `bush-b04` | bush | `approximate` | top-side jungle, by the river mouth wall |
| `bush-b05` | bush | `approximate` | wolves entrance |
| `bush-b06` | bush | `approximate` | mid lane, top side |
| `bush-b07` | bush | `approximate` | mid lane, bot side |
| `bush-b08` | bush | `approximate` | river edge, bot side |
| `bush-b09` | bush | `approximate` | red brambleback entrance |
| `bush-b10` | bush | `approximate` | bot lane side |
| `bush-b11` | bush | `approximate` | bot lane, river mouth |
| `bush-b12` | bush | `approximate` | dragon approach |
| `bush-r01` | bush | `approximate` | bot lane, river mouth |
| `bush-r02` | bush | `approximate` | bot lane side |
| `bush-r03` | bush | `approximate` | red sentinel entrance |
| `bush-r04` | bush | `approximate` | bot-side jungle, by the river mouth wall |
| `bush-r05` | bush | `approximate` | wolves entrance |
| `bush-r06` | bush | `approximate` | mid lane, bot side |
| `bush-r07` | bush | `approximate` | mid lane, top side |
| `bush-r08` | bush | `approximate` | river edge, top side |
| `bush-r09` | bush | `approximate` | red brambleback entrance |
| `bush-r10` | bush | `approximate` | top lane side |
| `bush-r11` | bush | `approximate` | top lane, river mouth |
| `bush-r12` | bush | `approximate` | baron approach |
| `pit-baron` | pit | `measured:dd crop k3 (pit circle)` | baron |
| `pit-dragon` | pit | `measured:dd crop h_7500_0 (pit circle)` | dragon |
| `obj-baron` | objective | `measured:dd crop k3 (pit centre)` | baron |
| `obj-herald` | objective | `measured:dd crop k3 (same pit as Baron)` | riftherald |
| `obj-grubs` | objective | `measured:dd crop k3 (same pit as Baron)` | grub |
| `obj-dragon` | objective | `fixture:tests/fixtures/riot_match_v5_timeline.json:ELITE_MONSTER_KILL@150000` | dragon |
| `camp-b-blue` | camp | `measured:dd crop k1 (clearing centre)` | blue sentinel (blue side) |
| `camp-b-gromp` | camp | `measured:dd crop k1 (clearing centre)` | gromp (blue side) |
| `camp-b-wolves` | camp | `measured:dd crop k1 (clearing centre)` | wolves (blue side) |
| `camp-b-raptors` | camp | `measured:dd centre crop (clearing centre)` | raptors (blue side) |
| `camp-b-red` | camp | `measured:dd crop g_5000_0 (clearing centre)` | red brambleback (blue side) |
| `camp-b-krugs` | camp | `measured:dd crop h_7500_0 (clearing centre)` | krugs (blue side) |
| `camp-r-blue` | camp | `measured:dd crop r1 (clearing centre)` | blue sentinel (red side) |
| `camp-r-gromp` | camp | `measured:dd crop r1 (clearing centre)` | gromp (red side) |
| `camp-r-wolves` | camp | `measured:dd crop r2 (clearing centre)` | wolves (red side) |
| `camp-r-raptors` | camp | `measured:dd crop r5 (clearing centre)` | raptors (red side) |
| `camp-r-red` | camp | `measured:dd crop r5 (clearing centre)` | red brambleback (red side) |
| `camp-r-krugs` | camp | `measured:dd crop r3 (clearing centre)` | krugs (red side) |
| `camp-scuttle-top` | camp | `measured:dd crop k3 (middle of the river at the pit mouth)` | scuttle crab (top river) |
| `camp-scuttle-bot` | camp | `measured:dd crop h_7500_0 (middle of the river at the pit mouth)` | scuttle crab (bottom river) |
| `nexus-blue` | nexus | `measured:m11 crop n_blue (centre of the base emblem)` | blue |
| `nexus-red` | nexus | `measured:m11 crop n_red (centre of the base emblem)` | red |
| `inhib-b-top` | inhibitor | `reported` | blue top |
| `inhib-b-mid` | inhibitor | `reported` | blue mid |
| `inhib-b-bot` | inhibitor | `reported` | blue bot |
| `inhib-r-top` | inhibitor | `reported` | red top |
| `inhib-r-mid` | inhibitor | `reported` | red mid |
| `inhib-r-bot` | inhibitor | `reported` | red bot |
| `tower-b-top-outer` | tower | `reported` | blue top outer |
| `tower-b-top-inner` | tower | `reported` | blue top inner |
| `tower-b-top-inhib` | tower | `reported` | blue top inhibitor |
| `tower-b-mid-outer` | tower | `reported` | blue mid outer |
| `tower-b-mid-inner` | tower | `reported` | blue mid inner |
| `tower-b-mid-inhib` | tower | `reported` | blue mid inhibitor |
| `tower-b-bot-outer` | tower | `fixture:tests/fixtures/riot_match_v5_timeline.json:BUILDING_KILL@179500` | blue bot outer |
| `tower-b-bot-inner` | tower | `reported` | blue bot inner |
| `tower-b-bot-inhib` | tower | `reported` | blue bot inhibitor |
| `tower-b-nexus-1` | tower | `reported` | blue nexus nexus |
| `tower-b-nexus-2` | tower | `reported` | blue nexus nexus |
| `tower-r-top-outer` | tower | `reported` | red top outer |
| `tower-r-top-inner` | tower | `reported` | red top inner |
| `tower-r-top-inhib` | tower | `reported` | red top inhibitor |
| `tower-r-mid-outer` | tower | `reported` | red mid outer |
| `tower-r-mid-inner` | tower | `reported` | red mid inner |
| `tower-r-mid-inhib` | tower | `reported` | red mid inhibitor |
| `tower-r-bot-outer` | tower | `fixture:tests/fixtures/riot_match_v5_timeline.json:TURRET_PLATE_DESTROYED@170000` | red bot outer |
| `tower-r-bot-inner` | tower | `reported` | red bot inner |
| `tower-r-bot-inhib` | tower | `reported` | red bot inhibitor |
| `tower-r-nexus-1` | tower | `reported` | red nexus nexus |
| `tower-r-nexus-2` | tower | `reported` | red nexus nexus |
