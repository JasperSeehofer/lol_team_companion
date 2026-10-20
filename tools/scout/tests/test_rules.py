"""Thresholds and the route rule (M166 A3). The numbers are the page's proposal."""

from scout import rules


def test_thresholds_are_the_proposed_numbers():
    th = rules.load_thresholds()
    assert th["status"] == "proposed"
    assert th["positions_minimap"]["go"] == {"median_error_max": 350, "detection_rate_min": 0.85,
                                             "longest_gap_s_max": 15}
    assert th["positions_minimap"]["partial"] == {"median_error_max": 700, "detection_rate_min": 0.60}
    assert th["positions_topdown"]["go"] == {"median_error_max": 150, "bar_hit_rate_min": 0.80}
    for k in ("wards", "recalls"):
        assert th[k]["go"] == {"precision_min": 0.80, "recall_min": 0.70,
                               "handcheck_agree_min": 4, "handcheck_of": 5}
    assert any("written reason" in line for line in th["route_rule"])


def test_positions_minimap_bounds():
    th = rules.load_thresholds()["positions_minimap"]
    v = lambda m, d, g: rules.positions_minimap(  # noqa: E731
        {"median_error": m, "detection_rate": d, "longest_gap_s": g}, th)["verdict"]
    assert v(350, 0.85, 15) == "GO"
    assert v(351, 0.85, 15) == "PARTIAL"
    assert v(300, 0.90, 16) == "PARTIAL"
    assert v(700, 0.60, 99) == "PARTIAL"
    assert v(701, 0.90, 0) == "NO-GO"
    assert v(300, 0.59, 0) == "NO-GO"
    assert rules.positions_minimap(None, th)["verdict"] == "NOT REACHED"


def test_positions_topdown_bounds():
    th = rules.load_thresholds()["positions_topdown"]
    v = lambda m, h, c=True: rules.positions_topdown(  # noqa: E731
        {"median_error": m, "bar_hit_rate": h, "calibrated": c}, th)["verdict"]
    assert v(150, 0.80) == "GO"
    assert v(151, 0.90) == "NO-GO"
    assert v(100, 0.79) == "NO-GO"
    assert v(None, 0.9, False) == "PENDING"
    assert rules.positions_topdown(None, th)["verdict"] == "NOT REACHED"


def test_wards_and_recalls_hand_check():
    th = rules.load_thresholds()["wards"]
    v = lambda p, r, a, n: rules.precision_recall_handcheck(  # noqa: E731
        {"precision": p, "recall": r, "handcheck_agree": a, "handcheck_rows": n}, th, "wards")["verdict"]
    assert v(0.8, 0.7, 4, 5) == "GO"
    assert v(0.8, 0.7, 3, 5) == "NO-GO"
    assert v(0.79, 0.9, 5, 5) == "NO-GO"
    assert v(0.9, 0.69, 5, 5) == "NO-GO"
    assert v(0.9, 0.9, None, None) == "PENDING"
    assert v(None, 0.9, 5, 5) == "NO-GO"


def test_route_rule_cases():
    G, P, N, R = rules.GO, rules.PARTIAL, rules.NOGO, rules.NOT_REACHED
    assert rules.route(G, G, False)["route"] == "A+B"
    assert rules.route(G, G, True)["route"] == "A+B (B on key windows)"
    assert rules.route(P, G, True)["route"] == "A+B (B on key windows)"
    assert rules.route(N, G, False)["route"] == "B-first"
    assert rules.route(G, R, None)["route"] == "A-only"
    assert rules.route(G, N, False)["route"] == "A-only"
    assert rules.route(P, N, False)["route"] == "no-go"
    assert "override" in rules.route(P, N, False)["reason"]
    assert rules.route(N, R, None)["route"] == "no-go"
