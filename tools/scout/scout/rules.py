"""Go/no-go per signal and the route rule (M166 A3).

The numbers live in thresholds.json next to this file (status "proposed").
Every verdict is one of GO, PARTIAL, NO-GO, PENDING (numbers pass but the hand
check is not marked or no homography is fitted yet) or NOT REACHED (the input
is missing). Each verdict carries the measured numbers and the reason, so the
findings can quote both.
"""

from __future__ import annotations

from pathlib import Path

from .util import read_json

DEFAULT_THRESHOLDS = Path(__file__).resolve().parent / "thresholds.json"
GO, PARTIAL, NOGO, PENDING, NOT_REACHED = "GO", "PARTIAL", "NO-GO", "PENDING", "NOT REACHED"


def load_thresholds(path: Path | str | None = None) -> dict:
    return read_json(path or DEFAULT_THRESHOLDS)


def _fmt(v) -> str:
    return "n/a" if v is None else str(v)


def verdict(status: str, reason: str, **numbers) -> dict:
    return {"verdict": status, "reason": reason, "numbers": numbers}


def positions_minimap(m: dict | None, th: dict) -> dict:
    """m: median_error, detection_rate, longest_gap_s (death gaps removed)."""
    if not m:
        return verdict(NOT_REACHED, "no minimap track or no timeline")
    med, det, gap = m.get("median_error"), m.get("detection_rate"), m.get("longest_gap_s")
    nums = dict(median_error=med, detection_rate=det, longest_gap_s=gap)
    if med is None or det is None:
        return verdict(NOGO, "nothing scored", **nums)
    go, part = th["go"], th["partial"]
    if med <= go["median_error_max"] and det >= go["detection_rate_min"] \
            and (gap or 0) <= go["longest_gap_s_max"]:
        return verdict(GO, f"median {med} <= {go['median_error_max']}, detection {det} >= "
                           f"{go['detection_rate_min']}, longest gap {gap} s <= "
                           f"{go['longest_gap_s_max']} s", **nums)
    if med <= part["median_error_max"] and det >= part["detection_rate_min"]:
        return verdict(PARTIAL, f"median {med} <= {part['median_error_max']}, detection {det} >= "
                                f"{part['detection_rate_min']}, but not every GO bound holds "
                                f"(longest gap {gap} s)", **nums)
    return verdict(NOGO, f"median {med}, detection {det}: outside the PARTIAL bounds "
                         f"({part['median_error_max']}, {part['detection_rate_min']})", **nums)


def positions_topdown(m: dict | None, th: dict) -> dict:
    """m: median_error, bar_hit_rate, calibrated (bool)."""
    if not m:
        return verdict(NOT_REACHED, "no track_topdown.json (route B not reached)")
    med, hit = m.get("median_error"), m.get("bar_hit_rate")
    nums = dict(median_error=med, bar_hit_rate=hit)
    go = th["go"]
    if not m.get("calibrated"):
        return verdict(PENDING, f"no homography fitted (anchors.json not filled); bar hit rate "
                                f"{_fmt(hit)}", **nums)
    if med is None or hit is None:
        return verdict(NOGO, "nothing scored", **nums)
    if med <= go["median_error_max"] and hit >= go["bar_hit_rate_min"]:
        return verdict(GO, f"median {med} <= {go['median_error_max']}, bar hit rate {hit} >= "
                           f"{go['bar_hit_rate_min']}", **nums)
    return verdict(NOGO, f"median {med} (GO <= {go['median_error_max']}), bar hit rate {hit} "
                         f"(GO >= {go['bar_hit_rate_min']})", **nums)


def _handcheck_ok(agree: int | None, rows: int | None, go: dict) -> bool:
    return bool(rows) and agree / rows >= go["handcheck_agree_min"] / go["handcheck_of"]


def precision_recall_handcheck(m: dict | None, th: dict, what: str) -> dict:
    """m: precision, recall, handcheck_agree, handcheck_rows (None = not marked)."""
    if not m:
        return verdict(NOT_REACHED, f"no {what} input")
    p, r = m.get("precision"), m.get("recall")
    agree, rows = m.get("handcheck_agree"), m.get("handcheck_rows")
    nums = dict(precision=p, recall=r, handcheck_agree=agree, handcheck_rows=rows)
    go = th["go"]
    if p is None or r is None:
        return verdict(NOGO, f"precision {_fmt(p)}, recall {_fmt(r)}: nothing detected or "
                             f"nothing to find", **nums)
    numbers_ok = p >= go["precision_min"] and r >= go["recall_min"]
    hc = "not marked" if rows is None else f"{agree}/{rows}"
    bounds = (f"precision {p} (GO >= {go['precision_min']}), recall {r} (GO >= {go['recall_min']}), "
              f"hand check {hc} (GO >= {go['handcheck_agree_min']} of {go['handcheck_of']})")
    if not numbers_ok:
        return verdict(NOGO, bounds, **nums)
    if rows is None:
        return verdict(PENDING, bounds + "; numbers pass, the hand check is not marked", **nums)
    if _handcheck_ok(agree, rows, go):
        note = f"; only {rows} marked rows" if rows < go["handcheck_of"] else ""
        return verdict(GO, bounds + note, **nums)
    return verdict(NOGO, bounds + "; the hand check disagrees", **nums)


def route(a: str, b: str, realtime_only: bool | None) -> dict:
    """The route rule on the A and B verdicts. realtime_only: True when the
    probe found no accelerated recording, None when that is unknown."""
    if b == GO and realtime_only and a in (GO, PARTIAL):
        return {"route": "A+B (B on key windows)",
                "reason": f"A {a}, B GO, recording is real time only: B on key windows, A for "
                          "the whole game"}
    if a == GO and b == GO:
        return {"route": "A+B", "reason": "A GO and B GO: B the primary source where recorded, "
                                          "A for full-game continuity"}
    if b == GO:
        extra = " (recording is real time only: key windows)" if realtime_only else ""
        return {"route": "B-first", "reason": f"B GO without A (A {a}){extra}"}
    if a == GO:
        why = "B not reached" if b == NOT_REACHED else f"B {b}"
        return {"route": "A-only", "reason": f"A GO, {why}"}
    note = " A is PARTIAL: half C may override to A-only (dashed, coarse) with a written reason." \
        if a == PARTIAL else ""
    return {"route": "no-go", "reason": f"neither GO (A {a}, B {b}): the board stays on tier 0+1 "
                                        f"and Scout is retired on evidence.{note}"}
