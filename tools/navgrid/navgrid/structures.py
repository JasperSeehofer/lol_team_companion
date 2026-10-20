# SPDX-License-Identifier: AGPL-3.0-only
"""Summoner's Rift structure positions in Match-V5 world units (x, y).

Copied from tools/scout/scout/calibrate.py on origin/mission/M156-scout-replay-spike
(commit 2182417, TURRETS and INHIBITORS). That table was checked on 2026-10-17:
all 22 turrets match the Bayes Esports "Tower Coordinates" table exactly, the
6 inhibitors come from the same table, and the repo fixture's Match-V5
BUILDING_KILL positions agree. The nexus centres are not in the Bayes table and
are left out here (unverified).

Axis convention: Match-V5 position.x is the navgrid's x, Match-V5 position.y is
the navgrid's z (the grid's second horizontal axis; y is height there).
"""

from __future__ import annotations

# name -> (x, y). side_lane() reads side and lane from the name.
TURRETS: dict[str, tuple[float, float]] = {
    "blue_top_outer": (981, 10441), "blue_top_inner": (1512, 6699),
    "blue_top_inhib": (1169, 4287), "blue_mid_outer": (5846, 6396),
    "blue_mid_inner": (5048, 4812), "blue_mid_inhib": (3651, 3696),
    "blue_bot_outer": (10504, 1029), "blue_bot_inner": (6919, 1483),
    "blue_bot_inhib": (4281, 1253), "blue_nexus_top": (1748, 2270),
    "blue_nexus_bot": (2177, 1807),
    "red_top_outer": (4318, 13875), "red_top_inner": (7943, 13411),
    "red_top_inhib": (10481, 13650), "red_mid_outer": (8955, 8510),
    "red_mid_inner": (9767, 10113), "red_mid_inhib": (11134, 11207),
    "red_bot_outer": (13866, 4505), "red_bot_inner": (13327, 8226),
    "red_bot_inhib": (13624, 10572), "red_nexus_top": (12611, 13084),
    "red_nexus_bot": (13052, 12612),
}

INHIBITORS: dict[str, tuple[float, float]] = {
    "blue_top_inhibitor": (1169, 3573), "blue_mid_inhibitor": (3203, 3208),
    "blue_bot_inhibitor": (3454, 1241), "red_top_inhibitor": (11261, 13659),
    "red_mid_inhibitor": (11603, 11667), "red_bot_inhibitor": (13598, 11316),
}

# Fountain spawn points (the mission page's values).
FOUNTAINS: dict[str, tuple[float, float]] = {
    "blue_fountain": (554, 581),
    "red_fountain": (14302, 14387),
}


def side_lane(name: str) -> tuple[str, str | None]:
    """('blue'|'red', 'top'|'mid'|'bot'|None) from a structure name."""
    parts = name.split("_")
    side = parts[0]
    lane = parts[1] if parts[1] in ("top", "mid", "bot") else None
    return side, lane


def all_structures() -> dict[str, tuple[float, float]]:
    """The 22 turrets and 6 inhibitors validation checks."""
    out = dict(TURRETS)
    out.update(INHIBITORS)
    return out
