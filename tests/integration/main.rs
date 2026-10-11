//! DB integration tests, compiled as a single test binary so the crate links
//! once (`cargo test --features ssr`). Each module covers one area and uses an
//! in-memory SurrealDB from `common::test_db()`.
#![cfg(feature = "ssr")]

mod common;

mod db_champion_pool;
mod db_champion_trends;
mod db_drafts;
mod db_game_plan_pipeline;
mod db_goal_progress;
mod db_lp_history;
mod db_opponents;
mod db_teams;
mod db_tree;
mod db_users;
mod security;
