-- Baseline schema. Applied on every start; every statement must be
-- idempotent (IF NOT EXISTS). Backfills, type changes and unique constraints
-- that need clean data live in src/server/migrations.rs, which runs right
-- after this file. Where a migration tightened a definition, the line below
-- says so: IF NOT EXISTS keeps the tightened version on existing databases.

-- Applied migrations (src/server/migrations.rs)
DEFINE TABLE IF NOT EXISTS schema_migration SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS version ON schema_migration TYPE int;
DEFINE FIELD IF NOT EXISTS name ON schema_migration TYPE string;
DEFINE FIELD IF NOT EXISTS applied_at ON schema_migration TYPE datetime;
DEFINE INDEX IF NOT EXISTS schema_migration_version ON schema_migration FIELDS version UNIQUE;

-- Users
DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS username ON user TYPE string;
DEFINE FIELD IF NOT EXISTS email ON user TYPE string;
DEFINE FIELD IF NOT EXISTS password_hash ON user TYPE string;
DEFINE FIELD IF NOT EXISTS riot_puuid ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS riot_summoner_name ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON user TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS email_unique ON user FIELDS email UNIQUE;
DEFINE FIELD IF NOT EXISTS mode ON user TYPE string DEFAULT 'solo';
DEFINE FIELD IF NOT EXISTS riot_region ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS last_solo_sync ON user TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS theme ON user TYPE string DEFAULT 'demacia'
  ASSERT $value IN ['demacia', 'pandemonium'];

-- Phase 18 region-variants: per-route mode persistence (D-03)
-- No ASSERT constraint — value set is wider (route-scoped) + may grow in future phases
DEFINE FIELD IF NOT EXISTS draft_mode ON user TYPE string DEFAULT 'auto';
DEFINE FIELD IF NOT EXISTS team_dashboard_mode ON user TYPE string DEFAULT 'auto';
DEFINE FIELD IF NOT EXISTS solo_mode ON user TYPE string DEFAULT 'auto';

-- M1: admin flag. option<bool> (NONE = not admin) so users created before
-- this field existed stay valid on UPDATE; ADMIN_EMAILS promotes at login.
-- Migration 3 backfills NONE to false and tightens it to bool DEFAULT false.
DEFINE FIELD IF NOT EXISTS is_admin ON user TYPE option<bool>;

-- M6: lowercase username for case-insensitive uniqueness. Migration 2
-- backfills it, tightens it to string and adds the unique index
-- username_lc_unique. Emails are stored lowercase (migration 1), so
-- email_unique is case-insensitive in practice.
DEFINE FIELD IF NOT EXISTS username_lc ON user TYPE option<string>;
DEFINE INDEX IF NOT EXISTS user_riot_puuid ON user FIELDS riot_puuid;

-- Teams
DEFINE TABLE IF NOT EXISTS team SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON team TYPE string;
DEFINE FIELD IF NOT EXISTS region ON team TYPE string;
DEFINE FIELD IF NOT EXISTS created_by ON team TYPE record<user>;
DEFINE FIELD IF NOT EXISTS created_at ON team TYPE datetime DEFAULT time::now();

-- Team membership
DEFINE TABLE IF NOT EXISTS team_member SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON team_member TYPE record<team>;
DEFINE FIELD IF NOT EXISTS user ON team_member TYPE record<user>;
DEFINE FIELD IF NOT EXISTS role ON team_member TYPE string;
DEFINE FIELD IF NOT EXISTS roster_type ON team_member TYPE string DEFAULT 'sub';

-- Join requests
DEFINE TABLE IF NOT EXISTS join_request SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON join_request TYPE record<team>;
DEFINE FIELD IF NOT EXISTS user ON join_request TYPE record<user>;
DEFINE FIELD IF NOT EXISTS status ON join_request TYPE string DEFAULT 'pending';
DEFINE FIELD IF NOT EXISTS created_at ON join_request TYPE datetime DEFAULT time::now();

