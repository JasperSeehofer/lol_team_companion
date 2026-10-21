# M160 item 4 — fix round 1 (from the Sonnet verifier)
Production code in commit 90f106f is correct; only the new test `nudges_are_filtered_by_role` in src/models/guidance.rs (~:846) is wrong. `NudgeKind::ALL` order: Returning, MatchWithoutReview, OldActionItems, DraftWithoutPlan, DossierWithoutDraft; `pick_nudge` returns the FIRST applicable kind, so earlier kinds mask later ones.
Failures: (a) the leader/coach case expects DraftWithoutPlan but `old_open_items = 3` is still set, so OldActionItems wins; (b) the last member assert expects `pick_nudge(..., Some("/game-plan?draft_id=draft:d")).is_none()` but MatchWithoutReview still applies; (c) member asserts only check `is_some()`, proving nothing.
Rewrite the test (or split into two tests) so that:
1. Facts with ONLY draft_without_plan and dossier_without_draft set (match_without_review = None, old_open_items = 0, and whatever makes Returning not apply, as in the neighbouring tests' `facts()`):
   - viewer_manages = true: `pick_nudge(true, &f, &[], None)` has kind DraftWithoutPlan; with DraftWithoutPlan muted (use the existing helper the other tests use to build a muted GuidanceRow, e.g. `row(NudgeKind::DraftWithoutPlan.key(), ..)` — read it) the result is DossierWithoutDraft.
   - viewer_manages = false: `pick_nudge(true, &f, &[], None)` is None.
2. Facts with match_without_review set and viewer_manages = false: the result is MatchWithoutReview (members still get their own nudges).
3. Remove the contradictory asserts. Change nothing outside this test.
