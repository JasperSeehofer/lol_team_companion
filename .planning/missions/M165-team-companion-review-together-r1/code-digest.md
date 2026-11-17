# M165 code digest (tree at 7951e05, line numbers re-located 2026-11-13)

Paths are relative to the repo root. Read CLAUDE.md "Security model" first. Rules recap (.claude/rules): bind every value (never format! client data); `type::record('tbl', $key)` with `record_key()`; links are `record<...>`, never id strings; datetimes load as `surrealdb::types::Datetime` (convert with `db::dt_to_string`/`opt_dt`); `.check()`/`check_tx` on writes; unique indexes decide uniqueness under `db::invariant_lock()`; Resources are created before the first `<Suspense>`; `#[server]` fns before the component that calls them; `into_any()` on divergent branches; `get_untracked()` in handlers; `recursion_limit = "512"` stays.

## 1. schema.surql (827 lines)

- Idioms: `DEFINE TABLE IF NOT EXISTS x SCHEMAFULL;`, `DEFINE FIELD IF NOT EXISTS f ON x TYPE ...;`, `DEFINE INDEX IF NOT EXISTS name ON x FIELDS a, b [UNIQUE];`. Every comment block is `-- ...`. Baseline runs on EVERY start; it is idempotent. Team-scoped tables use `team ON x TYPE record<team>` and a plain index on the scope (`team_member_team ON team_member FIELDS team`, schema.surql:517; indexes for stamps at :511-513).
- End of file is :827 (`password_reset_code_unique ...`). Append the new block after it, with a `-- ===` banner comment like the password_reset one (:~816-827).
- Template A, team-scoped (`action_item`, :275-284 verbatim):
```
DEFINE TABLE IF NOT EXISTS action_item SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON action_item TYPE record<team>;
DEFINE FIELD IF NOT EXISTS source_review ON action_item TYPE option<record<post_game_learning>>;
DEFINE FIELD IF NOT EXISTS text ON action_item TYPE string;
DEFINE FIELD IF NOT EXISTS status ON action_item TYPE string DEFAULT 'open';
DEFINE FIELD IF NOT EXISTS assigned_to ON action_item TYPE option<string>;
-- Due date as a calendar day, YYYY-MM-DD (no time zone; checked in db.rs).
DEFINE FIELD IF NOT EXISTS due ON action_item TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON action_item TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS resolved_at ON action_item TYPE option<datetime>;
```
- Template B, post_game_learning (:163-183) fields: team, match_riot_id, game_plan_id, draft_id (option<record<..>>), `what_went_well`/`improvements`/`action_items` `array<string>`, open_notes option<string>, `created_by record<user>`, created_at, win_loss option<string>, rating option<int>, draft_deleted option<bool>, and the M159 stamps:
```
DEFINE FIELD IF NOT EXISTS scheduled_match ON post_game_learning TYPE option<record<scheduled_match>>;   -- :180
DEFINE FIELD IF NOT EXISTS series ON post_game_learning TYPE option<record<series>>;                      -- :181
DEFINE FIELD IF NOT EXISTS game_number ON post_game_learning TYPE option<int> ASSERT $value = NONE OR ($value >= 1 AND $value <= 5);  -- :182
DEFINE FIELD IF NOT EXISTS scheduled_match_deleted ON post_game_learning TYPE option<bool>;               -- :183
DEFINE INDEX IF NOT EXISTS post_game_learning_scheduled_match ON post_game_learning FIELDS scheduled_match;  -- :513
```
- ASSERT style examples: `TYPE string ASSERT string::len($value) >= 1 AND string::len($value) <= 100;` (proposal.title), `TYPE string ASSERT $value IN ['scrim','match']` (scheduled_match.kind :592), `ASSERT $value = NONE OR (...)` for options.
- Unique indexes (copy form): `DEFINE INDEX IF NOT EXISTS proposal_vote_unique ON proposal_vote FIELDS proposal, user UNIQUE;` (:774); `survey_prompt_unique ... FIELDS user, survey, occurrence UNIQUE` (:769); `user_guidance_unique ... FIELDS user, key UNIQUE` (:796); `replay_team_match ON replay FIELDS team, match_id UNIQUE` (:457); `board_version_unique ... FIELDS board, version UNIQUE` (:500). NONE values do not collide in a unique index.
- Migration needed for a NEW table with a unique index? NO. docs/ARCHITECTURE.md:73-79 "How to add a migration": a new table, a new `option<T>` field or a plain index goes in schema.surql only; migrations are for backfills, type changes, renames and unique indexes on EXISTING data. Latest migration is v16 (src/server/migrations.rs:180); do not add one (v17 reserved for M159 R2 item 8). New fields on EXISTING tables must be `option<T>` (rule 9).
- scheduled_match is :589-596+ (team, starts_at, kind, format, opponent, notes, brief_read_by array<record<user>> DEFAULT [], series, created_by).

## 2. db.rs (8793 lines) patterns

- `DbError` (:34-48): `Surreal(#[from] surrealdb::types::Error)`, `NotFound` (missing OR out of scope, same error), `Invalid(String)` (shown to the user as written), `Other(String)`. `type DbResult<T>` (:50).
- Helpers: `pub fn record_key(id:&str, table:&str)->String` (:55; strips `table:`), `dt_to_string(&Datetime)` (:69), `opt_dt(Option<Datetime>)` (:74), `opt_key(Option<String>, table)->Option<String>` (:81, blank = None), private `opt_link(Option<RecordId>)->Option<String>` (:88, `.to_sql()`), private `struct IdRecord { id: RecordId }` (~:292), `TeamRef`.
- Scope checks (:161, :188):
```rust
pub async fn ensure_team_record(db:&Surreal<Db>, table:&'static str, id:&str, team_id:&str) -> DbResult<()>
pub async fn ensure_user_record(db:&Surreal<Db>, table:&'static str, id:&str, user_id:&str) -> DbResult<()>
```
  (via private `ensure_owned(db, table, id, owner_path, owner_table, owner_id)` which runs `SELECT id FROM type::record($tb,$key) WHERE {owner_path} = type::record($otb,$okey)`; zero rows = NotFound). Child records must be checked through their parent (own helper) — there is no `ensure_*` for a nested path other than via `owner_path` private fn.
