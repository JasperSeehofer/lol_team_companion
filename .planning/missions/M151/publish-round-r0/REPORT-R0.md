# M151 — Team Companion identity, "Survey, sharpened" — REPORT R0 (with R0b)

Branch `mission/M151-identity-survey-sharpened-r0` (pushed, branch only; `origin/main` merged). Nothing merged,
deployed or touched on the VM. Everything below is gallery and planning files plus one `/design` gallery section
and `public/marks/`; no product page changed. Card: `publish-round-r0/` (id `m151-r0-team-companion-identity`).

CI: <filled by orchestrator>

Clippy `explicit_auto_deref` (src/pages/design_marks.rs, three errors; log `ci-check-log.md:20-27`) is fixed in
da342bc (`fix(design): drop explicit derefs in marks gallery (clippy)`); the "Check, lint, unit tests" job is green
on da342bc, 7cb2759 and later.

## Least-confident decisions (read these first)

1. **RESOLVED (R0b): T1 read as "a square with an X".** The fix list T1-1 to T1-7 (`critique-r0-t1.md:58-74`) is
   applied (`gallery/out/t1-fix-log.md:5-13`), then MD-1 to MD-3 from the re-critique (`t1-fix-log.md:21-28`).
   Atelier iteration 2: PASS on all four T1 stills and og-t1 (`critique-r0b-t1.md:80-92`).
2. **T2 ring spacing.** The card asked for 6 to 8 closed rings 12 units apart; that needs ~190 units in a 100-unit
   field. Rings are ~5.5 units across the river and ~8 along it (`gallery/out/t2-checks.md:41`). Uncritiqued.
3. **RESOLVED (R0b): mark placement on the sketches.** Baron and Dragon now sit in the curved river's bends,
   3.35 units clear of the near bank (`t1-fix-log.md:8`; `gallery/surfaces/art/t1.json:6-9`). Tower is still not
   used on any surface (`gallery/out/t1-checks.md:39`).
4. **Chalk count on landing = 2** ("YOUR TEAM" label in accent plus the primary button, `references.md:95-96`;
   the atelier counts 2 on both landing stills, `critique-r0b-t1.md:48-49`). DESIGN allows chalk only for focus,
   selection, the step being decided, the playhead and the one primary action (`docs/DESIGN.md:202`). Amendment
   text proposed below (for R1, not applied); the contradiction stays open (`critique-r0b-t1.md:78`).
5. **Decision against the brief's wording: the Match Day "both sides' marks facing" field card is dropped (MD-1).**
   R0b built it as a facing row (team names either side of the inset); the atelier ruled that it invents sides (in a
   best of 3 neither team owns blue or red), repeats the header and adds a fourth block to a one-figure card
   (`critique-r0b-t1.md:37`, `:61`). The inset now floats top right like T2/T3 (`t1-fix-log.md:25`;
   `gallery/surfaces/art/t1.json` has no `facing` key), as `references.md:129` asks. The owner may overrule.
6. **Stamp ships static.** The brief said "drawn once"; the atelier ruled static at first paint and the entrance
   was removed (`critique-r0-t1.md:88`; `gallery/prototypes/NOTES.md:7` records the conflict). The card's treatment
   question carries a chip "T1, but stamp drawn once" so the owner can override.
7. **Hatch density accepted at about 5 % lit pixels.** The atelier ruled it acceptable as is (5.1 to 5.5 %,
   `critique-r0b-t1.md:42`). The earlier "halve to ~3 %" target was a miscalculation by the R0 critique (it left out
   the longer curved banks; `critique-r0b-t1.md:27`, `:76`), not a builder miss.
8. **Share OG must never carry the token** or anything derived from it, and must look the same for unknown,
   expired and revoked links (`gallery/og/NOTES.md:6-8`). The proposed `og-meta.diff` keeps a static card. The T1
   share variant labels a team and invents a side: FIX for R1 (R1-d, `critique-r0b-t1.md:53`, `:72`).
9. **C paper:** pale chalk is 1.2:1 on paper, so it becomes deep olive `#5d5b0a`, and the danger hover needs a
   rule change (label switches to paper), not only a token remap (`gallery/c/contrast.md:40-41`). Uncritiqued.

