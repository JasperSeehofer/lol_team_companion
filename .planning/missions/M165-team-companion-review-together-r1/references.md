# M165 references (atelier intake, sheet mode, 2026-11-13)

Judged by the atelier from the Sonnet researcher's raw notes
(`references-raw.md`, gathered 2026-11-13). **Honesty note: every row below is
described, not seen.** The researcher worked from doc text and search snippets.
Status codes are theirs: [200] means curl resolved it, [403] means it is bot-blocked
and was seen only in a snippet. The atelier has no web tool and opened none of these
pages. No screenshot exists for any reference. Each row therefore commits to a
**technique**, never to a look. Before the frame-strip critique, someone with a
browser should capture one still of rows a1, b2, c1, d1 and e1 into
`publish-round-r1/refs/`, so that V8 has something to put beside the render. Until
then, V8 is judged against the recipe parameters in `design-brief-r1.md`.

Rows with a status in brackets were checked by the researcher. "(snippet)" means the
content is known only from a search snippet.

| effect | reference (URL) | why it's the target | what to copy / what to ignore | phone caveat |
|---|---|---|---|---|
| **(a) silent-round card** | a1 Parabol, Reflect phase [403, snippet] https://www.parabol.co/templates/sprint-retrospectives/hopes-and-fears-retrospective/ | The closest product to our model: each person writes privately per prompt, and the reveal is a **phase change**, not a toggle on each card. | **Copy:** the sealed sentence ("no one can see them until…") stated once, in words. Others' cards are absent during writing. **Ignore:** sticky-note columns, colour-coded cards, drag. | Its columns become a swipe on phones. Ours is one stacked column of four prompts, with no horizontal swipe. |
| | a2 Typeform [200] https://www.typeform.com | The calm of writing alone: one prompt set large, a lot of space, a quiet progress cue. | **Copy:** the prompt as a display-face heading with a single field under it, and a progress cue in mono caption ("2 of 4 answered", your own only). **Ignore:** one question per screen with slide transitions, full-bleed colour, Enter-to-advance. Ours shows all four prompts so a player can skip around. | Native strength. Keep the field full width and the prompt at text-heading, never smaller. |
| | a3 Echometer retro boards [200] https://echometerapp.com/en/retro-boards | The only source with a **soft timer**: a step that runs long is flagged, not cut off. | **Copy:** the timer is advisory text ("3 min left"). **Ignore:** board clutter and the show/hide-cards control. Rejected from the same group: Retrium's **blurred** sealed notes. Blur is banned (lint `blur`), and placeholders for other people's notes would leak a count per prompt. | The timer is one line of text. It never sits in a sticky bar and never pins over the fields. |
| **(b) simultaneous reveal (signature)** | b1 planningpoker.live, "what is planning poker" [403, snippet] https://planningpoker.live/what-is-planning-poker | Gives the **reason** for flipping together ("so the last person is not swayed"), which becomes our microcopy. A host reveals, or the reveal happens when the round ends. | **Copy:** one explicit "Reveal now" action, or the deadline. Every card arrives in one state change. **Ignore:** the per-person "has voted" check, which the source set warns about. That is exactly the tally we ban. | None. It is a state change, not a layout. |
| | b2 scrumonline (open source) [200] https://github.com/Zahak/scrumonline | It runs locally, so a builder could actually *see* a one-flip reveal before building ours. | **Copy:** a single flip of the whole set. **Ignore:** the card-deck look, poker chrome, and above all the min/max highlight after the reveal. Highlighting outliers singles out the minority, and our rule is that the minority card stays plain and visible. | Card-deck grids break at 390 px. We use ruled notes in one column. |
| | b3 Parabol, Group phase [403, snippet] (same URL as a1) | The reveal as a phase: the whole list changes state at once, with nothing staggered. | **Copy:** the phase change as the only trigger of the reveal. **Ignore:** drag-to-group. Our merge is a facilitator action on a suggestion. | n/a |
| **(c) presence strip** | c1 Liveblocks, presence use case / AvatarStack [200] https://liveblocks.io/docs/use-cases/presence | Presence as "who is here", deduplicated across tabs. | **Copy:** one entry per user however many tabs they have open, and a stable order. **Ignore:** live cursors, "+N" overflow (a team of 5 to 8 fits), and coloured activity rings. | Five avatars at 24 px with names wrap onto two lines at most. Never truncate a name to "+2". |
| | c2 SaaS UI, avatar presence patterns [200] https://www.saasui.design/blog/saas-avatar-user-presence-ux-patterns | Its rule "a stale dot is worse than none" is the honesty rule. | **Copy:** no status dots at all. When a poll fails, a word: "Reconnecting…". **Ignore:** green/grey dots, "typing…" indicators. A typing indicator is a "has written" signal and is banned. | Same rules as at 1280 px. |
| | c3 Fluent 2 AvatarGroup [200] https://fluent2.microsoft.design/components/ios/avatargroup/usage | Separates "stack" from "pile" and reserves rings for activity. We take the pile, with no rings. | **Copy:** a flat pile (no overlap) with the name beside each avatar. **Ignore:** overlapping stacks (names hidden) and activity rings. | A pile wraps where a stack would not, which is acceptable at 390 px. |
| **(d) decide vote + owner chips** | d1 Klaxoon, Dot Voting help [200] https://help.klaxoon.com/hc/en-us/articles/32680747508626-Use-Dot-Voting | A live "tokens left" counter that cannot go past the total. | **Copy:** "2 of 3 votes left" in mono figures. At zero, the vote buttons are disabled with a sentence that says why. **Ignore:** coloured dot tokens. | The counter sits in the Decide heading line, not in a floating bar. |
| | d2 AgileBox vote stage [200] https://docs.devsamurai.com/agilebox-dc/vote-stage | Separates a **total** cap from a **per-item** cap. | **Copy:** 3 in total and 1 per proposal, so nobody can stack votes. **Ignore:** configurable caps and pollunit-style forms. | n/a |
| | d3 Linear, assigning issues [200] https://linear.app/docs/assigning-issues | The owner shown as a small chip, chosen from the team's people. | **Copy:** the owner is a plain username chip, with "No owner yet" until one is chosen. **Ignore:** the A shortcut, type-ahead search (5 to 8 members fit as chips) and agent delegation. | A row of 44 px chips that wraps. Primer's sheet-instead-of-popover advice (seen, [200]) supports never using an anchored popover on phones. |
| **(e) member words in serif, author in sans caption** | e1 Stripe Press [200] https://stripe.press | Restrained serif on a plain dark-enough ground, with generous leading and small sans metadata. | **Copy:** the contrast of a serif voice against sans metadata, at reading size with 1.6 leading. **Ignore:** book-cover colour and display sizes. (The third-party style-export claim, "Ivar", is unverified and irrelevant: our face is Source Serif 4.) | 16 px serif italic at 358 px gives about 45 to 50 characters per line. That is fine, and it stays 16 px. |
| | e2 Microsoft Writing Style Guide, pull quotes [200] https://learn.microsoft.com/style-guide/scannable-content/pull-quotes | Attribution on its own line, smaller, in sans. | **Copy:** the username on its own line under the words, in the sans caption, with no em-dash and no quotation marks. **Ignore:** oversized quote marks. | The same at every width. |
| | e3 Fonts In Use, pull-quotes tag [200] https://www.fontsinuse.com/tags/3713/pull-quotes | Real editorial examples of italic serif between hairline rules. This is the "ruled notes, not boxes" form. | **Copy:** notes separated by hairlines (`divide-rule`), with no box, fill or card radius. **Ignore:** giant pull-quote sizes and double rules. Dropped: the New Yorker "design-md export" lead (third-party, not verified). | Hairlines survive any width. |

**Dropped from the raw set:** Retrium (blur), the Miro and Figma feature requests
(they are evidence of need, not UI), NN/g dot voting (the method behind the cap
rationale, kept in the brief's reasoning rather than as a visual), and the New Yorker
export.

**Reference gap (stated, not padded):** no reference shows a *typographic*
simultaneous reveal. Every source flips cards. The motion parameters of the reveal
(220 ms, opacity only, one element) therefore come from DESIGN §3.7 and principle 11,
not from a reference. V8 for the reveal is judged against the recipe card's frame
spec (card 8).
