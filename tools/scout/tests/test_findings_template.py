"""The findings template (M166 A2) and the filled docs/scout-findings.md (M156 half C).

The committed doc is half C's hand-written version of the template, so the test
checks that it keeps the template's sections in order and its fixed content,
holds no unfilled slot, and carries no match ID (the repository is public).
"""

import re
from pathlib import Path

from scout import findings

REPO = Path(__file__).resolve().parents[3]
DOCS = REPO / "docs" / "scout-findings.md"
PORTAL = REPO / ".planning" / "missions" / "M156" / "portal-question.md"
SECTIONS = ["## (a) Facts table", "## (b) Go/no-go per signal",
            "## (c) Route decision for the Scout product round", "## (d) The portal question",
            "## (e) Licence note", "## (f) Privacy", "## (g) What M156b would cover"]
MATCH_ID = re.compile(r"\b[A-Z]{2,4}\d?_\d{6,}\b|\b\d{10}\b")


def headings(text: str) -> list[str]:
    return [ln.strip() for ln in text.splitlines() if ln.startswith("#")]


def test_docs_keeps_the_template_sections_no_drift():
    template = findings.TEMPLATE.read_text(encoding="utf-8")
    assert headings(DOCS.read_text(encoding="utf-8")) == headings(template), \
        "docs/scout-findings.md and scout/templates/scout-findings.md have different sections; " \
        "change both together"
    for heading in SECTIONS:
        assert heading in headings(template), heading


def test_docs_is_filled():
    text = DOCS.read_text(encoding="utf-8")
    assert text.splitlines()[2].startswith("Status: half C"), text.splitlines()[2]
    assert "{{" not in text and findings.EMPTY_VALUE not in text
    assert "(agent judgment: half C writes this line)" not in text
    for n in range(1, 7):   # the six unknowns, plus the HDR row
        assert re.search(rf"^\| {n} \|", text, re.M), f"facts row {n}"
    assert re.search(r"^\| HDR \|", text, re.M)
    for signal in ["Positions via minimap (A)", "Positions via top-down (B)", "Wards, blue team",
                   "Wards, red team", "Recalls"]:
        assert re.search(rf"^\| {re.escape(signal)} \|.*\| \*\*(GO|PARTIAL|NO-GO)\*\*", text, re.M), signal
    for route in ["A-only", "A+B", "B-first"]:
        assert route in text


def test_docs_carries_no_match_id():
    text = DOCS.read_text(encoding="utf-8")
    assert MATCH_ID.findall(text) == []


def test_privacy_licence_and_portal_content():
    for text in (findings.TEMPLATE.read_text(encoding="utf-8"), DOCS.read_text(encoding="utf-8")):
        assert "**L-1**" in text and "**12 months**" in text and "MATCH_CACHE_RETENTION_DAYS" in text
        assert "Art. 6(1)(f), 14, 17 and 21 GDPR" in text
        assert "AGPL-3.0-or-later" in text and "GPL-3.0" in text and "No stated licence" in text
        if PORTAL.is_file():   # the quoted message is verbatim
            body = PORTAL.read_text(encoding="utf-8").split("## Message", 1)[1].strip()
            for para in [p for p in body.split("\n\n") if p.strip()]:
                first = para.splitlines()[0]
                assert first in text, first


def test_every_slot_is_filled_or_judgment():
    empty = findings.render_empty()
    assert "{{" not in empty
    filled = findings.render({s: "x" for s in findings.slots()})
    assert findings.unfilled_lines(filled) == []
    judgment = [ln for ln in filled.splitlines() if "agent judgment" in ln.lower() and "_(" in ln]
    assert len(judgment) >= 8


def test_render_refuses_a_missing_slot():
    import pytest

    with pytest.raises(KeyError):
        findings.render({"status": "x"})
