# M160 item 2a — fix round 1 (from the Sonnet verifier)
Your commit dc25f72 left three assertions on the old step indices. New order: 0 team, 1 invite, 2 starting five, 3 dossier, 4 draft, 5 plan, 6 review.
1. src/models/guidance.rs, test `steps_tick_themselves_from_the_records`, second block (~:756-766): the `states` vector becomes [true,false,false,false,true,false,false] and the status check becomes `items[4].status == "2 drafts"`.
2. Same test, last assert (currently `Some(3)`): with starting_positions=5, opponents=1, members=1 the invite step (index 1) is still open, so assert `Some(1)`.
3. tests/integration/guidance.rs ~:186: `assert!(items[0].done && items[3].done && items[6].done);` becomes `assert!(items[0].done && items[1].done && items[4].done);`.
Grep the rest of both files for any other `items[N]` or `next_step` assert and trace it against the new order; fix only what is wrong. Change nothing else.
