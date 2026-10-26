# CI check log, run 40403819841 (0dcc401de95d13a829bb3cf06d6f285cea24d141)

Last 300 lines of the Check, lint, unit tests job.

```
=== Format check ===
Diff in /home/runner/work/lol_team_companion/lol_team_companion/src/models/guidance.rs:856:
         let n = pick_nudge(true, &f, &[], None).unwrap();
         assert_eq!(n.kind, NudgeKind::DraftWithoutPlan);
         assert_eq!(n.href, "/game-plan?draft_id=draft:d");
-        
+
         // With DraftWithoutPlan muted, they see DossierWithoutDraft
         let muted = [row(NudgeKind::DraftWithoutPlan.key(), false, true)];
         let n = pick_nudge(true, &f, &muted, None).unwrap();
Diff in /home/runner/work/lol_team_companion/lol_team_companion/src/models/guidance.rs:863:
         assert_eq!(n.kind, NudgeKind::DossierWithoutDraft);
-        
+
         // Regular member sees nothing
         f.viewer_manages = false;
         assert!(pick_nudge(true, &f, &[], None).is_none());
```
