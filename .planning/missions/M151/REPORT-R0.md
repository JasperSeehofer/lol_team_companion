# M151 — Team Companion identity, "Survey, sharpened" — REPORT R0

Branch `mission/M151-identity-survey-sharpened-r0` (pushed, branch only; `origin/main` merged). Nothing merged,
deployed or touched on the VM. Everything below is gallery and planning files plus one `/design` gallery section
and `public/marks/`; no product page changed. Card: `publish-round-r0/` (id `m151-r0-team-companion-identity`).

CI on 09a766d (run 38985876361): FAILED, "Check, lint, unit tests" at Clippy (SSR): three `clippy::explicit_auto_deref` errors at src/pages/design_marks.rs:155, :167, :170 (log: .planning/missions/M151/ci-check-log.md). Tip 24715b5 (run 38986142851) carries no code change and will fail the same way; e2e is skipped behind it. The fix for R0b is to drop the three explicit derefs.

## Least-confident decisions (read these first)

1. **T1 reads as "a square with an X".** The atelier's T1 critique (`critique-r0-t1.md:45-76`, verdict table
   `:90-100`) is FIX on all four T1 stills and writes a buildable fix list T1-1 to T1-7 (straight lanes, one curved
   river, perpendicular sparser hatch, a real Match Day inset, share crop naming a base, title wrap, Match Day
   header). **None of it is applied**: the fix pass stopped at the budget cap. The T1 stills on the card are pre-fix.
2. **T2 ring spacing.** The card asked for 6 to 8 closed rings 12 units apart; that needs ~190 units in a 100-unit
   field. Rings are ~5.5 units across the river and ~8 along it (`gallery/out/t2-checks.md:41`). Uncritiqued.
3. **Mark placement on the sketches.** Only Baron and Dragon sit on the T1 sketch (1280 only); Tower is not used on
   any surface (`gallery/out/t1-checks.md:39`). The atelier passed Baron/Dragon but wants them moved into the curved
   river's bends (`critique-r0-t1.md:42`, `:60`).
4. **Chalk count on landing = 2** ("YOUR TEAM" label in accent plus the primary button, `references.md:95-96`).
   DESIGN allows chalk only for focus, selection, the step being decided, the playhead and the one primary action
   (`docs/DESIGN.md:202`). Needs a DESIGN amendment, or the label loses its chalk.
5. **Match Day inset / "both sides' marks facing" not drawn.** The 88 px inset is grey mush and the phrase is not
   in `references.md` (`critique-r0-t1.md:33`); fix T1-4 (`:64`) is written, not built.
6. **Stamp ships static.** The brief said "drawn once"; the atelier ruled static at first paint and the entrance
   was removed (`critique-r0-t1.md:88`; `gallery/prototypes/NOTES.md:7` records the conflict). Owner may override.
7. **Share OG must never carry the token** or anything derived from it, and must look the same for unknown,
   expired and revoked links (`gallery/og/NOTES.md:6-8`). The proposed `og-meta.diff` keeps a static card.
8. **C paper:** pale chalk is 1.2:1 on paper, so it becomes deep olive `#5d5b0a`, and the danger hover needs a
   rule change (label switches to paper), not only a token remap (`gallery/c/contrast.md:40-41`).

## Scope

| # | Item | State | Evidence |
|---|---|---|---|
| 0 | References sheet + link check | done | `references.md` (recipe cards `:16-76`, mark grid spec `:153`), `link-check.md` (89 lines) |
| 1 | Nine original marks + gallery section | done, atelier PASS | `public/marks/` (18 files, 16 and 24 px); `src/pages/design_marks.rs` (219 lines); `src/pages/design.rs:31,70,172` (SECTIONS); `e2e/tests/gallery.spec.ts:41,402`; `critique-marks.md:53-55` FIX, then `critique-r0-t1.md:78-80` PASS |
| 2 | reMarkable template | done | `remarkable/marks-template.pdf`, `remarkable/IMPORT.md` (14 lines), `remarkable/build-template.py` |
| 3 | T1 / T2 / T3 on landing 1280+390, share 390, Match Day 390 | partly | 12 stills `gallery/out/t{1,2,3}-*.png`; `gallery/tools/lint-gallery.sh` passing (`t1-checks.md:23`, `t2-checks.md:23`); only T1 critiqued (FIX, not applied); T2, T3 uncritiqued |
| 4 | Lock-in prototype | partly | `gallery/prototypes/lockin.html`; FIX applied at 390 (stack below 640 px; `lockin-bulk-390.png` captured, `critique-r0-t1.md:84-86`); `lockin-strip-1280.png` and `prototypes/NOTES.md:9` not refreshed after the fix |
| 5 | Result stamp | done (static) | `gallery/prototypes/stamp.html` (no `stamp-in` left), `stamp-{390,1280}.png`; PASS static `critique-r0-t1.md:88,99` |
| 6 | OG cards + favicon | partly | `og-t{1,2,3}.png`; `og-meta.diff` (44 lines, `git apply --check` passed; favicon link `:16-17`); `gallery/og/favicon.svg`; no critique |
| 7 | C paper still | partly | `gallery/c/share-paper.html`, `compare.html`, `contrast.md`; `c-share-{compare,paper}-390.png`; no critique |