-- Cached match data
DEFINE TABLE IF NOT EXISTS match SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS match_id ON match TYPE string;
DEFINE FIELD IF NOT EXISTS queue_id ON match TYPE int;
DEFINE FIELD IF NOT EXISTS game_duration ON match TYPE int;
DEFINE FIELD IF NOT EXISTS game_end ON match TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS team_id ON match TYPE option<record<team>>;
DEFINE INDEX IF NOT EXISTS match_id_unique ON match FIELDS match_id UNIQUE;
DEFINE FIELD IF NOT EXISTS synced_by ON match TYPE option<record<user>>;

-- Per-player match stats
DEFINE TABLE IF NOT EXISTS player_match SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS match ON player_match TYPE record<match>;
DEFINE FIELD IF NOT EXISTS user ON player_match TYPE record<user>;
DEFINE FIELD IF NOT EXISTS champion ON player_match TYPE string;
DEFINE FIELD IF NOT EXISTS kills ON player_match TYPE int;
DEFINE FIELD IF NOT EXISTS deaths ON player_match TYPE int;
DEFINE FIELD IF NOT EXISTS assists ON player_match TYPE int;
DEFINE FIELD IF NOT EXISTS cs ON player_match TYPE int;
DEFINE FIELD IF NOT EXISTS vision_score ON player_match TYPE int;
DEFINE FIELD IF NOT EXISTS damage ON player_match TYPE int;
DEFINE FIELD IF NOT EXISTS win ON player_match TYPE bool;

-- Draft plans
DEFINE TABLE IF NOT EXISTS draft SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON draft TYPE string;
DEFINE FIELD IF NOT EXISTS team ON draft TYPE record<team>;
DEFINE FIELD IF NOT EXISTS created_by ON draft TYPE record<user>;
DEFINE FIELD IF NOT EXISTS opponent ON draft TYPE option<string>;
DEFINE FIELD IF NOT EXISTS notes ON draft TYPE option<string>;
DEFINE FIELD IF NOT EXISTS comments ON draft TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS rating ON draft TYPE option<string>;
DEFINE FIELD IF NOT EXISTS our_side ON draft TYPE string DEFAULT 'blue';
DEFINE FIELD IF NOT EXISTS tags ON draft TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS win_conditions ON draft TYPE option<string>;
DEFINE FIELD IF NOT EXISTS watch_out ON draft TYPE option<string>;
DEFINE FIELD IF NOT EXISTS series_id ON draft TYPE option<record<series>>;
DEFINE FIELD IF NOT EXISTS game_number ON draft TYPE option<int>;
DEFINE FIELD IF NOT EXISTS created_at ON draft TYPE datetime DEFAULT time::now();

-- Picks/bans in a draft
DEFINE TABLE IF NOT EXISTS draft_action SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS draft ON draft_action TYPE record<draft>;
DEFINE FIELD IF NOT EXISTS phase ON draft_action TYPE string;
DEFINE FIELD IF NOT EXISTS side ON draft_action TYPE string;
DEFINE FIELD IF NOT EXISTS champion ON draft_action TYPE string;
DEFINE FIELD IF NOT EXISTS `order` ON draft_action TYPE int;
DEFINE FIELD IF NOT EXISTS comment ON draft_action TYPE option<string>;
DEFINE FIELD IF NOT EXISTS role ON draft_action TYPE option<string>;

-- Game plans
DEFINE TABLE IF NOT EXISTS game_plan SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON game_plan TYPE string DEFAULT '';
DEFINE FIELD IF NOT EXISTS draft ON game_plan TYPE option<record<draft>>;
DEFINE FIELD IF NOT EXISTS team ON game_plan TYPE record<team>;
DEFINE FIELD IF NOT EXISTS our_champions ON game_plan TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS enemy_champions ON game_plan TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS win_conditions ON game_plan TYPE array<string>;
DEFINE FIELD IF NOT EXISTS objective_priority ON game_plan TYPE array<string>;
DEFINE FIELD IF NOT EXISTS teamfight_strategy ON game_plan TYPE string;
DEFINE FIELD IF NOT EXISTS early_game ON game_plan TYPE option<string>;
DEFINE FIELD IF NOT EXISTS top_strategy ON game_plan TYPE option<string>;
DEFINE FIELD IF NOT EXISTS jungle_strategy ON game_plan TYPE option<string>;
DEFINE FIELD IF NOT EXISTS mid_strategy ON game_plan TYPE option<string>;
DEFINE FIELD IF NOT EXISTS bot_strategy ON game_plan TYPE option<string>;
DEFINE FIELD IF NOT EXISTS support_strategy ON game_plan TYPE option<string>;
DEFINE FIELD IF NOT EXISTS notes ON game_plan TYPE option<string>;
DEFINE FIELD IF NOT EXISTS win_condition_tag ON game_plan TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_by ON game_plan TYPE option<record<user>>;
DEFINE FIELD IF NOT EXISTS created_at ON game_plan TYPE datetime DEFAULT time::now();

