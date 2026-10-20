# navgrid

Reads the Summoner's Rift navigation grid (`.aimesh_ngrid`, v7.1) from **your
own local game install**, checks it against public structure positions, and
(optionally) turns it into GeoJSON layers. Offline: no command opens a socket.

## The ticket-2 rule

Riot has not yet answered ticket 2 (submitted 2026-10-18): whether polygons
derived from the navgrid may be published, and whether the file is a
permitted source at all. Until Riot says yes:

- **Nothing derived from a real file is committed, shipped or shared**: no
  grid, raster, GeoJSON, preview image or screenshot of one, on any branch, in
  the app, or on a card.
- The extraction is your own act on your own install, with the game closed.
  The output stays on your PC and in one directory on brain
  (`~/.local/share/navgrid/`), outside every repository, and can be deleted.
- Three guards back this up: `.gitignore` (`*.aimesh_ngrid`, `*.wad.client`,
  `*.geojson`, `tools/navgrid/out/`), the refusal to write output inside a git
  work tree, and `tests/test_guards.py::test_no_riot_derived_files`.

The tests read only the fabricated grid from `navgrid/synth.py` and WADs the
tests write themselves, in temporary directories.

## Install

```powershell
py -3.12 -m venv tools\navgrid\.venv
.\tools\navgrid\.venv\Scripts\pip install -e tools/navgrid
```

Python 3.10 or newer. `pip install -e "tools/navgrid[vector]"` adds `shapely`
for `vectorise`; `[test]` adds `pytest`.

## Commands

`python -m navgrid pc-run [--wad PATH] [--out DIR]`
: The one PC command. With the game closed, finds `Map11.wad.client` under
  `C:\Riot Games\League of Legends\Game\DATA\FINAL\Maps\Shipping\` (or the
  same without `Game\`, drives C to F), reads it once, finds the navgrid by
  its known paths (`aipath_srx_2`, `_3`, `aipath_srx`, `aipath`) or by its
  header, and writes `%LOCALAPPDATA%\navgrid\latest\`: the grid file,
  `validation.json` and `manifest.json` (hashes, WAD date, entry path, how it
  was found, client version when readable). The previous run moves to
  `previous\`. It ends by printing the line that sends the folder to brain:

  ```powershell
  scp -r "$env:LOCALAPPDATA\navgrid\latest" ops@10.44.0.1:.local/share/navgrid/inbox/
  ```

`python -m navgrid validate GRID [--timeline TIMELINE.json]`
: Numbers only: each of the 22 turrets and 6 inhibitors within 2 cells of a
  structure cluster and in its lane region, both fountains walkable, the
  bit-0x04 structure-wall hypothesis, the flag histogram, and with a Match-V5
  timeline the `BUILDING_KILL` / `TURRET_PLATE_DESTROYED` positions. Exit 0 on
  pass, 1 on fail.

`python -m navgrid vectorise GRID [--out DIR]`
: GeoJSON layers in game units (needs the `vector` extra). Refuses an output
  directory inside a git work tree; the default is
  `~/.local/share/navgrid/out/<sha8>/`.

## Format notes

`navgrid/ngrid.py` documents the layout. Flag meanings disagree between
sources; `ngrid.FLAG_TABLE` carries the plan's, MapgeoAddon's and the
NGRID converter's reading of every bit, and the real file decides in R2.
Axis convention: grid x is Match-V5 `position.x`, grid z is `position.y`.

## Tests

```bash
tools/navgrid/.venv/bin/python -m pytest tools/navgrid
```

## Licence

AGPL-3.0-only; see `LICENSE` and `NOTICE.md` (MapgeoAddon MIT notice, NGRID
converter attribution).