| Done criterion | State | Evidence |
|---|---|---|
| 1 References verified (at least 3 seen per effect) | done | `references.md:3`, `link-check.md` |
| 2 Atelier PASS per still | **not done** | PASS only for marks and stamp; T1 FIX unapplied; T2, T3, OG, C never critiqued |
| 3 Three treatments on all three surfaces at 390/1280 | done | `gallery/out/t*-{landing-1280,landing-390,share-390,matchday-390}.png` |
| 4 Marks in `/design` with an e2e check | done (not run in CI) | `e2e/tests/gallery.spec.ts:402`; e2e skipped, see CI |
| 5 Lint gates hard zero on gallery | done | `gallery/tools/lint-gallery.sh`; `t1-checks.md:23`, `t2-checks.md:23` |
| 6 Motion strips incl. reduced motion | partly | 390 refreshed after fix; 1280 stale |
| 7 Card staged with react questions | done | `publish-round-r0/entry.json`, `report.html`, link check clean |
| 8 CI green | **not done** | see CI |

## CI

The "Check, lint, unit tests" job **FAILED at "Clippy (SSR)" (exit 101)** on every pushed commit since 253dad9
(`feat(design): nine original marks`, the `design_marks.rs` gallery section). A fix agent identified a cause but
was stopped before writing it. `.github/workflows/ci.yml:76-87` now commits `.planning/missions/M151/ci-check-log.md`
on failure so the next run shows the error without a GitHub token. Because the check job failed, **e2e was
skipped**: there is no CI capture of `/design`. **CI is NOT green; done criterion 8 is not met; the verifier will
likely return FIX.**

## Legal question for the overseer

Ready-to-paste open-questions row:

| ASK-20261017-M151-01 | May a fan tool (Team Companion, no Riot branding) draw its own position/objective marks that resemble the in-game icons (roles, Baron, Dragon, Herald, Tower)? The set here is original geometry on a 16-unit grid, deliberately distant from the in-game icons (`references.md` Mark grid spec `:153`; "Riot distance" column `critique-marks.md:24`), drawn, not traced. | Ask the librarian (`/consult legal`) and the adviser before the marks ship in production; until then they live only in `/design`. | open |

## DESIGN.md amendments R1 needs (proposal only)

- **2 (principles) and the chalk line `docs/DESIGN.md:202`:** allow one chalk label naming a team on landing and
  share, or drop it (decision 4).
- **6.3 `RoleLabel` (`docs/DESIGN.md:276`)** and **7 "no role icons" (`:500`, `:540`):** allow the drawn mark beside
  the word, `aria-hidden` / `alt=""`, never `text-accent` (`critique-r0-t1.md:80`).
- New, if the owner says yes: the lead-figure grammar (one figure, one sentence, one action); stamp `GlyphSize::Lg`.

## Budget

Metered cap 5.0 M reached (meter stop); wrap-up allowance at most 0.6 M. STOP RULE outcome: all three treatments
were started before the 3.5 M line; nothing landed in production. R0b = apply the `critique-r0-t1.md` T1 fixes,
fix the Clippy error, atelier critique of T2/T3/OG/C, refresh the lock-in 1280 strip, get CI green.

## Next steps (R0b)

1. Read `ci-check-log.md` from the next failing run (or run `cargo clippy --features ssr -- -D warnings`), fix
   `design_marks.rs`, get every CI job green.
2. Apply T1-1 to T1-7 (`critique-r0-t1.md:58-74`) in `gallery/tools/build-t1-rift.py` and the surfaces; re-render.
3. Fresh atelier critique of T1 (after fix), T2, T3, the OG cards and the C still.
4. Re-capture `lockin-strip-1280.png`; update `prototypes/NOTES.md`.
5. Act on the owner's four card answers; settle the legal row before the marks leave `/design`.