-- Post-game learnings
DEFINE TABLE IF NOT EXISTS post_game_learning SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON post_game_learning TYPE record<team>;
DEFINE FIELD IF NOT EXISTS match_riot_id ON post_game_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS game_plan_id ON post_game_learning TYPE option<record<game_plan>>;
DEFINE FIELD IF NOT EXISTS draft_id ON post_game_learning TYPE option<record<draft>>;
DEFINE FIELD IF NOT EXISTS what_went_well ON post_game_learning TYPE array<string>;
DEFINE FIELD IF NOT EXISTS improvements ON post_game_learning TYPE array<string>;
DEFINE FIELD IF NOT EXISTS action_items ON post_game_learning TYPE array<string>;
DEFINE FIELD IF NOT EXISTS open_notes ON post_game_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_by ON post_game_learning TYPE record<user>;
DEFINE FIELD IF NOT EXISTS created_at ON post_game_learning TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS win_loss ON post_game_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS rating ON post_game_learning TYPE option<int>;

-- Champion pool (personal champion pool per role)
DEFINE TABLE IF NOT EXISTS champion_pool SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON champion_pool TYPE record<user>;
DEFINE FIELD IF NOT EXISTS champion ON champion_pool TYPE string;
DEFINE FIELD IF NOT EXISTS role ON champion_pool TYPE string;
DEFINE FIELD IF NOT EXISTS tier ON champion_pool TYPE string DEFAULT 'comfort';
DEFINE FIELD IF NOT EXISTS notes ON champion_pool TYPE option<string>;
DEFINE FIELD IF NOT EXISTS comfort_level ON champion_pool TYPE option<int>;
DEFINE FIELD IF NOT EXISTS meta_tag ON champion_pool TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON champion_pool TYPE datetime DEFAULT time::now();

-- Champion notes (structured knowledge base per champion)
DEFINE TABLE IF NOT EXISTS champion_note SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON champion_note TYPE record<user>;
DEFINE FIELD IF NOT EXISTS champion ON champion_note TYPE string;
DEFINE FIELD IF NOT EXISTS role ON champion_note TYPE string;
DEFINE FIELD IF NOT EXISTS note_type ON champion_note TYPE string;
DEFINE FIELD IF NOT EXISTS title ON champion_note TYPE string;
DEFINE FIELD IF NOT EXISTS content ON champion_note TYPE string DEFAULT '';
DEFINE FIELD IF NOT EXISTS difficulty ON champion_note TYPE option<int>;
DEFINE FIELD IF NOT EXISTS created_at ON champion_note TYPE datetime DEFAULT time::now();

-- Draft trees (branching draft plans)
DEFINE TABLE IF NOT EXISTS draft_tree SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON draft_tree TYPE string;
DEFINE FIELD IF NOT EXISTS team ON draft_tree TYPE record<team>;
DEFINE FIELD IF NOT EXISTS created_by ON draft_tree TYPE record<user>;
DEFINE FIELD IF NOT EXISTS opponent ON draft_tree TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON draft_tree TYPE datetime DEFAULT time::now();

