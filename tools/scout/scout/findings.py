"""Render docs/scout-findings.md from templates/scout-findings.md (M166 A2).

Slots are written {{name}}. The empty rendering fills every slot with a
"fills on run" marker; half C's rendering (scout.halfc) fills every slot with
computed values. Lines that carry "agent judgment" are left for half C to write
by hand. The committed docs/scout-findings.md is half C's filled, hand-written
version (aggregate numbers only: no match ID, no per-player data); a test checks
it keeps the template's section headings.
"""

from __future__ import annotations

import re
from pathlib import Path

TEMPLATE = Path(__file__).resolve().parent / "templates" / "scout-findings.md"
SLOT = re.compile(r"\{\{([a-z0-9_]+)\}\}")
EMPTY_STATUS = "Status: template — fills on Jasper's run (M156 half C)"
EMPTY_VALUE = "_(fills on Jasper's run)_"
JUDGMENT_MARK = "agent judgment"


def slots(text: str | None = None) -> list[str]:
    text = TEMPLATE.read_text(encoding="utf-8") if text is None else text
    return sorted(set(SLOT.findall(text)))


def render(values: dict[str, str], text: str | None = None) -> str:
    """Fill every slot; a slot without a value raises KeyError (no silent gaps)."""
    text = TEMPLATE.read_text(encoding="utf-8") if text is None else text
    missing = [s for s in slots(text) if s not in values]
    if missing:
        raise KeyError(f"no value for slot(s): {', '.join(missing)}")
    return SLOT.sub(lambda m: str(values[m.group(1)]), text)


def render_empty(text: str | None = None) -> str:
    values = {s: EMPTY_VALUE for s in slots(text)}
    values["status"] = EMPTY_STATUS
    return render(values, text)


def unfilled_lines(rendered: str) -> list[str]:
    """Lines that still hold a placeholder and are not agent-judgment lines."""
    out = []
    for line in rendered.splitlines():
        if (SLOT.search(line) or EMPTY_VALUE in line or "_(" in line) \
                and JUDGMENT_MARK not in line.lower():
            out.append(line)
    return out