## Atelier verdict per still

| Still | Verdict | Source |
|---|---|---|
| T1 landing 1280 | PASS | `critique-r0b-t1.md:87` (iteration 2) |
| T1 landing 390 | PASS | `critique-r0b-t1.md:88` (iteration 2) |
| T1 share 390 | PASS (R1-b open) | `critique-r0b-t1.md:89` (iteration 2) |
| T1 Match Day 390 (1x, DPR 2, full) | PASS | `critique-r0b-t1.md:86` (iteration 2) |
| og-t1 | PASS | `critique-r0b-t1.md:52`, `:90` |
| og-t1-share | FIX, R1 only (R1-d) | `critique-r0b-t1.md:53`, `:72` |
| Marks (nine) | PASS | `critique-r0-t1.md:78-80`, `:98` |
| Stamp | PASS (static) | `critique-r0-t1.md:88`, `:100` |
| Lock-in | FIX applied at 390; 1280 not re-judged (strip stale) | `critique-r0-t1.md:84-86`, `:99` |
| T2 (four stills), T3 (four stills), og-t2, og-t3, C paper | not critiqued | none |

## Scope

| # | Item | State | Evidence |
|---|---|---|---|
| 0 | References sheet + link check | done | `references.md` (recipe cards `:16-76`, T1 card `:90-99`, Match Day field card `:126-132`, mark grid spec `:153`), `link-check.md` |
| 1 | Nine original marks + gallery section | done, atelier PASS | `public/marks/` (18 files, 16 and 24 px); `src/pages/design_marks.rs:146` (`MarksSection`), `:187` (file test); `src/pages/design.rs:31`; `e2e/tests/gallery.spec.ts:41,402`; `critique-r0-t1.md:78-80` |
| 2 | reMarkable template | done | `remarkable/marks-template.pdf`, `remarkable/IMPORT.md`, `remarkable/build-template.py` |
| 3 | T1 / T2 / T3 on landing 1280+390, share 390, Match Day 390 | T1 done (PASS); T2, T3 drawn, uncritiqued | `gallery/out/t{1,2,3}-*.png` (T2/T3 Match Day and share re-captured after the shared skeleton change, `t1-fix-log.md:15`); T1 build `gallery/tools/build-t1-rift.py:47-83` (inset river gap `mid_dist < 4` at `:66`); `gallery/surfaces/surfaces.css:65` (crispEdges); `critique-r0b-t1.md:86-90` |
| 4 | Lock-in prototype | partly | `gallery/prototypes/lockin.html`; FIX applied at 390 (`critique-r0-t1.md:84-86`); `lockin-strip-1280.png` and `prototypes/NOTES.md:9` not refreshed |
| 5 | Result stamp | done (static) | `gallery/prototypes/stamp.html`, `stamp-{390,1280}.png`; `critique-r0-t1.md:88,100` |
| 6 | OG cards + favicon | partly | `og-t{1,2,3}.png`, `og-t1-share.png`; `gallery/og/og.html:30-31, 66` (share label, `t1-fix-log.md:13`); `og-meta.diff`; `gallery/og/favicon.svg`; og-t1 PASS, og-t1-share FIX R1, og-t2/t3 uncritiqued |
| 7 | C paper still | partly | `gallery/c/share-paper.html`, `compare.html`, `contrast.md`; `c-share-{compare,paper}-390.png`; no critique |

| Done criterion | State | Evidence |
|---|---|---|
| 1 References verified (at least 3 seen per effect) | done | `references.md:3`, `link-check.md` |
| 2 Atelier PASS per still | partly | T1 (four stills, og-t1), marks, stamp PASS (`critique-r0b-t1.md:86-90`, `critique-r0-t1.md:98,100`); T2, T3, og-t2/t3, C never critiqued; og-t1-share FIX; lock-in 1280 not re-judged |
| 3 Three treatments on all three surfaces at 390/1280 | done | `gallery/out/t*-{landing-1280,landing-390,share-390,matchday-390}.png`; T1 Match Day also at DPR 2 (`t1-matchday-390-dpr2.png`, `t1-fix-log.md:28`) |
| 4 Marks in `/design` with an e2e check | done | `e2e/tests/gallery.spec.ts:402-409` |
| 5 Lint gates hard zero on gallery | done | `gallery/tools/lint-gallery.sh`; `t1-checks.md:23`, `t2-checks.md:23`; "Gallery lint OK" `t1-fix-log.md:28` |
| 6 Motion strips incl. reduced motion | partly | 390 refreshed after fix; 1280 stale |
| 7 Card staged with react questions | done | `publish-round-r0/entry.json` (4 questions), `report.html`, link check clean |
| 8 CI green | see the CI line | da342bc fixed Clippy; check job green on da342bc, 7cb2759 and later |