-- Nodes within a draft tree
DEFINE TABLE IF NOT EXISTS draft_tree_node SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS tree ON draft_tree_node TYPE record<draft_tree>;
DEFINE FIELD IF NOT EXISTS parent ON draft_tree_node TYPE option<record<draft_tree_node>>;
DEFINE FIELD IF NOT EXISTS label ON draft_tree_node TYPE string;
DEFINE FIELD IF NOT EXISTS notes ON draft_tree_node TYPE option<string>;
DEFINE FIELD IF NOT EXISTS is_improvised ON draft_tree_node TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS sort_order ON draft_tree_node TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS created_at ON draft_tree_node TYPE datetime DEFAULT time::now();

-- Picks/bans within a tree node
DEFINE TABLE IF NOT EXISTS tree_node_action SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS node ON tree_node_action TYPE record<draft_tree_node>;
DEFINE FIELD IF NOT EXISTS phase ON tree_node_action TYPE string;
DEFINE FIELD IF NOT EXISTS side ON tree_node_action TYPE string;
DEFINE FIELD IF NOT EXISTS champion ON tree_node_action TYPE string;
DEFINE FIELD IF NOT EXISTS `order` ON tree_node_action TYPE int;
DEFINE FIELD IF NOT EXISTS comment ON tree_node_action TYPE option<string>;

-- Ban priority list
DEFINE TABLE IF NOT EXISTS ban_priority SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON ban_priority TYPE record<team>;
DEFINE FIELD IF NOT EXISTS champion ON ban_priority TYPE string;
DEFINE FIELD IF NOT EXISTS rank ON ban_priority TYPE int;
DEFINE FIELD IF NOT EXISTS reason ON ban_priority TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON ban_priority TYPE datetime DEFAULT time::now();

-- Opponents
DEFINE TABLE IF NOT EXISTS opponent SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON opponent TYPE string;
DEFINE FIELD IF NOT EXISTS team ON opponent TYPE record<team>;
DEFINE FIELD IF NOT EXISTS notes ON opponent TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON opponent TYPE datetime DEFAULT time::now();

-- Opponent players (5 per opponent)
DEFINE TABLE IF NOT EXISTS opponent_player SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS opponent ON opponent_player TYPE record<opponent>;
DEFINE FIELD IF NOT EXISTS name ON opponent_player TYPE string;
DEFINE FIELD IF NOT EXISTS role ON opponent_player TYPE string;
DEFINE FIELD IF NOT EXISTS riot_puuid ON opponent_player TYPE option<string>;
DEFINE FIELD IF NOT EXISTS riot_summoner_name ON opponent_player TYPE option<string>;
DEFINE FIELD IF NOT EXISTS recent_champions ON opponent_player TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS notes ON opponent_player TYPE option<string>;
DEFINE FIELD IF NOT EXISTS last_fetched ON opponent_player TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS mastery_data_json ON opponent_player TYPE option<string>;
DEFINE FIELD IF NOT EXISTS role_distribution_json ON opponent_player TYPE option<string>;

-- Series (best-of sets)
DEFINE TABLE IF NOT EXISTS series SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON series TYPE string;
DEFINE FIELD IF NOT EXISTS team ON series TYPE record<team>;
DEFINE FIELD IF NOT EXISTS opponent_id ON series TYPE option<record<opponent>>;
DEFINE FIELD IF NOT EXISTS opponent_name ON series TYPE option<string>;
DEFINE FIELD IF NOT EXISTS format ON series TYPE string DEFAULT 'bo1';
DEFINE FIELD IF NOT EXISTS is_fearless ON series TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS notes ON series TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_by ON series TYPE record<user>;
DEFINE FIELD IF NOT EXISTS created_at ON series TYPE datetime DEFAULT time::now();

-- Action items (from post-game reviews)
DEFINE TABLE IF NOT EXISTS action_item SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON action_item TYPE record<team>;
DEFINE FIELD IF NOT EXISTS source_review ON action_item TYPE option<record<post_game_learning>>;
DEFINE FIELD IF NOT EXISTS text ON action_item TYPE string;
DEFINE FIELD IF NOT EXISTS status ON action_item TYPE string DEFAULT 'open';
DEFINE FIELD IF NOT EXISTS assigned_to ON action_item TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON action_item TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS resolved_at ON action_item TYPE option<datetime>;

