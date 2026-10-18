#!/usr/bin/env python3
"""Tests for the Rift geometry generator (standard library only).

    python3 -I scripts/rift-geometry/test_build.py
"""
import importlib.util
import json
import pathlib
import sys
import unittest

sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("rift_build", HERE / "build.py")
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


def outputs():
    src = build.load()
    stats = build.check(src)
    sk = build.build_sketch(src)
    stats.update(sk["stats"])
    return src, build.geometry_json(src, stats), build.rust_module(src, sk)


class GeneratorTest(unittest.TestCase):
    def test_deterministic(self):
        _, js1, rs1 = outputs()
        _, js2, rs2 = outputs()
        self.assertEqual(js1, js2)
        self.assertEqual(rs1, rs2)

    def test_committed_outputs_are_current(self):
        _, js, rs = outputs()
        self.assertEqual(build.OUT_JSON.read_text(), js, "run scripts/rift-geometry/build.py")
        self.assertEqual(build.OUT_RS.read_text(), rs, "run scripts/rift-geometry/build.py")

    def test_counts_and_sources(self):
        src, js, rs = outputs()
        doc = json.loads(js)
        c = doc["counts"]
        self.assertEqual(c["tower"], 22)
        self.assertEqual(c["inhibitor"], 6)
        self.assertEqual(c["nexus"], 2)
        self.assertEqual(c["pit"], 2)
        self.assertEqual(c["wall"], len(src["walls"]))
        for f in doc["features"]:
            self.assertTrue(any(f["source"].startswith(k) for k in build.SOURCE_KINDS), f["id"])
        self.assertIn("// @generated", rs.splitlines()[0])
        self.assertIn(f"[(f32, f32); 22]", rs)

    def test_fixture_positions_are_used(self):
        src = build.load()
        fixture = json.loads((build.ROOT / "tests/fixtures/riot_match_v5_timeline.json").read_text())
        events = []

        def walk(o):
            if isinstance(o, dict):
                if o.get("type") in ("BUILDING_KILL", "ELITE_MONSTER_KILL", "TURRET_PLATE_DESTROYED"):
                    events.append(o)
                for v in o.values():
                    walk(v)
            elif isinstance(o, list):
                for v in o:
                    walk(v)

        walk(fixture)
        used = {}
        for key in ("towers", "objectives"):
            for f in src[key]:
                if f["source"].startswith("fixture:"):
                    used[f["source"].rsplit(":", 1)[1]] = f["at"]
        for e in events:
            tag = f"{e['type']}@{e['timestamp']}"
            self.assertIn(tag, used)
            self.assertEqual(used[tag], [e["position"]["x"], e["position"]["y"]])

    def test_no_image_library(self):
        # Spelled in pieces so this file passes the same grep gate.
        banned = ["P" + "IL", "im" + "read", "c" + "v2", "sk" + "image", "pot" + "race"]
        for p in sorted(q for q in HERE.iterdir() if q.suffix in (".py", ".json")):
            text = p.read_text()
            for word in banned:
                self.assertNotIn(word, text, f"{p.name} mentions {word}")


if __name__ == "__main__":
    unittest.main()