## Legal question for the overseer

Ready-to-paste open-questions row:

| ASK-20261017-M151-01 | May a fan tool (Team Companion, no Riot branding) draw its own position/objective marks that resemble the in-game icons (roles, Baron, Dragon, Herald, Tower)? The set here is original geometry on a 16-unit grid, deliberately distant from the in-game icons (`references.md` Mark grid spec `:153`; "Riot distance" column `critique-marks.md:24`), drawn, not traced. | Ask the librarian (`/consult legal`) and the adviser before the marks ship in production; until then they live only in `/design`. | open |

## DESIGN.md amendments R1 needs (proposal only, not applied)

- **Chalk count of 2 on the landing and the share hero.** Current rule, `docs/DESIGN.md:202`: "**Accent:** chalk
  is for focus, the selected tab, the step being decided ("selecting"), the playhead and the one primary action per
  view. Not for decoration, headings or icons." Proposed replacement, ready to paste in R1:

  > - **Accent:** chalk is for focus, the selected tab, the step being decided ("selecting"), the playhead and the
  >   one next action per view. On the signed-out landing and the share hero a second chalk element, the surveyed
  >   team name, is allowed, because it is the same promise ("this sheet is for you"). Never more than two, and
  >   never a mark in chalk. Not for decoration, headings or icons.

  Principle 3 (`docs/DESIGN.md:13`) would gain "and the surveyed team name on landing and share". If the owner
  declines, the label drops to `primary` ink.
- **6.3 `RoleLabel` (`docs/DESIGN.md:276`)** and **7 "no role icons" (`:500`, `:540`):** allow the drawn mark beside
  the word, `aria-hidden` / `alt=""`, never `text-accent` (`critique-r0-t1.md:80`).
- New, if the owner says yes: the lead-figure grammar (one figure, one sentence, one action); stamp `GlyphSize::Lg`.

## R1 refinements (`critique-r0b-t1.md:67-72`, `:92-95`)

- **R1-a** River curvature about 2.3x: both cubics become `M14,14 C24,32 34,44 50,50 C66,56 76,68 86,86`, also in
  the `--inset` build (first priority: it separates river from mid at 88 px). Optional if T1 is picked: inset gap
  `mid_dist < 5`.
- **R1-b** Share strip without hatch: `[data-art="share"] .hatch { display: none; }`.
- **R1-c** Even hatch tone at 1x: `shape-rendering: crispEdges` on `.hatch`. Already applied in R0b
  (`gallery/surfaces/surfaces.css:65`).
- **R1-d** OG share variant: no chalk team label and no leader; ask the owner whether a link preview may show the
  sharing team's name at all.

## Budget

Metered cap 5.0 M reached in R0 (meter stop); wrap-up allowance at most 0.6 M. R0b is bounded at ≤1.2 M on top of
the 5.0 M + 0.6 M wrap-up; the cap of 7.5 M is the last raise. STOP RULE outcome: all three treatments were started
before the 3.5 M line; nothing landed in production.

## Next steps (R1)

1. Act on the owner's four card answers (treatment and stamp, marks, lead-figure grammar, paper).
2. If T1: apply R1-a, R1-b, R1-d; bring the `references.md` T1 card in line (`critique-r0b-t1.md:77`).
3. Atelier critique of T2 or T3 if picked, and of og-t2/t3 and C if they stay in play.
4. Re-capture and re-judge `lockin-strip-1280.png`; update `prototypes/NOTES.md`.
5. Settle the legal row and the DESIGN amendments before the marks or the chalk label leave the gallery.