-- Team notes (shared notebook)
DEFINE TABLE IF NOT EXISTS team_note SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON team_note TYPE record<team>;
DEFINE FIELD IF NOT EXISTS author ON team_note TYPE record<user>;
DEFINE FIELD IF NOT EXISTS author_name ON team_note TYPE string DEFAULT '';
DEFINE FIELD IF NOT EXISTS content ON team_note TYPE string;
DEFINE FIELD IF NOT EXISTS pinned ON team_note TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS created_at ON team_note TYPE datetime DEFAULT time::now();

-- Checklist templates (reusable per team)
DEFINE TABLE IF NOT EXISTS checklist_template SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON checklist_template TYPE record<team>;
DEFINE FIELD IF NOT EXISTS name ON checklist_template TYPE string;
DEFINE FIELD IF NOT EXISTS items ON checklist_template TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS created_at ON checklist_template TYPE datetime DEFAULT time::now();

-- Checklist instances (per-game)
DEFINE TABLE IF NOT EXISTS checklist_instance SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON checklist_instance TYPE record<team>;
DEFINE FIELD IF NOT EXISTS game_plan_id ON checklist_instance TYPE option<record<game_plan>>;
DEFINE FIELD IF NOT EXISTS template_id ON checklist_instance TYPE option<record<checklist_template>>;
DEFINE FIELD IF NOT EXISTS items ON checklist_instance TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS checked ON checklist_instance TYPE array<bool> DEFAULT [];
DEFINE FIELD IF NOT EXISTS created_at ON checklist_instance TYPE datetime DEFAULT time::now();

-- Sessions (tower-sessions via SurrealSessionStore). `data` is the
-- MessagePack record; `expires_at` (unix seconds) is written on every save
-- so expired rows can be purged without decoding them.
DEFINE TABLE IF NOT EXISTS sessions SCHEMALESS;
DEFINE INDEX IF NOT EXISTS sessions_expires_at ON sessions FIELDS expires_at;

-- Closed-beta invite codes (M1). Registration consumes one use atomically.
DEFINE TABLE IF NOT EXISTS invite SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS code ON invite TYPE string;
DEFINE FIELD IF NOT EXISTS created_by ON invite TYPE option<record<user>>;
DEFINE FIELD IF NOT EXISTS created_at ON invite TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS max_uses ON invite TYPE int DEFAULT 1 ASSERT $value >= 1;
DEFINE FIELD IF NOT EXISTS uses ON invite TYPE int DEFAULT 0 ASSERT $value >= 0;
DEFINE FIELD IF NOT EXISTS expires_at ON invite TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS disabled ON invite TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS note ON invite TYPE option<string>;
DEFINE INDEX IF NOT EXISTS invite_code_unique ON invite FIELDS code UNIQUE;

-- Ranked snapshots
DEFINE TABLE IF NOT EXISTS ranked_snapshot SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON ranked_snapshot TYPE record<user>;
DEFINE FIELD IF NOT EXISTS queue_type ON ranked_snapshot TYPE string;
DEFINE FIELD IF NOT EXISTS tier ON ranked_snapshot TYPE string;
DEFINE FIELD IF NOT EXISTS division ON ranked_snapshot TYPE string;
DEFINE FIELD IF NOT EXISTS lp ON ranked_snapshot TYPE int;
DEFINE FIELD IF NOT EXISTS wins ON ranked_snapshot TYPE int;
DEFINE FIELD IF NOT EXISTS losses ON ranked_snapshot TYPE int;
DEFINE FIELD IF NOT EXISTS snapshotted_at ON ranked_snapshot TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS ranked_snapshot_user_queue ON ranked_snapshot FIELDS user, queue_type;

-- Match detail cache (full 10-player scoreboard data)
DEFINE TABLE IF NOT EXISTS match_detail SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS match_id ON match_detail TYPE string;
DEFINE FIELD IF NOT EXISTS participants_json ON match_detail TYPE string;
DEFINE FIELD IF NOT EXISTS game_duration ON match_detail TYPE int;
DEFINE FIELD IF NOT EXISTS game_mode ON match_detail TYPE option<string>;
DEFINE FIELD IF NOT EXISTS fetched_at ON match_detail TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS match_detail_match_id ON match_detail FIELDS match_id UNIQUE;

