"""The findings template (M166 A2): docs/scout-findings.md is its empty rendering."""

from pathlib import Path

from scout import findings

REPO = Path(__file__).resolve().parents[3]
DOCS = REPO / "docs" / "scout-findings.md"
PORTAL = REPO / ".planning" / "missions" / "M156" / "portal-question.md"


def test_docs_is_the_empty_rendering_no_drift():
    assert DOCS.read_text(encoding="utf-8") == findings.render_empty(), \
        "docs/scout-findings.md drifted from scout/templates/scout-findings.md; re-render it with " \
        "scout.findings.render_empty()"


def test_banner_and_sections():
    text = DOCS.read_text(encoding="utf-8")
    assert "Status: template — fills on Jasper's run (M156 half C)" in text.splitlines()[2]
    for heading in ["## (a) Facts table", "## (b) Go/no-go per signal",
                    "## (c) Route decision for the Scout product round", "## (d) The portal question",
                    "## (e) Licence note", "## (f) Privacy", "## (g) What M156b would cover"]:
        assert heading in text, heading
    for unknown in ["interfaceAll=false", "replay speed", "Accelerated recording", "farClip",
                    "Vanguard", "Enemy wards on the minimap with fog off"]:
        assert unknown in text, unknown
    for signal in ["Positions via minimap (A)", "Positions via top-down (B)", "Wards", "Recalls"]:
        assert signal in text
    for route in ["A-only", "A+B", "B-first"]:
        assert route in text


def test_privacy_licence_and_portal_content():
    text = findings.TEMPLATE.read_text(encoding="utf-8")
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
