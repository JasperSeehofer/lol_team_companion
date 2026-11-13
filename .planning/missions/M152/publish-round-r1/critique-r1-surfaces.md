# Critique R1 surfaces (atelier, critique mode, 2026-10-17)

Rubric CLI: not runnable here (needs a live page and playwright-core, absent on this host; the garden-wiki path also does not exist). CI-only. Contrast not recomputed this pass; ink/chalk values carried from critique-r1-rift (8.3:1 ink on page). Judged from CI frames (after/) and gallery/out stills; no motion exists on these surfaces, so no strips. Before frames: M148 `match-day-phone-390/1280`, `login-*` (no landing/share/closed-beta before-frame exists on main in M148/M141: new surfaces, judged against cards only). OG PNGs were not in the frame set: not judged.

| Surface | Verdict | Craft answer, one line |
|---|---|---|
| landing-1280 / -390 / -390-flow (hero) | FIX | Layout, one headline, one chalk button, YOUR TEAM leader: PASS. But the wall-hatch fragments read as arbitrary bars (see ruling), and they compete with BARON/DRAGON. Whole drawing sits inside the first 852 px at 390 (bottom ~725). |
| closed-beta-390/1280 | FIX | Same sketch, same hatch fault; chalk count 2 (label + button), header "Request access" correctly Secondary. |
| rift-sketch stills (4.47 % / 6.41 %) | FIX | Numbers are in range, craft is not: bars float without a mass, left stack looks like a bulleted list. Could you name them? "Text lines." |
| design-rift-1280 | PASS (survey, previously PASS) / FIX (strips) | See share and inset. Survey stops not re-viewed. |
| match-day-390/-390-full/-1280 | FIX | Card grammar PASS (figure, sentence, one chalk button). Team names: REJECT (below). Inset at 75 px reads as an X in a box (envelope/cancel icon); card said no frame. |
| share-390/1280 | FIX | Ring, a rule and two stray clipped stubs at the top: nameable as a logo or loading glyph, not a map. Strip is a claim without a label. |
| rolelabel-roster-390 | PASS | 14 px mask icons muted, left of letters, letters unchanged; row rhythm intact. Guessed state not in frame. |
| PrepSequence Quiet (match-day-390-full) | FIX | Boxed chalk "1", chalk "Read by 0 of 5" plus the chalk button = 3 chalk items in one view; also amber "! 5 open". |

## Rulings on the orchestrator's decisions
1. **Team names at the inset ends: REJECT, ruled again on the render.** "Riverside Rooks" sits top left, by neither base (red square is top right); "Northgate Academy" bottom right, far from the blue circle bottom left. The placement asserts sides that do not exist (best of 3) and is misaligned anyway; 12 px names wrap to two lines each at 390 and shrink the picture; the title and the subtitle already print both names (third print). Remove. Scope 4's "facing" is the title "A vs B"; flag to owner as carried CONTRADICTION.
2. **Share strip centred on our side, no label: FIX, not PASS.** A base crop is only legitimate with its label (card A). Without one it is a ring and a line. Add a chalk "OURS" (the page already prints it) with leader, or revert to the bridge crop. Pick the label; chalk count stays 1.
3. **Quiet keeps chalk on next step: FIX.** Chalk is for the page's one action. Next step number box and "Read by" go `secondary`/`primary`, outline token box; "Next" word ink.
4. **Sketch hatch: drop wall hatch at sketch level; do not return to R0's quadrant hatch.** Hatch only reads as terrain when it has a mass shape; at sketch level the walls are, by my own card, never outlined, so the mass is invisible and the fragments read as arbitrary bars. The quadrant hatch had no relation to measured walls (R0 passed it as tone, not terrain). Survey level keeps outline + 45-degree hatch (that is where walls are real). Without hatch the sketch is frame, lanes, river, bases, Baron/Dragon, bush dots, one chalk label: calmer and the label is the sole signature. Amend card A: the 4.5-6 % lit-pixel target applies to survey only.

## Fix list (one builder pass; all small)
1. `scripts/rift-geometry/build.py` + generated rift const: sketch emits no wall hatch (keep bush dots); regenerate; survey untouched. Update `docs/rift-schematic.md` (hatch is survey-only; 13-lines finding recorded as the reason).
2. `src/components/ui/rift.rs` inset variant: remove the octagon frame path; keep lanes, river, bases; widen bridge gap to `mid_dist < 5` (card).
3. Match Day card component (`src/pages/home.rs` or its field-card): delete the two team-name captions around the inset; revert to inset floated top right with the sentence wrapping beside it.
4. `rift.rs` share variant: shift the crop so no lane/hatch stub is clipped at the top (stubs end in-frame or the crop clears them).
5. Share page (`src/pages/share.rs`): chalk "OURS" label with 7 u leader at the base, `eyebrow` style; no label on unknown/expired/revoked.
6. `src/components/ui/prep.rs` `PrepEmphasis::Quiet`: next-step number box `border-outline`, text `primary`; "Read by x of y" `secondary`; remove `text-accent` on the Quiet path (Primary keeps it).
7. `e2e` chalk-count check on match-day at 390: assert <= 2 accent items (button + none).
8. `docs/DESIGN.md` / REPORT: record ruling 1 for the owner.

R1b (not this pass): Match Day card is capped at 640 px at 1280, leaving a void right; OG PNG review once rendered.

## Staleness flags
- CONTRADICTION: mission Scope 4 (teams facing) vs card E / R0 MD-1, still open; ruling above.
- STALE: recipes-r1.md card A hatch target (4.5-6 %) now survey-only; amend after fix 1.

## Re-judge (iteration 2)

Rubric CLI: CI-only on this host. Contrast recomputed from pixels (bg rgb 8,20,21): chalk label 14.5:1 (landing, share), ink H1 16.2:1, OG title 16.2:1, OG subtitle 12.3:1, OG chalk 14.4:1. All clear AA/AAA. Static surfaces, no strips.

| Surface | Verdict | Note |
|---|---|---|
| landing 1280 | PASS | Hatch gone; frame, lanes, river, bases, Baron/Dragon, one chalk label. Calm, label is the signature. |
| landing 390 | PASS | Sketch below CTA, no clipping. |
| reading-flow (390-flow) | PASS | Headline, sentence, one chalk button, then sketch, four steps. |
| closed-beta | PASS | Same sketch (judged from shared component; frame not re-opened). |
| share 390 / 1280 | PASS | Chalk "OURS" with leader, ring and rule read as a labelled base crop. Top stubs no longer clipped. |
| Match Day 390 / 1280 | FIX (R1b) | Team-name captions gone, card grammar good, Quiet prep no chalk. Inset at ~75 px is still an X in a square (envelope). Does not block landing; drop it below 120 px or show bridge crop. |
| RoleLabel | PASS | Unchanged. |
| design-rift survey | PASS | Survey untouched. |
| og-landing | PASS | Text left, sketch bleeds right edge deliberately; chalk label legible at thumbnail. Red base crops out: acceptable. |
| og-share | PASS (note) | Lanes only, no frame, no label: abstract, reads as texture. Ship; R1b could add the OURS crop. |
| og-default | PASS | Same as card F, no chalk label (correct: no side). |

Nothing blocks landing. R1b: Match Day inset size; og-share crop.
