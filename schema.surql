-- Users
DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS username ON user TYPE string;
DEFINE FIELD IF NOT EXISTS email ON user TYPE string;
DEFINE FIELD IF NOT EXISTS password_hash ON user TYPE string;
DEFINE FIELD IF NOT EXISTS riot_puuid ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS riot_summoner_name ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON user TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS email_unique ON user FIELDS email UNIQUE;

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
DEFINE FIELD IF NOT EXISTS series_id ON draft TYPE option<string>;
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
DEFINE FIELD OVERWRITE draft ON game_plan TYPE option<string>;
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
DEFINE FIELD IF NOT EXISTS game_plan_id ON post_game_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS draft_id ON post_game_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS what_went_well ON post_game_learning TYPE array<string>;
DEFINE FIELD IF NOT EXISTS improvements ON post_game_learning TYPE array<string>;
DEFINE FIELD IF NOT EXISTS action_items ON post_game_learning TYPE array<string>;
DEFINE FIELD IF NOT EXISTS open_notes ON post_game_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_by ON post_game_learning TYPE record<user>;
DEFINE FIELD IF NOT EXISTS created_at ON post_game_learning TYPE datetime DEFAULT time::now();

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

-- Series (best-of sets)
DEFINE TABLE IF NOT EXISTS series SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON series TYPE string;
DEFINE FIELD IF NOT EXISTS team ON series TYPE record<team>;
DEFINE FIELD IF NOT EXISTS opponent_id ON series TYPE option<string>;
DEFINE FIELD IF NOT EXISTS opponent_name ON series TYPE option<string>;
DEFINE FIELD IF NOT EXISTS format ON series TYPE string DEFAULT 'bo1';
DEFINE FIELD IF NOT EXISTS is_fearless ON series TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS notes ON series TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_by ON series TYPE record<user>;
DEFINE FIELD IF NOT EXISTS created_at ON series TYPE datetime DEFAULT time::now();

-- Action items (from post-game reviews)
DEFINE TABLE IF NOT EXISTS action_item SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS team ON action_item TYPE record<team>;
DEFINE FIELD IF NOT EXISTS source_review ON action_item TYPE option<string>;
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
DEFINE FIELD IF NOT EXISTS game_plan_id ON checklist_instance TYPE option<string>;
DEFINE FIELD IF NOT EXISTS template_id ON checklist_instance TYPE option<string>;
DEFINE FIELD IF NOT EXISTS items ON checklist_instance TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS checked ON checklist_instance TYPE array<bool> DEFAULT [];
DEFINE FIELD IF NOT EXISTS created_at ON checklist_instance TYPE datetime DEFAULT time::now();

-- Sessions (for tower-sessions-surrealdb-store)
DEFINE TABLE IF NOT EXISTS sessions SCHEMALESS;
