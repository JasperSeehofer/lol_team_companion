# M160 item 4 — Nudges role-filtered (judgment, small)
Facts (re-locate): src/models/guidance.rs `pick_nudge` ~:656 `(enabled, f: &NudgeFacts, rows, skip_href) -> Option<Nudge>`; `NudgeFacts` ~:565; `candidate(kind, f)` ~:590; DraftWithoutPlan ~:622-628, DossierWithoutDraft ~:629-640. Built in src/server/guidance.rs `nudge_facts` ~:299 (has user_id and team_id), literal ~:424-432; only caller src/server/guidance.rs ~:489 (home_guide). Tests construct NudgeFacts via `facts()` ~:830 in guidance.rs.
Implement:
1. Add `viewer_manages: bool` to NudgeFacts (true when the viewer is the team leader or a coach: `db::is_team_leader_or_coach` db.rs ~:227), filled in nudge_facts.
2. In `candidate`, DraftWithoutPlan and DossierWithoutDraft return None when !f.viewer_manages. Other nudges unchanged.
3. Unit tests: with viewer_manages=false a draft without a plan / dossier without draft yields no such nudge (and a member's other nudges still appear, e.g. match_without_review); with true unchanged. `facts()` default viewer_manages: true so existing tests keep their meaning.
4. cargo test --features ssr --lib guidance passes; fmt; cargo check ssr.