-- Match timeline events cache
DEFINE TABLE IF NOT EXISTS match_timeline_event SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS match_id ON match_timeline_event TYPE string;
DEFINE FIELD IF NOT EXISTS timestamp_ms ON match_timeline_event TYPE int;
DEFINE FIELD IF NOT EXISTS event_type ON match_timeline_event TYPE string;
DEFINE FIELD IF NOT EXISTS team_id ON match_timeline_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS killer_participant_id ON match_timeline_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS victim_participant_id ON match_timeline_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS monster_type ON match_timeline_event TYPE option<string>;
DEFINE FIELD IF NOT EXISTS monster_sub_type ON match_timeline_event TYPE option<string>;
DEFINE FIELD IF NOT EXISTS building_type ON match_timeline_event TYPE option<string>;
DEFINE FIELD IF NOT EXISTS is_first_blood ON match_timeline_event TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS multi_kill_length ON match_timeline_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS is_teamfight ON match_timeline_event TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS involved_participants ON match_timeline_event TYPE option<string>;
DEFINE INDEX IF NOT EXISTS timeline_match_id ON match_timeline_event FIELDS match_id;

-- Personal Learnings (Phase 14) — user-scoped, NOT team-scoped
DEFINE TABLE IF NOT EXISTS personal_learning SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON personal_learning TYPE record<user>;
DEFINE FIELD IF NOT EXISTS title ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS learning_type ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS champion ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS opponent ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS what_happened ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS what_i_learned ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS next_time ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS tags ON personal_learning TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS win_loss ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS match_riot_id ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS game_timestamp_ms ON personal_learning TYPE option<int>;
DEFINE FIELD IF NOT EXISTS event_name ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON personal_learning TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS personal_learning_user ON personal_learning FIELDS user;

-- Personal Goals (Phase 15) — one active goal per type per user (D-08).
DEFINE TABLE IF NOT EXISTS personal_goal SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON personal_goal TYPE record<user>;
DEFINE FIELD IF NOT EXISTS goal_type ON personal_goal TYPE string;
DEFINE FIELD IF NOT EXISTS target_value ON personal_goal TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON personal_goal TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON personal_goal TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS personal_goal_user_type ON personal_goal FIELDS user, goal_type UNIQUE;

-- Bug reports (Phase 19 D-02) — captured by floating widget,
-- exported to .planning/INBOX/bug-reports.md on server start.
DEFINE TABLE IF NOT EXISTS bug_report SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user ON bug_report TYPE record<user>;
DEFINE FIELD IF NOT EXISTS page_url ON bug_report TYPE string;
DEFINE FIELD IF NOT EXISTS element_label ON bug_report TYPE string;
DEFINE FIELD IF NOT EXISTS description ON bug_report TYPE string;
DEFINE FIELD IF NOT EXISTS category ON bug_report TYPE string
  ASSERT $value IN ['bug', 'wishlist'];
DEFINE FIELD IF NOT EXISTS viewport_w ON bug_report TYPE option<int>;
DEFINE FIELD IF NOT EXISTS viewport_h ON bug_report TYPE option<int>;
DEFINE FIELD IF NOT EXISTS created_at ON bug_report TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS status ON bug_report TYPE string DEFAULT 'open'
  ASSERT $value IN ['open', 'triaged', 'closed'];
DEFINE INDEX IF NOT EXISTS bug_report_status_created
  ON bug_report FIELDS status, created_at;

-- Data Dragon (M6): the last good copy of the champion list, so a cold
-- start without network still has champion data, and detected patch changes.
DEFINE TABLE IF NOT EXISTS ddragon_cache SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS version ON ddragon_cache TYPE string;
DEFINE FIELD IF NOT EXISTS champions_json ON ddragon_cache TYPE string;
DEFINE FIELD IF NOT EXISTS fetched_at ON ddragon_cache TYPE datetime DEFAULT time::now();
DEFINE TABLE IF NOT EXISTS patch_change SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS old_version ON patch_change TYPE option<string>;
DEFINE FIELD IF NOT EXISTS new_version ON patch_change TYPE string;
DEFINE FIELD IF NOT EXISTS detected_at ON patch_change TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS patch_change_detected_at ON patch_change FIELDS detected_at;