- Membership: `is_team_member(db,&user_id,&team_id)->DbResult<bool>` (:242), `is_team_leader` (:253), `is_team_coach`, `is_team_leader_or_coach` (:278). `get_user_team_id(db,&user_id)->DbResult<Option<String>>` (:1505; returns "team:xxx"). `get_user_team_with_members(db,&user_id)->DbResult<Option<(Team, Vec<TeamMember>)>>` (:1634); `models::user::TeamMember { user_id, username, role, roster_type, riot_summoner_name }` (src/models/user.rs:42); roster_type values include 'coach'. `remove_team_member(db,&team_id,&user_id)` :1532 (a member who leaves must be handled for open sessions).
- `invariant_lock` (:141): `let _guard = db::invariant_lock().await;` — hold for whole check+write, never nest, no await on other lock. Used e.g. `redeem_team_invite` (:1032-1085). Take it for any write whose rule spans read+write (one entry per user and prompt, one vote per user and card, reveal/close transitions, 'at most 3 accepted').
- Db* struct → model conversion, verbatim (DbActionItem, :4274-4301):
```rust
#[derive(Debug, Deserialize, SurrealValue)]
struct DbActionItem { id: RecordId, team: RecordId, source_review: Option<RecordId>, text: String,
    status: String, assigned_to: Option<String>, due: Option<String>,
    created_at: Option<Datetime>, resolved_at: Option<Datetime> }
impl From<DbActionItem> for ActionItem {
    fn from(a: DbActionItem) -> Self { ActionItem { id: Some(a.id.to_sql()), team_id: a.team.to_sql(),
        source_review: opt_link(a.source_review), text: a.text, status: a.status, assigned_to: a.assigned_to,
        due: a.due, created_at: opt_dt(a.created_at), resolved_at: opt_dt(a.resolved_at) } } }
```
- Team-scoped list (:4352): 
```rust
pub async fn list_action_items(db:&Surreal<Db>, team_id:&str) -> DbResult<Vec<ActionItem>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db.query("SELECT * FROM action_item WHERE team = type::record('team', $team_key) ORDER BY status ASC, created_at DESC")
        .bind(("team_key", team_key)).await?;
    let items: Vec<DbActionItem> = r.take(0).unwrap_or_default();
    Ok(items.into_iter().map(ActionItem::from).collect()) }
```
  NOTE `take(0).unwrap_or_default()` silently hides deserialisation failures: prefer `r.take(0)?` for new code where an empty list would hide a bug.
- Team-scoped create (:4309, `create_action_item`): `ensure_team_record` on every linked id first, then
```rust
db.query("CREATE action_item SET team = type::record('team', $team_key), text = $text, source_review = IF $review_key = NONE { NONE } ELSE { type::record('post_game_learning', $review_key) }, assigned_to = $assigned_to, due = $due")
  .bind(("team_key", team_key)).bind(("text", text)).bind(("review_key", opt_key(source_review,"post_game_learning")))...
let row: Option<IdRecord> = response.take(0)?; match row { Some(r)=>Ok(r.id.to_sql()), None=>Err(DbError::Other("Failed to create action item".into())) }
```
  Full signature: `create_action_item(db, team_id:&str, text:String, source_review:Option<String>, assigned_to:Option<String>, due:Option<String>) -> DbResult<String>`. `assigned_to` is a USERNAME STRING (no validation anywhere; M165 must add "is a current member's username"). `update_action_item` :4420. `checked_due` :4305 (YYYY-MM-DD).
- Transactions (`create_review_with_action_items`, :3079-3127): `BEGIN TRANSACTION; LET $r = (CREATE ONLY post_game_learning SET ...); FOR $t IN $items { CREATE action_item SET team = ..., text = $t, source_review = $r.id; }; RETURN { id: $r.id }; COMMIT TRANSACTION;`, then `let mut response = check_tx(response)?; let row: Option<IdRecord> = response.take(3)?;` (statement index counts BEGIN=0, LET=1, FOR=2, RETURN=3). Do pre-checks (ensure_team_record, check_stamps) BEFORE the tx.
- `check_tx(IndexedResults) -> DbResult<IndexedResults>` (:462) picks the real cause among "transaction cancelled" errors; `thrown_tag(&DbError)->Option<String>` (:485) returns the tag of `THROW 'TAG'`. Pattern (`redeem_team_invite`, :1041-1075): inside the tx `IF cond { THROW 'IN_TEAM' };`, then
```rust
let mut r = match check_tx(r) { Ok(r)=>r, Err(e)=>return Err(match thrown_tag(&e).as_deref() {
    Some("IN_TEAM") => DbError::Invalid("...".into()),
    _ if e.to_string().contains("team_member_user_unique") => DbError::Invalid("...".into()),  // unique index name
    _ => e }) };
```
  A THROW only aborts statements after it INSIDE BEGIN..COMMIT.