-- Sand Table (M6): storage only, no UI yet. A replay is a team's imported
-- copy of one match timeline (src/server/sand_table.rs); frames hold every
-- participant per minute, events the positioned moments. Boards are saved as
-- numbered versions of a scene document.
DEFINE TABLE IF NOT EXISTS replay SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON replay TYPE record<team>;
DEFINE FIELD IF NOT EXISTS match_id ON replay TYPE string;
DEFINE FIELD IF NOT EXISTS patch ON replay TYPE option<string>;
DEFINE FIELD IF NOT EXISTS imported_at ON replay TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS source ON replay TYPE string ASSERT $value IN ['riot_timeline'];
DEFINE FIELD IF NOT EXISTS frame_interval_ms ON replay TYPE int;
DEFINE FIELD IF NOT EXISTS participants ON replay TYPE array<object> DEFAULT [];
DEFINE FIELD IF NOT EXISTS participants[*].participant_id ON replay TYPE int;
DEFINE FIELD IF NOT EXISTS participants[*].puuid ON replay TYPE option<string>;
DEFINE FIELD IF NOT EXISTS participants[*].team_id ON replay TYPE int;
DEFINE INDEX IF NOT EXISTS replay_team_match ON replay FIELDS team, match_id UNIQUE;

DEFINE TABLE IF NOT EXISTS replay_frame SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS replay ON replay_frame TYPE record<replay>;
DEFINE FIELD IF NOT EXISTS t_ms ON replay_frame TYPE int;
DEFINE FIELD IF NOT EXISTS participants ON replay_frame TYPE array<object> DEFAULT [];
DEFINE FIELD IF NOT EXISTS participants[*].participant_id ON replay_frame TYPE int;
DEFINE FIELD IF NOT EXISTS participants[*].x ON replay_frame TYPE int;
DEFINE FIELD IF NOT EXISTS participants[*].y ON replay_frame TYPE int;
DEFINE FIELD IF NOT EXISTS participants[*].gold ON replay_frame TYPE int;
DEFINE FIELD IF NOT EXISTS participants[*].current_gold ON replay_frame TYPE int;
DEFINE FIELD IF NOT EXISTS participants[*].xp ON replay_frame TYPE int;
DEFINE FIELD IF NOT EXISTS participants[*].cs ON replay_frame TYPE int;
DEFINE FIELD IF NOT EXISTS participants[*].level ON replay_frame TYPE int;
DEFINE FIELD IF NOT EXISTS participants[*].alive ON replay_frame TYPE bool;
DEFINE INDEX IF NOT EXISTS replay_frame_replay ON replay_frame FIELDS replay, t_ms;

DEFINE TABLE IF NOT EXISTS replay_event SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS replay ON replay_event TYPE record<replay>;
DEFINE FIELD IF NOT EXISTS seq ON replay_event TYPE int;
DEFINE FIELD IF NOT EXISTS t_ms ON replay_event TYPE int;
DEFINE FIELD IF NOT EXISTS kind ON replay_event TYPE string;
DEFINE FIELD IF NOT EXISTS x ON replay_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS y ON replay_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS participants ON replay_event TYPE array<int> DEFAULT [];
DEFINE FIELD IF NOT EXISTS team_id ON replay_event TYPE option<int>;
DEFINE FIELD IF NOT EXISTS meta ON replay_event TYPE object FLEXIBLE DEFAULT {};
DEFINE INDEX IF NOT EXISTS replay_event_replay ON replay_event FIELDS replay, seq;

DEFINE TABLE IF NOT EXISTS board SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON board TYPE record<team>;
DEFINE FIELD IF NOT EXISTS name ON board TYPE string;
DEFINE FIELD IF NOT EXISTS created_by ON board TYPE record<user>;
DEFINE FIELD IF NOT EXISTS created_at ON board TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON board TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS board_team ON board FIELDS team;