- Review helpers: `new_action_item_texts(db, team_id, review_id:Option<&str>, lines:&[String]) -> DbResult<Vec<String>>` (:3034, private): trims, drops blanks, drops lines equal (case-insens.) to an OPEN/in_progress team item or to any item with `source_review = this review`. `review_item_lines(&PostGameLearning) -> Vec<String>` (:3066, private): `improvements` CHAIN `action_items`. So: the generated review MUST NOT put "didn't go to plan" cards into `improvements` (each would become an action item on create AND on every later `update_review`); put only the <=3 decided texts in `action_items` and create those items with owners yourself (or let the dedup treat them as created). `create_review_with_action_items(db, review:PostGameLearning) -> DbResult<(String, usize)>` (:3079; runs `check_stamps` then tx); `update_review_with_action_items(db, team_id, review) -> DbResult<usize>` (:3130); `update_post_game_learning` (:3189); `check_stamps` (~:215, private; stamps must be the team's, else NotFound); `check_game_number` (:200).
- `get_scheduled_match(db, team_id:&str, match_id:&str) -> DbResult<ScheduledMatch>` (:8169; NotFound for foreign). `delete_scheduled_match` (:8273) is a tx that clears `scheduled_match` + sets `scheduled_match_deleted = true` on draft/game_plan/post_game_learning: a new `review_session.scheduled_match` link MUST get the same treatment there (else dangling link / delete of session). `match_prep_facts` :8330.
- Existing models: `PostGameLearning` (src/models/game_plan.rs:42-72): id, team_id, match_riot_id, game_plan_id, draft_id, what_went_well, improvements, action_items: Vec<String>, open_notes, created_by, win_loss, rating: Option<u8>, draft_deleted, created_at, scheduled_match_id, series_id, game_number: Option<i32>, scheduled_match_deleted (all `#[serde(default)]` after created_by).

## 3. src/pages/post_game.rs (1574 lines)

Server fns (all start `let authz::Ctx { db: surreal, user } = authz::authed().await?;`): `list_reviews` :23, `get_review_candidates(match_id:Option<String>)` :87, `team_review_state(riot_match_id:String)->Result<Option<bool>,_>` :105 (None = not a candidate game; Some(reviewed)), `get_match_facts` :143, `check_review_links(db, team_id, &review)` :247 (private ssr; `ensure_team_record` for `game_plan_id` and `draft_id`), `carry_match_stamp(db, team_id, &mut review)` :272 (private ssr; if no `scheduled_match_id`: take stamp+game from plan, else from draft; then if stamped and no series, copy `series_id` from `get_scheduled_match`; sets `scheduled_match_deleted=false`; the series is NONE in practice), `create_review(review_json:String)->(String, usize)` :315 (team from session, `created_by = user.id`, check links, carry stamp, `db::create_review_with_action_items`, `community::store::note_review_saved`), `update_review` :343, `delete_review` :358 (`db::delete_post_game_learning`), `analyze_patterns(&[PostGameLearning]) -> (Vec<(String,usize)>, Vec<(String,usize)>)` :444 (counts identical lower-cased lines across reviews, >=2, top 5 each, for what_went_well and improvements).
`create_review` shape:
```rust
#[server] pub async fn create_review(review_json: String) -> Result<(String, usize), ServerFnError> {
    use crate::server::authz; use crate::server::db;
    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let team_id = db::get_user_team_id(&surreal,&user.id).await.map_err(authz::server_err)?.ok_or_else(|| ServerFnError::new(authz::NO_TEAM))?;
    let mut review: PostGameLearning = serde_json::from_str(&review_json).map_err(authz::server_err)?;
    review.team_id = team_id.clone(); review.created_by = user.id.clone();
    check_review_links(&surreal,&team_id,&review).await?; carry_match_stamp(&surreal,&team_id,&mut review).await?;
    let created = db::create_review_with_action_items(&surreal, review).await.map_err(authz::server_err)?; ...
```
Page structure (`PostGamePage` :625-1107): `auth_user` Resource + Effect → `redirect_to_login()`; `toast = use_context::<ToastContext>().expect("ToastProvider")`; `query = leptos_router::hooks::use_query_map()`; ALL Resources created up front (`reviews`, `plans`, `drafts`, `candidates` keyed on `?match_id`, `facts`, `thread_res`); editor state `ReviewEd` (:497, Copy struct of RwSignals; `load(&r)`, `build()`, `clear()`, `track()`); `dirty` Memo vs `saved` snapshot; `edit_gen` StoredValue generation counter so a late save does not land in another review; saving = `Callback` + `leptos::task::spawn_local` calling the server fn with `serde_json::to_string(&review)`, success -> `replace_url(nav, "/post-game?review_id=..")`, error -> `toast.push.run(Toast::error(..))`. No ActionForm. View: `<PageShell density=Density::Compact attr:data-feedback-label="Post game"><PageHeader title=.. eyebrow=.. meta=move|| view!{..} actions=move|| view!{..}/>` then a 2-col grid `lg:grid-cols-[16rem_minmax(0,1fr)]` (list left in `<Suspense fallback=SkeletonGroup>` with `Err`/empty/`Ok(list)` match arms using `.into_any()`; editor right). Arrays are edited as ONE multi-line textarea each (`ed.went_well: RwSignal<String>`, `.join("\n")` on load; `lines(&str)->Vec<String>` trims and drops blanks :481 on build) in `<Field id="review-well" label=.. hint="One point per line."><TextField value=ed.went_well rows=7 placeholder=../></Field>` (:1045-1056). List rows render `Button variant=Quiet full_width pressed=Signal::derive(..)` with `ResultGlyph`. Data-testids used: `review-list`, `review-save`, `review-save-state`, `review-share`, `review-thread`.
Match-detail button (the name collision): src/pages/match_detail.rs:933-948: `{team_review.get().and_then(|r| r.ok()).flatten().map(|reviewed| view!{ <Button variant=ButtonVariant::Secondary size=ButtonSize::Touch href=format!("/post-game?riot_match_id={}", review_mid.clone()) attr:data-testid="review-as-team">{ if reviewed {"Open team review"} else {"Review as a team"} }</Button> })}`; `team_review` Resource created at :839 before Suspense (`Resource::new(match_id, crate::pages::post_game::team_review_state)`).

## 4. authz.rs / rate_limit.rs

- `authed() -> Result<Ctx, ServerFnError>` (:409), `pub struct Ctx { pub db: DbHandle, pub user: AppUser }`; `ctx.team()->Result<Option<String>>` :415, `ctx.require_team()` :419 ("Create or join your team first", `NO_TEAM`), `ctx.require_leader()` :424, `ctx.require_leader_or_coach()` :432. Free fns: `current_team_for(&db,&user)`, `require_team(&db,&user)->Result<String,_>` :328, `require_team_member(&db,&user,&team_id)` :335 (non-member = "Not found"), `require_team_leader` :351 (`LEADER_ONLY` = "Only the team leader can do that"), `require_team_leader_or_coach` :370, `require_admin()` :312, `optional_user()`. `AppUser { id, username, email, riot_*, is_admin, .. }` (auth.rs:57). `DbHandle` derefs to `Surreal<Db>`.
- `server_err<E: IntoServerError>(e) -> ServerFnError` (:187): DbError::NotFound -> "Not found", Invalid(msg) -> msg, others logged + generic. Also `authz::internal_error(context, e)`.
- Rate limit calls: `authz::rate_limit(Bucket::X).await?` (per IP, :481); `authz::rate_limit_user(Bucket::X, &user)?` (per user id, sync, :490); `team_cooldown(name, team_id, Duration)` :497; `user_cooldown(name,&user,Duration)` :513.
- `enum Bucket` (src/server/rate_limit.rs:17-60): Login, LoginFailuresPerEmail, Register, Riot, ImageFetch, Graph, Share, Feedback, Community, PublicRead, SurveyCheck, Account, PasswordReset, Guidance, GuidanceCheck, TeamInvite. `fn limit(bucket)->Limit{capacity, refill_per_sec}` (:70) is an exhaustive match: add a new variant in BOTH places, e.g. `Bucket::Review => Limit{ capacity: 60.0, refill_per_sec: 1.0 }` and a poll bucket (short-poll every ~3 s per member needs refill >= 0.5/s and burst ~60; GuidanceCheck is 60 then 2/s; Community 30 then 0.5/s). Multiplier scales both (e2e uses 100).
- In-memory `OnceLock<Mutex<HashMap>>` pattern (authz.rs:265-300 `seen_map`):
```rust
fn seen_map() -> &'static std::sync::Mutex<std::collections::HashMap<String, std::time::Instant>> {
    static SEEN: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, std::time::Instant>>> = std::sync::OnceLock::new();
    SEEN.get_or_init(Default::default) }
... seen_map().lock().unwrap_or_else(|p| p.into_inner()) ...
```
  Limiter (rate_limit.rs): `buckets: Mutex<HashMap<(Bucket,String),State>>`, `cooldowns: Mutex<HashMap<(&'static str,String),Instant>>`, `Limiter::global()` = `static GLOBAL: OnceLock<Limiter>`; purge = `purge_at(now)` retains only entries that still limit anyone (buckets below capacity; cooldowns < 1 h old, `COOLDOWN_RETENTION`), run by `spawn_purge_task(every: Duration)` (`tokio::spawn` + `interval.tick()`, `MissedTickBehavior::Delay`), started in main.rs. Any new in-memory state (e.g. who-is-present) needs a purge too.
- Auth to member helpers: for per-session checks use team from session; NEVER accept team id from the client.

## 5. src/server/account.rs

- `pub enum Treatment { Delete, Tombstone, Anonymise, RemoveFromArray, Leader }` (:56). `pub const USER_LINKS: &[(&str, &str, Treatment)]` (:73-124), entries like:
```
("post_game_learning", "created_by", Treatment::Tombstone),
("champion_pool", "user", Treatment::Delete),
("scheduled_match", "brief_read_by", Treatment::RemoveFromArray),
("proposal_vote", "user", Treatment::Delete),
("password_reset", "created_by", Treatment::Tombstone),
```
  Add one tuple per new `record<user>` field (also `array<record<user>>`; schema scan matches lines containing `record<user>` or `| user>`). Anonymise is bug_report only (test asserts exactly 1).
- `DELETE_USER` (:196-258, runs with `$u`, `$name`, `$former`, `$removed`) — add matching statements in exactly these forms (a test greps for them):
```
DELETE proposal_vote WHERE user = $u; \                       (Delete: "DELETE {t} WHERE {f} = $u")
UPDATE post_game_learning SET created_by = $tomb WHERE created_by = $u; \   (Tombstone: "UPDATE {t} SET {f} = $tomb")
UPDATE scheduled_match SET brief_read_by = array::complement(brief_read_by, [$u]) WHERE $u IN brief_read_by; \  (RemoveFromArray)
UPDATE action_item SET assigned_to = NONE WHERE team IN $my_teams AND assigned_to = $name; \
```
  `$tomb = user:former_member`. Anonymity design warning: whatever is stored for an unattributed card, deleting the author must not leave a link; and an author's attributed text lines "keep their author" (tombstone shows "Former member").
- `DELETE_TEAM` (:155-194, `$t` = team): uses LET lists for child tables (`LET $drafts = (SELECT VALUE id FROM draft WHERE team = $t); ... DELETE draft_action WHERE draft IN $drafts; DELETE draft WHERE id IN $drafts;`) and plain lines `DELETE action_item WHERE team = $t;`, `DELETE scheduled_match WHERE team = $t;`, ... ending `DELETE team_member WHERE team = $t; DELETE $t;`. Add `DELETE review_vote/review_entry/review_proposal WHERE session IN $sessions; DELETE review_session WHERE team = $t;` (children BEFORE parent; a test only enforces the literal `DELETE {t} WHERE {f} = $t` or `DELETE {t} WHERE id IN $` or `UPDATE {t} SET {f} = NONE WHERE {f} = $t` for every `record<team>` field, so a child that has its own `team` field is easiest). Also leader-vs-member deletion logic at :262-330 (`delete_account`: username typed, takes invariant_lock, refuses a leader with other members).
- `EXPORT_SECTIONS: &[(&str, &str)]` (:392-487): (section name, SurrealQL over `$u`), each returns only the user's own rows and fields about the user, e.g.
```
("votes", "SELECT proposal.title AS idea, created_at FROM proposal_vote WHERE user = $u"),
("guidance", "SELECT key, seen_at, muted_at FROM user_guidance WHERE user = $u"),
("team_records_created", "array::flatten([ (SELECT 'draft' AS kind, id, name, created_at FROM draft WHERE created_by = $u), ... (SELECT 'review' AS kind, id, created_at FROM post_game_learning WHERE created_by = $u), ... ])"),
```
  `export_user_data` runs all statements in one query (`LET $u = type::record('user',$user_key);` + every section; section i is result index i+1). A member's own review_entry text/votes belong in a new section (e.g. "review_entries_written", "review_votes"); anonymous cards cannot be exported (no link) — say so in `not_included` or the about text. `export_section_names()` :493.
- Tests: unit (account.rs `mod tests` :533+): `every_user_link_has_a_treatment` (schema `record<user>` lines vs USER_LINKS, both directions), `the_deletion_query_covers_every_link` (grep of the exact needle strings above), `anonymised_rows_lose_what_their_author_wrote`, `every_route_with_an_id_is_cut_from_anonymised_reports` (see 8: a `path!("/x/:id")` route must be added to `BUG_REPORT_PAGES_WITH_IDS`, and `found == len`), `deleting_a_team_covers_every_team_link`. Integration tests/integration/account.rs: `world(db)` creates lena (leader), alice (member), otto (other team) then `seed_rows(db,user,leader,team,tag)` (:55-103) runs ONE big query creating one row per USER_LINKS table (e.g. `CREATE action_item SET team = $t, text = 'do it ' + $tag, assigned_to = $name;`) — add rows for the new tables there or `the_seed_covers_every_user_link` (:140) fails (needs count > 0 per link except Leader); `export_holds_every_section_and_nothing_of_others` (:156) requires EVERY export section non-empty for alice and asserts own markers present and otto's absent (add markers to the lists :171-190); `deletion_leaves_no_row_pointing_at_the_user` (:204); also asserts leader-delete removes team rows.

## 6. src/pages/legal/datenschutz.rs

- `pub const STORED: &[(&str, &[&str])]` (:18+): (area, items). "Your team" block verbatim (:46-54):
```
    (
        "Your team",
        &[
            "The team's name and region, its members with their place on the roster (starting five, substitute, coach) and position, and join requests with their status.",
            "What the team writes: drafts with picks and bans, series of drafts (best-of sets), draft trees, ban priority lists, game plans with checklists and checklist templates, reviews, action items (text, owner, due day), Sand Table boards with their saved versions, team notes (with the text before the last edit, and deleted notes for one day, so they can be restored), notes on champion pairs, and the next match (time, opponent, notes, who marked the brief read).",
            "Opponent dossiers: player names, Riot IDs and Riot account ids, champions seen, mastery and role data read from Riot, and your notes.",
            "Share links: a random token, what it opens, who made it, when it expires, and whether it was revoked.",
        ],
    ),
```
  Add the review-together sentence to the "What the team writes" string (or a new item): written cards, +1s, merges, who wrote which attributed card (NOT who wrote an anonymous card), proposal votes, owners.
- Test mod (:244+): `const LISTED: &[&str] = &["user","sessions","invite","team","team_member","join_request","draft",... "post_game_learning","action_item",... "proposal","proposal_vote","ledger_month","site_text","champion_trait"]` (:248-297). Tests: `the_listed_tables_exist` (every LISTED/NOT_PERSONAL name is in schema), `every_table_is_accounted_for` (every `DEFINE TABLE IF NOT EXISTS x` in schema.surql must be in LISTED or `NOT_PERSONAL`) -> add `"review_session","review_entry","review_vote","review_proposal"` (and any other new table) to LISTED. `schema_tables()` parses lines starting `DEFINE TABLE IF NOT EXISTS ` exactly.

## 7. Match Day review step and entry points

- src/pages/home.rs: `thread_suffix(m:&ScheduledMatch, game:Option<i32>)->String` :347 returns `"&match_id=<id>&game=<g>"` (or without game). `prep_steps(md:&MatchDay)->Vec<PrepStep>` :357. Review step :487-531 (verbatim core):
```rust
let reviewed: Vec<i32> = if md.reviewed_games.is_empty() && md.review_after.is_some() { vec![1] } else { md.reviewed_games.clone() };
let games = review_games(&m.format, &md.played_games, &reviewed);
let complete = reviews_complete(&games);  let series = game_count(&m.format) > 1;
let readout = review_readout(&games);
let to_write = games.iter().find(|(_, s)| *s == GameReview::ToDo).map(|(g,_)| *g).unwrap_or(1);
let review = match (&md.review_after, complete, started) {
    (Some(r), true, _) => PrepStep::new("After the game: review", PrepState::Done).detail(..).status("Written")
        .link(format!("/post-game?review_id={}", r.id), "Open the review"),
    (None, _, false) => PrepStep::new("After the game: review", PrepState::Todo).detail("Write it right after the match, while it is fresh.").status("After the game"),
    (written, _, _) => { let carried = thread_suffix(m, series_game(&m.format, to_write));
        let href = match (md.plans.first(), md.drafts.first()) { (Some(p),_) => format!("/post-game?plan_id={}{carried}", p.id),
            (None, Some(d)) => format!("/post-game?draft_id={}{carried}", d.id), _ => format!("/post-game?{}", &carried[1..]) };
        PrepStep::new(..., PrepState::Todo).detail(..).status(..).link(href, "Write the review") } };
let mut steps = vec![brief, drafts, plan, items, review];  // then the first Todo/Attention step with href becomes Next
```
  `PrepStep` (src/components/ui/prep.rs:28-70) has ONE `href`/`action` (`.link(href, action)`); the Next step renders its action as a primary/quiet link button (prep.rs:~162), non-next steps link the title. There is NO second-link slot: a "Review together" sibling needs a new optional field (e.g. `extra: Option<(String,String)>`) rendered in prep.rs (and unit/gallery coverage), or a separate link rendered by home.rs next to `<PrepSequence>`. Do not break `design` gallery (`design_domain.rs`).
- src/models/match_day.rs: `ScheduledMatch { id, starts_at, kind, format, opponent_id, opponent_name, notes, brief_read_by, series_id }` :35; `game_count(format)->i32` :204 (bo1=1, bo3=3, bo5=5); `parse_game(Option<&str>, format)->Option<i32>` :215 (None for Bo1); `enum GameReview {Done,ToDo,NotPlayed}` :384; `review_games(format, played:&[i32], reviewed:&[i32])->Vec<(i32,GameReview)>` :396; `reviews_complete(&games)->bool` :415; `review_readout(&games)->String` :421; `MatchDay` struct :514 (next, drafts, plans, `review_after: Option<LinkRef>`, played_games, reviewed_games, ... `viewer_id`, `member_count`, `chosen`); `LinkRef {id,name}` :150.
- `home::get_thread_match(match_id)->Result<ThreadMatch,_>` (home.rs:249; `ctx.require_team()` + `db::get_scheduled_match`; Not found for foreign) — reuse for validating `?match_id=` on the new page. A session's game key = `scheduled_match` + `game_number` (series NONE in practice; copy only when match has one).
- Match detail "Review as a team": see section 3 end.

## 8. Timers, query params, routes, nav

- Timer pattern (tree_drafter.rs:308-322 verbatim):
```rust
#[cfg(feature = "hydrate")]
{
    use wasm_bindgen::prelude::*;
    let cb = Closure::once(move || loading.set(false));
    if let Some(win) = web_sys::window() {
        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 0);
    }
    cb.forget();
}
#[cfg(not(feature = "hydrate"))]
loading.set(false);
```
  Debounced re-arm with cancel (draft_graph.rs:~590-612, draft.rs:1515-1543): Effect captures values eagerly, `win.clear_timeout_with_handle(id)` on the stored handle (`RwSignal<Option<i32>>`), `Closure::once(move || spawn_local(async { ... if generation.get_value() == gen {..} }))`, `set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 250)` -> store `Ok(id)`; `cb.forget()`; non-hydrate branch `let _ = json;` to silence unused warnings. NO `set_interval` exists anywhere: a short poll = a self-re-arming `set_timeout` (re-arm in the completion of the previous request, after which stop on `on_cleanup`, a generation counter, or session closed). Clean up with `on_cleanup(...)` (used in graph_canvas.rs:441, command.rs:165, status.rs:241 with `StoredValue::new_local`). Fire only after hydration (inside `Effect::new` + `#[cfg(feature="hydrate")]`) so SSR never polls. Window listeners need a re-entry guard (rule 61); Closure::forget leaks on every re-arm unless you keep one handle.
- Page visibility: NO code reads document visibility today. web-sys features in Cargo.toml:55 include `Document`, `Window`, `Event`, `EventTarget` (no `VisibilityState`; `document.hidden()` works with `Document` alone; `visibility_state()` would need the feature added; a `visibilitychange` listener via `add_event_listener_with_callback` works with `EventTarget`).
- Query params: `let query = leptos_router::hooks::use_query_map();` then `query.read().get("match_id").filter(|m| !m.trim().is_empty())` as the Resource source key (post_game.rs:635-649; home.rs:554/596; game_plan.rs:697). Path params: `use_params_map()` (`params.read().get("id")`, match_detail.rs:826-834). `crate::pages::navigate::{navigator, go, replace_url, redirect_to_login}`; `replace_url(nav, "/post-game?review_id=..")`.
- Routes: src/app.rs:119-160 `<Routes fallback=|| view! { <NotFound /> }>` with `<Route path=path!("/post-game") view=PostGamePage />`; public/admin ones carry `ssr=SsrMode::Async` for admin. Add `pub mod review_together;` to src/pages/mod.rs and the Route. USE A QUERY PARAM (`/review-together?session=<id>` or `?match_id=&game=`), NOT `/review-together/:id`: `account.rs` test `every_route_with_an_id_is_cut_from_anonymised_reports` requires every `path!("...:x")` prefix in `BUG_REPORT_PAGES_WITH_IDS` (currently `/team/player/`, `/s/`, `/match/`, `found == len == 3`), and `Hint::routes`/nav treat paths literally.
- nav.rs: `PAGES: &[NavPage]` :~44-102 (`page(path,label,hub,keywords)`), `HUBS` (today, prep, review, team, you), `hub_for_path(path)` :127 (special cases `/` -> today, `/match/` -> review, `/team/player` -> team; then prefix match over PAGES, `"/draft-x"` -> ""), `MENU_GROUPS: &[(&str,&[&str])]` :347 (Today / After the game / Team / You). Unit tests (nav.rs:843-915): `every_page_has_a_hub_and_lands_in_it`, `the_phone_menu_lists_every_page_once` (listed.len() == PAGES.len(), each path once), `the_command_bar_covers_every_page`. A sub-route does NOT need a PAGES entry (like `/match/:id`, `/personal-learnings/new`); if you want it in the Review hub without a menu entry add `if path.starts_with("/review-together") { return "review"; }` beside the `/match/` special case (:127-135) and a line in the first test. If you DO add a PAGES entry also add it to exactly one MENU_GROUPS list and it appears in the command bar automatically. A new page also needs a smoke entry: e2e/tests/pages.spec.ts `AUTHED_PAGES` (CLAUDE.md "New page"), and `data-feedback-label="Review together"` on the PageShell.
- Guidance: a `Hint` registered in models/guidance.rs: enum :68, `Hint::ALL: [Hint; 7]` :88 (bump the array length), `key()` ("hint.xxx") :~97, `routes()` :~117 (paths, trailing `/` = prefix), plus a "used" rule in `HintUse`; place `<FirstUseHint hint=Hint::X title="..">text</FirstUseHint>` in the page (components/guidance.rs:215; Callout role=status, "Got it" mutes; one per page; none on admin pages). For a facilitator nudge ("one player has been quiet in the last 3 reviews") use `NudgeKind` (models/guidance.rs:508-540: variants, `ALL: [NudgeKind;5]`, `key()` "nudge.*", `from_key`; `Nudge{kind,text,action,href}`; `NudgeFacts` struct) and `components::guidance::NudgeLine(nudge: Nudge, on_change: Callback<()>)` (:533: a `<p data-nudge=key>` with text, a link `action`, and a "Don't show this again" button calling `mark_then(key, true, ..)`). Mute keys are stored by `mark_guidance(key, mute)` and accepted only by `models::guidance::is_known_key(key, intros)` (:709: WELCOME, CHECKLIST, `Hint::from_key`, `NudgeKind::from_key`, build-time intros) so a new key MUST be in the Hint/NudgeKind enum. Nudges are chosen by pure rules in models/guidance.rs from `NudgeFacts` and rendered on Match Day; a facilitator-only nudge on the new page can reuse `NudgeLine` with a constructed `Nudge` plus a new NudgeKind (also add its text to the ARCHITECTURE/DESIGN 6.11 lists, and unit tests for `from_key` round trip). Guidance tests: tests/integration/guidance.rs.

## 9. UI primitives (src/components/ui/mod.rs re-exports)

Exports: avatar{AvatarShape,AvatarSize,AvatarState,ChampionAvatar,UserAvatar}, badge{Badge,BadgeTone}, bar{ProportionBar,BarSegment,BarInk,BarSize,bar_widths}, button{Button,ButtonSize,ButtonType,ButtonVariant,Force,Keycap}, callout{Callout,CalloutTone}, canvas{..}, command, dialog{ConfirmDialog,ConfirmTone,Dialog,DialogSize}, draft, empty{EmptySize,EmptyState,NoTeamState}, field{Combobox,Field,FieldContext,FieldSize,InputKind,Select,SelectOption,TextField,ctrl_enter_submits,enter_submits}, layout{Density,Edge,HeadingLevel,Layer,Pad,PageHeader,PageShell,PageWidth,SectionHeader,Surface,SurfaceTag,page_title}, markers{GameResult,GlyphSize,ResultGlyph,Role,RoleLabel,RoleLabelStyle,SideMarker,SideText}, menu, prep{PrepEmphasis,PrepSequence,PrepState,PrepStep}, readout{StatReadout,..}, skeleton{Skeleton,SkeletonGroup,SkeletonShape}, status{LoadFailed,SaveState,SaveStatus,SaveTracker,SAVE_FAILED_TOAST,save_failure_reason,use_save_state,use_unsaved_guard}, table, tabs{TabItem,TabPanel,Tabs}, text{Text,TextRole,TextTag,Tone}, toast{Toast,ToastAction,ToastContext,ToastEntry,ToastKind,ToastProvider}, toggle{Checkbox,Segment,SegmentSize,SegmentedControl,Switch}, tooltip. Every `#[component]` needs a doc comment with an `A11y:` line (test in mod.rs).
- `Text(#[optional] role: TextRole, tone: Tone, tag: TextTag, class: String, children)`; `TextRole`: Display, Page, Title, Heading, Reading (16px, 1.6 leading: for review text), Body (default), Label, Caption, Overline, Figure, FigureLarge, Note (serif italic: "annotations ONLY" per DESIGN — the mission wants a member's own words in this serif; check docs/DESIGN.md before using it for cards, an exception may be needed); `Tone`: Inherit, Primary, Secondary, Muted, Accent, Success, Warning, Danger, Info; `TextTag`: Span, P, Div.
- `Button(variant: ButtonVariant{Primary,Secondary(default),Quiet,Danger}, size: ButtonSize{Sm,Md,Lg,Touch}, kind: ButtonType{Button,Submit,Reset}, disabled: Signal<bool>, loading: Signal<bool>, on_click: Option<Callback<MouseEvent>>, href: Option<String> (renders <a>), icon, shortcut: &'static str, aria_label, force, full_width: bool, pressed: Option<Signal<bool>> (toggle, aria-pressed: use for +1), reserve: &'static [&'static str], children)`. One Primary per view. Pass test ids as `attr:data-testid="..."`.
- `Badge(tone: BadgeTone{Neutral,Accent,Success,Warning,Danger,Info,SideBlue,SideRed,Win,Loss}, label: Option<String>, children)`.
- `Callout(tone: CalloutTone{Info,Success,Warning,Danger}, title: Option<String>, action: Option<ViewFn> (`action=move || view!{..}`), children)`.
- `Field(id: String, label: String, hint: Option<String>, error: Signal<Option<String>>, required: bool, children)` + `TextField(value: Option<RwSignal<String>>, kind: InputKind{Text,Email,Password,Search,Number,Date,Time,Url}, size, name, placeholder, autocomplete, minlength, maxlength: Option<u32>, required, disabled: Signal<bool>, rows: Option<u32> (Some => multi-line textarea), id, aria_label, force, on_keydown: Option<Callback<KeyboardEvent>>, on_blur: Option<Callback<()>>, combobox, class: &'static str)`. Use `maxlength` and enforce the same limit on the server.
- Save state: `use_save_state() -> SaveTracker {state: RwSignal<SaveState>, .. .saving(), .saved_steady(), .saved() (fades), .show(SaveState), .failed(reason)->bool (true once per failure: show the toast), .is_failing()}`; `SaveState {Idle, NotSaved, Unsaved, Saving, Saved, Failed(String)}`; `<SaveStatus state=Signal on_retry=Option<Callback<()>> />` (role=status, one per page, in PageHeader `meta`); `save_failure_reason(&ServerFnError)->String`; `use_unsaved_guard(Signal<bool>)` (beforeunload).
- `UserAvatar(name: String, size: AvatarSize{Xs,Sm,Md(default),Lg}, label: Option<String>)` (monogram; role=img with `label` when it stands alone). `SectionHeader(title: String, level: HeadingLevel{H2,H3}, eyebrow, meta: Option<String>, actions: Option<ViewFn>, id: Option<String>)`. `PageShell(width: PageWidth{Narrow,Reading,Standard,Wide(default),Full}, density: Density{Compact,Comfortable}, class, children)`; `PageHeader(title: TextProp (closure ok), eyebrow, meta: Option<ViewFn>, actions: Option<ViewFn>, document_title)`. `Surface(layer, edge, pad, tag, label, labelled_by, class, children)`. `EmptyState(message, title, level, size: EmptySize, cta label+href..)`. `Switch(label, checked: RwSignal<bool>, label_hidden, show_state, disabled, on_change, force)`, `Checkbox(label, checked, strike, disabled, on_change, id, name, required)`, `SegmentedControl(id, label, options: Vec<Segment>, value: RwSignal<String>, size, on_change, disabled, manual..)`, `Segment::new(value,label)`.
- Toast: `let toast = use_context::<ToastContext>().expect("ToastProvider");` `toast.push.run(Toast::success("..")|Toast::error(format!(..))|Toast::info(..))`, `toast.show.run((ToastKind, String))`; undo variant = Toast with `.action` (ToastAction; see action-items/team notes for the builder). `ConfirmDialog(id, open: RwSignal<bool>, title, message, confirm_label, tone: ConfirmTone::Destructive, on_confirm: Callback)`. `Skeleton(shape: SkeletonShape::{Line,Block}, lines, size, width)` in `SkeletonGroup(label)`; `LoadFailed`.
- Design rules for new views: only tokens (see 11); no emoji; headings plain; `figures` class for numbers.

## 10. Tests and e2e

- tests/integration/main.rs (`#![cfg(feature = "ssr")]`): `mod account; mod common; mod community; mod data_dragon; mod datetimes; mod db_champion_pool; mod db_champion_trends; mod db_drafts; mod db_game_plan_pipeline; mod db_goal_progress; mod db_lp_history; mod db_opponents; mod db_teams; mod db_tree; mod db_users; mod graph; mod guidance; mod match_day; mod match_thread; mod migrations; mod password; mod retention; mod sand_table; mod security; mod team_invites; mod transactions;` — add `mod review_together;` (alphabetical) with a new file. Cross-tenant cases required for every new team/user DB function (security.rs, match_day.rs style).
- `common::test_db() -> Arc<Surreal<Db>>` (tests/integration/common.rs): in-memory `Mem` db, `db::prepare(&db)` = baseline schema + all migrations. Short test (team_invites.rs:13-40):
```rust
#[tokio::test]
async fn a_team_code_registers_the_account_into_the_team() {
    let db = common::test_db().await;
    let lead = db::create_user(&db, "lead".into(), "lead@example.com".into(), "h".into()).await.unwrap();
    let team = db::create_team(&db, &lead, "Northgate".into(), "EUW".into()).await.unwrap();
    let carl = db::create_user(&db, "carl".into(), "carl@example.com".into(), "h".into()).await.unwrap();
    db::join_team(&db, &carl, &team).await.unwrap();   // member join
    assert_eq!(db::get_user_team_id(&db, &carl).await.unwrap().as_deref(), Some(team.as_str()));
```
  (`use crate::common; use lol_team_companion::server::db::{self, DbError};`; ids are "user:xxx"/"team:xxx" strings; foreign-scope calls assert `matches!(.., Err(DbError::NotFound))`.) `db::create_scheduled_match`-style fns for a match fixture: see tests/integration/match_day.rs / match_thread.rs. Concurrency tests live in transactions.rs/security.rs (spawn tasks, assert one wins).
- e2e (e2e/tests/fixtures.ts): `import { test, expect } from "./fixtures"`. `authenticatePage(page)` registers `pages_<ts>_<worker>x<n>@test.invalid` at `/auth/register?invite=${INVITE_CODE}` (`input[name=username|email|password]`, `#register-terms`, `main button[type=submit]`), waits for Home `/`, returns `{email, username}`; password `TEST_PASSWORD = "Test1234!long"`. `createTeam(page, name)` goes to `/team/roster`, fills `input[name="name"]`, clicks `getByTestId("create-team")`, waits for `/team/dashboard`. Fixtures `authedPage`, `teamPage` (authenticate + createTeam `AuditTeam_<ts>`), `adminPage` (`signInAsAdmin`); `page` fixture stubs `/img/(champion|item)/`. Helpers e2e/tests/helpers.ts: `navigateTo(page, route)`, `captureErrors`, `filterRealErrors`, `captureHydrationWarnings`.
- Several users: use `browser.newContext()` per user (see team-invites.spec.ts): leader `const leader = await (await browser.newContext()).newPage(); await authenticatePage(leader); await createTeam(leader, name);` then `mintLink(leader, position)` (team page `[data-team-invites]`, `#team-invite-uses`, `#team-invite-days`, `#team-invite-position`, `getByTestId("mint-team-invite")`, link in `getByTestId("team-invite-link").locator("code")` containing `/auth/register?invite=`); each new player: `await player.goto(link)`, fill username/email/password, `check("#register-terms")`, click submit, `waitForHome(player)`, `waitForLoadState("networkidle")`; they land in the team "not placed yet" (welcome says `You are on TEAM`). Invite `#team-invite-uses` accepts more than 2 (use 4 for a five-person team). Scheduling a match (match-thread.spec.ts:240-252): `navigateTo(page,"/")`, `getByTestId("add-next-match").click()`, dialog "Add the next match": `#match-date` (fill YYYY-MM-DD), `#match-time` "19:00", optional `#match-opponent`, `getByTestId("save-next-match")`; then h1 "Scrim vs <opp>"; Match Day steps `[data-prep] li`; review link `[data-prep] a[href^="/post-game?"][href*="match_id=scheduled_match:"]`. A match must exist with a started time for the review step (CI seeds only apply to some specs; the spec has to schedule a match starting in the past or tomorrow and be able to open the session regardless of `started`).
- CI (.github/workflows/ci.yml:182-184, step "Playwright smoke, hydration gate, auth, admin, legal, account, guidance and team invites"): `npx playwright test smoke.spec.ts hydration-no-panic.spec.ts auth.spec.ts admin-invites.spec.ts cross-tenant.spec.ts gallery.spec.ts community-public.spec.ts community-gates.spec.ts bug-triage.spec.ts legal.spec.ts account.spec.ts guidance.spec.ts team-invites.spec.ts password.spec.ts frames-m141.spec.ts match-detail.spec.ts sortable-tables.spec.ts post-game-fill.spec.ts champion-pool-moves.spec.ts enter-submits.spec.ts bug-report.spec.ts feedback-attachment.spec.ts match-day-phone.spec.ts frames-m148.spec.ts login-next.spec.ts save-failure.spec.ts ui-icons.spec.ts og-meta.spec.ts hero-matchup.spec.ts frames-m152.spec.ts match-thread.spec.ts --retries=1 --reporter=list,json,github` — append `review-together.spec.ts` before `--retries=1`. (The release-image step at :470 runs only smoke/hydration/auth/legal/account.) CLAUDE.md "Commands" and "Testing" lists also name the CI specs: update them too. `MISSION_FRAMES_DIR` env: PNGs written there at top level or `after/` are committed as frames/after (use for the atelier frame strips); `e2e/tests/frames-m152.spec.ts` is the frame-spec model.
- Also run: `cargo test --features ssr --lib`, `cargo test --features ssr`, both clippy targets with `-D warnings`, `cargo fmt --check`, `./scripts/lint_design.sh`.

## 11. scripts/lint_design.sh (hard zero, src/**/*.rs excluding server, models, bin; escape hatch `lint:allow(<rule>)` in a same-line comment)

Rules: `palette` (raw Tailwind palette classes like `bg-red-500`, `text-white/black`); `hex` (any hex colour literal outside comments); `px-text` (`text-[Npx]` under 12px / tiny rem); `rounded-full` (outside src/components/ui/); `blur` (backdrop-blur/filter); `gradient`; `drop-shadow`/`text-shadow`; `pulse` (`animate-pulse` outside skeleton.rs); `ornament` (old fantasy font/ornament names); `emoji` (pictographs; typographic stars/check/cross U+2605/2606/2713/2715/2717 allowed; write `\u{2192}`-style arrows via escapes as existing code does); `region` (`region ==`, `data-theme`...); `external-url` (any `http(s)://` host in client code; also scans models); `faded-text` (`text-muted/60` etc.); `faint-outline` (`border-outline/NN`); `legacy-token` (`bg-base|surface|elevated|divider|dimmed|accent-contrast|overlay`); `g12` (`outline-none` needs a `focus-visible` replacement on the same line). Allowed tokens: `bg-page|ink-1|ink-2|ink-3|canvas`, `border-rule|outline`, `text-primary|secondary|muted`, `bg-accent text-accent-ink`, `side-blue|side-red`, `win|win-ink|loss`, `success|warning|danger|info`; type `text-caption`(12px floor)...`text-headline`, `eyebrow`, `figures`, `note` (serif italic; DESIGN reserves it for annotations/team notes — a decision for the atelier/round if cards use it).

## 12. CHANGELOG.md

- Head (:1-9): "# Changelog", Keep a Changelog intro, then the paragraph defining `Introduce:` lines, then `## [Unreleased]` (:9) whose first entry is `### Milestone M159: Match thread (2026-10-18)` (:11). Add a new heading ABOVE it: `### Milestone M165: Review together (YYYY-MM-DD)` (date required by a unit test; slug of heading-without-date is the anchor, never rename later), sections `#### Added` / `#### Changed` / `#### Fixed`, bullets are user-facing prose.
- Introduce line format (not a bullet, a bare line inside the entry; unit test parses every one; examples :112, :131, :197):
`Introduce: /team/dashboard | Invite a player | Team leaders: create a link here that signs a new player up straight into your team.`
  i.e. `Introduce: <route> | <Feature name> | <how to try it>.` Route is a path used by `IntroNote`/`NewMark` (e.g. `/review-together`); accounts created before the entry's date see the one-time New note for 180 days. The route must exist as a page path the nav/guidance can match; `models::guidance::is_known_key` accepts `intro:<slug>:<route>` keys of the running build automatically.
- docs to update with the change: docs/ARCHITECTURE.md (new section "Review together", tests list), CLAUDE.md (routes table, tables, CI spec list, rate bucket), docs/DESIGN.md if a new primitive/pattern, docs/LAUNCH-CHECKLIST.md only if privacy-relevant (the privacy page is the privacy source of truth: STORED + LISTED).

## Gotchas found while reading (read before designing)
1. `review_item_lines` turns EVERY `improvements` line into an action item on create and on each `update_review` -> the generated review must keep "didn't go to plan" cards out of `improvements` (mission context says so).
2. `assigned_to` on action_item is a plain username string with no validation; deleting an account NONEs it by name inside the user's teams. Owner validation must be added server-side (members from `get_user_team_with_members`).
3. Anonymous column: any `record<user>` field on an entry row is a DB-level author link even if the UI hides it (export, `USER_LINKS`, admin/DB readers). For "no team view can link a card to an author" decide whether anonymous cards store no author at all (then silent-round "who has written" must live in a separate presence row/count keyed by user+session+prompt, without the card id) — the account.rs/export tests force every `record<user>` to be declared and every export section non-empty.
4. `PrepStep` has a single link; `/post-game` page already uses `?match_id=&game=` thread links (`thread_suffix`).
5. Routes with `:param` must be added to `BUG_REPORT_PAGES_WITH_IDS` (count-checked); prefer query params.
6. Hydration: poll only from an Effect under `#[cfg(feature = "hydrate")]`; create Resources before Suspense; polled payloads must never include other members' text before reveal (poll fn returns counts only until `revealed`).
7. `team_review_state`/"Review as a team" (match_detail) already means "open the single-author editor"; the mission item 3 must say which of the two names wins.