DEFINE TABLE IF NOT EXISTS board_version SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS board ON board_version TYPE record<board>;
DEFINE FIELD IF NOT EXISTS version ON board_version TYPE int;
DEFINE FIELD IF NOT EXISTS scene ON board_version TYPE object FLEXIBLE;
DEFINE FIELD IF NOT EXISTS author ON board_version TYPE record<user>;
DEFINE FIELD IF NOT EXISTS created_at ON board_version TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS board_version_unique ON board_version FIELDS board, version UNIQUE;

-- Record links (M6). These fields held record IDs as strings before M6;
-- migration 5 converted existing values and tightened the types above.
-- action_item.assigned_to stays a string: it holds the assignee's username as
-- picked on the page, not a record ID.
DEFINE INDEX IF NOT EXISTS draft_series ON draft FIELDS series_id;
DEFINE INDEX IF NOT EXISTS game_plan_draft ON game_plan FIELDS draft;
DEFINE INDEX IF NOT EXISTS post_game_learning_plan ON post_game_learning FIELDS game_plan_id;
DEFINE INDEX IF NOT EXISTS checklist_instance_plan ON checklist_instance FIELDS game_plan_id;

-- Indexes on the scope fields that tenant-scoped reads filter on (M6).
-- Plain (non-unique) indexes are safe to add to an existing database here.
DEFINE INDEX IF NOT EXISTS team_member_team ON team_member FIELDS team;
DEFINE INDEX IF NOT EXISTS team_member_user ON team_member FIELDS user;
DEFINE INDEX IF NOT EXISTS join_request_team ON join_request FIELDS team;
DEFINE INDEX IF NOT EXISTS join_request_user ON join_request FIELDS user;
DEFINE INDEX IF NOT EXISTS match_team ON match FIELDS team_id;
DEFINE INDEX IF NOT EXISTS player_match_user ON player_match FIELDS user;
DEFINE INDEX IF NOT EXISTS player_match_match ON player_match FIELDS match;
DEFINE INDEX IF NOT EXISTS draft_team ON draft FIELDS team;
DEFINE INDEX IF NOT EXISTS draft_action_draft ON draft_action FIELDS draft;
DEFINE INDEX IF NOT EXISTS game_plan_team ON game_plan FIELDS team;
DEFINE INDEX IF NOT EXISTS post_game_learning_team ON post_game_learning FIELDS team;
DEFINE INDEX IF NOT EXISTS champion_pool_user ON champion_pool FIELDS user;
DEFINE INDEX IF NOT EXISTS champion_note_user ON champion_note FIELDS user;
DEFINE INDEX IF NOT EXISTS draft_tree_team ON draft_tree FIELDS team;
DEFINE INDEX IF NOT EXISTS draft_tree_node_tree ON draft_tree_node FIELDS tree;
DEFINE INDEX IF NOT EXISTS draft_tree_node_parent ON draft_tree_node FIELDS parent;
DEFINE INDEX IF NOT EXISTS tree_node_action_node ON tree_node_action FIELDS node;
DEFINE INDEX IF NOT EXISTS ban_priority_team ON ban_priority FIELDS team;
DEFINE INDEX IF NOT EXISTS opponent_team ON opponent FIELDS team;
DEFINE INDEX IF NOT EXISTS opponent_player_opponent ON opponent_player FIELDS opponent;
DEFINE INDEX IF NOT EXISTS series_team ON series FIELDS team;
DEFINE INDEX IF NOT EXISTS action_item_team ON action_item FIELDS team;
DEFINE INDEX IF NOT EXISTS team_note_team ON team_note FIELDS team;
DEFINE INDEX IF NOT EXISTS checklist_template_team ON checklist_template FIELDS team;
DEFINE INDEX IF NOT EXISTS checklist_instance_team ON checklist_instance FIELDS team;
DEFINE INDEX IF NOT EXISTS bug_report_user ON bug_report FIELDS user;
