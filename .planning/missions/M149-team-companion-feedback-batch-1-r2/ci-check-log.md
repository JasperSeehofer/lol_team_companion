# CI check log, run 43295337294 (9f0cb34b3b888e411192080a0c39b966b533736e)

Last 300 lines of the Check, lint, unit tests job.

```
test server::graph::traits::tests::sync_plan_never_touches_maintainer_rows ... ok
test server::graph::rules::tests::pair_relations_cover_both_orders_from_the_shipped_defaults ... ok
test server::images::tests::champion_ids_are_alphanumeric ... ok
test server::images::tests::etags_change_with_the_patch ... ok
test server::images::tests::item_ids_are_short_digit_strings ... ok
test server::graph::traits::tests::table_resolves_ids_and_display_names ... ok
test server::images::tests::unknown_keys_are_not_found_without_a_fetch ... ok
test server::images::tests::upstream_urls_are_data_dragon_only ... ok
test server::inflight::tests::a_request_stays_registered_until_its_body_is_sent ... ok
test server::inflight::tests::share_tokens_are_redacted_from_logged_paths ... ok
test server::inflight::tests::slow_requests_are_reported_once ... ok
test server::inflight::tests::the_ci_stall_check_matches_the_watchdog_line ... ok
test server::inflight::tests::the_invariant_lock_holder_is_named ... ok
test server::password::tests::codes_are_long_random_and_readable ... ok
test server::password::tests::hashes_compare_exactly ... ok
test server::password::tests::new_passwords_follow_the_registration_rule ... ok
test server::password::tests::the_hash_ignores_case_spaces_and_dashes ... ok
test server::password::tests::the_tombstone_is_excluded_by_its_id ... ok
test server::rate_limit::tests::bucket_allows_burst_then_limits_then_refills ... ok
test server::rate_limit::tests::check_does_not_consume ... ok
test server::rate_limit::tests::cooldown_blocks_until_period_passes ... ok
test server::rate_limit::tests::message_mentions_wait ... ok
test server::rate_limit::tests::multiplier_scales_capacity ... ok
test server::rate_limit::tests::purge_drops_only_entries_that_no_longer_limit ... ok
test server::rate_limit::tests::riot_budget_has_a_short_and_a_long_window ... ok
test server::rate_limit::tests::riot_budget_headroom_is_capped_at_the_burst ... ok
test server::images::tests::simultaneous_misses_share_one_download ... ok
test server::riot::tests::account_region_mapping ... ok
test server::riot::tests::an_unconfigured_client_reports_not_configured ... ok
test server::riot::tests::conversion_keeps_positions_where_riot_has_them ... ok
test server::riot::tests::conversion_keeps_purchases_with_buyer_item_and_team ... ok
test server::riot::tests::conversion_skips_other_events_and_marks_first_blood ... ok
test server::riot::tests::converted_purchases_give_inferred_recalls ... ok
test server::riot::tests::match_ids_name_their_platform ... ok
test server::riot::tests::only_missing_games_are_permanent_failures ... ok
test server::riot::tests::platform_prefers_the_first_region_that_is_set ... ok
test server::riot::tests::platform_route_from_str_all_regions ... ok
test server::riot::tests::test_classify_event_types ... ok
test server::riot::tests::test_compute_performance_stats ... ok
test server::sand_table::tests::alive_is_estimated_from_kills ... ok
test server::sand_table::tests::events_are_converted_in_time_order ... ok
test server::sand_table::tests::frames_carry_position_economy_and_level ... ok
test server::sand_table::tests::malformed_documents_are_rejected ... ok
test server::riot::tests::a_spent_budget_fails_fast_with_busy ... ok
test server::sand_table::tests::reads_match_and_participants ... ok
test server::sand_table::tests::respawn_table_is_clamped ... ok
test server::sand_table::tests::unknown_events_are_counted_not_fatal ... ok
test server::sand_table::tests::wrong_types_are_skipped_not_fatal ... ok
test server::ui_icons::tests::a_good_upstream_body_is_fetched_once_and_cached ... ok
test server::ui_icons::tests::allow_list_is_complete_and_unique ... ok
test server::ui_icons::tests::committed_copies_are_not_in_the_static_site_root ... ok
test server::ui_icons::tests::committed_svgs_are_sanitised ... ok
test server::ui_icons::tests::etag_answers_304 ... ok
test server::ui_icons::tests::failed_upstream_serves_the_committed_copy ... ok
test server::ui_icons::tests::unknown_and_traversal_shaped_names_are_refused ... ok
test server::ui_icons::tests::unknown_pairs_are_404_without_a_fetch ... ok
test server::ui_icons::tests::upstream_urls_are_pinned_constants ... ok
test server::graph::fit::tests::ties_break_by_pool_comfort_then_playstyle_then_name ... ok
test server::sand_table::tests::oversized_documents_are_rejected ... ok
test server::graph::fit::tests::fuzzed_drafts_stay_bounded_explained_deterministic_and_scaled ... ok

test result: ok. 488 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.12s

=== DB integration tests ===
   Compiling lol_team_companion v0.1.0 (/home/runner/work/lol_team_companion/lol_team_companion)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3m 05s
warning: the following packages contain code that will be rejected by a future version of Rust: proc-macro-error2 v2.0.1
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
     Running tests/integration/main.rs (target/debug/build/lol_team_companion/104ce02611d5d47e/out/integration-104ce02611d5d47e)

running 211 tests
test account::bug_reports_survive_deletion_anonymised ... ok
test account::deleting_ends_the_users_sessions_only ... ok
test account::a_hidden_card_cannot_be_shared_and_old_links_are_revoked ... ok
test account::a_leader_hands_over_first_or_deletes_a_team_of_one ... ok
test account::export_holds_every_section_and_nothing_of_others ... ok
test account::export_carries_the_reporters_own_screenshots ... ok
test account::deletion_leaves_no_row_pointing_at_the_user ... ok
test account::the_seed_covers_every_user_link ... ok
test community::a_wording_edit_withdraws_the_approval ... ok
test community::admin_list_filters_by_status_and_names_the_reporter ... ok
test community::answers_count_against_the_wording_that_was_shown ... ok
test community::concurrent_triggers_leave_one_prompt_and_patches_count_once_each ... ok
test community::concurrent_votes_leave_one_row ... ok
test community::delivery_answer_and_weekly_cap ... ok
test community::dismissed_prompts_never_return_and_opt_out_expires_pending ... ok
test community::feedback_export_admin_rule_comes_from_the_user_row ... ok
test community::feedback_export_writes_both_files_and_survives_hostile_text ... ok
test community::hidden_and_closed_proposals_refuse_votes ... ok
test community::imported_agent_proposals_are_unpublished_drafts ... ok
test community::ledger_drafts_stay_private_until_published ... ok
test community::only_wishlist_reports_become_proposals ... ok
test community::proposal_rules ... ok
test community::one_vote_per_user_and_withdrawal_is_own_only ... ok
test community::prompts_are_private_to_their_user ... ok
test community::proposals_with_votes_cannot_be_deleted ... ok
test community::reporters_see_only_their_own_reports ... ok
test community::site_texts_need_approval_of_the_exact_body ... ok
test community::the_board_shows_only_admin_written_text_of_public_reports ... ok
test community::the_admin_overview_counts ... ok
test community::triage_rules_are_enforced ... ok
test community::stale_pending_prompts_expire_and_paused_surveys_stay_silent ... ok
test community::feedback_export_debounces_and_writes_the_final_state ... ok
test community::the_schema_rejects_bad_ledger_values ... ok
test data_dragon::snapshots_persist_and_patch_changes_are_recorded ... ok
test community::unapproved_surveys_never_reach_users ... ok
test datetimes::champion_trend_window_filters_by_game_end ... ok
test datetimes::lists_with_timestamps_return_their_rows ... ok
test db_champion_pool::test_add_champion_duplicate_is_idempotent ... ok
test datetimes::lp_history_window_filters_by_time ... ok
test db_champion_pool::test_add_champion_to_pool ... ok
test db_champion_pool::test_get_pool_returns_all_entries ... ok
test db_champion_pool::test_remove_champion_from_pool ... ok
test db_champion_pool::test_update_champion_tier ... ok
test db_champion_trends::get_champion_trends_aggregates_correctly ... ok
test db_champion_trends::get_champion_trends_empty_returns_empty_vec ... ok
test db_champion_trends::get_champion_trends_cs_per_min_uses_seconds ... ok
test db_champion_trends::get_champion_trends_excludes_aram ... ok
test db_champion_trends::get_champion_trends_includes_solo_and_flex ... ok
test db_drafts::guessed_roles_round_trip ... ok
test db_drafts::test_list_drafts_aggregates_actions_correctly ... ok
test db_drafts::test_delete_draft ... ok
test db_drafts::test_save_draft_with_actions ... ok
test db_game_plan_pipeline::dashboard_counts_follow_record_links ... ok
test db_drafts::test_update_draft_replaces_actions ... ok
test db_game_plan_pipeline::test_get_draft_for_prefill_found ... ok
test db_game_plan_pipeline::test_get_draft_for_prefill_not_found ... ok
test db_game_plan_pipeline::test_get_game_plans_for_draft_found ... ok
test db_game_plan_pipeline::test_get_game_plans_for_draft_empty ... ok
test db_goal_progress::goal_progress_caps_at_twenty_games ... ok
test db_goal_progress::goal_progress_queue_isolation_solo_only ... ok
test db_goal_progress::goal_progress_three_games_below_threshold ... ok
test db_goal_progress::goal_progress_seven_games_aggregates ... ok
test db_goal_progress::goal_progress_zero_games_insufficient ... ok
test db_lp_history::get_lp_history_includes_rank_score ... ok
test db_lp_history::get_lp_history_empty_returns_empty_vec ... ok
test db_lp_history::get_lp_history_sorted_ascending ... ok
test db_opponents::test_create_opponent_with_empty_summoner_names ... ok
test db_opponents::test_update_opponent_player_intel ... ok
test db_teams::new_members_are_unplaced_and_the_leader_places_them ... ok
test db_opponents::test_create_opponent_with_players ... ok
test db_teams::test_assign_to_slot_bumps_existing_starter ... ok
test db_teams::test_create_team_and_get_it ... ok
test db_teams::test_join_request_duplicate_fails ... ok
test db_teams::test_join_request_flow_accept ... ok
test db_teams::test_join_request_flow_decline ... ok
test db_teams::test_join_team_already_member_fails ... ok
test db_teams::test_remove_from_slot ... ok
test db_teams::test_join_team_success ... ok
test db_tree::test_add_child_node ... ok
test db_teams::the_join_list_counts_members_per_team ... ok
test db_tree::test_create_tree_creates_root_node ... ok
test db_tree::test_delete_node_cascades_to_children ... ok
test db_tree::test_tree_sort_order_is_maintained ... ok
test db_tree::test_get_tree_nodes_reconstructs_hierarchy ... ok
test db_users::test_create_user_duplicate_email_fails ... ok
test db_users::sign_up_without_the_terms_box_leaves_the_invite_unused ... ok
test db_users::test_create_user_success ... ok
test db_users::test_get_nonexistent_user_returns_none ... ok
test db_users::the_promote_command_promotes_lists_and_revokes ... ok
test graph::editing_an_unknown_champion_is_not_found ... ok
test graph::maintainer_edits_survive_sync_and_can_be_reset ... ok
test graph::schema_rejects_values_outside_the_vocabulary ... ok
test graph::team_games_group_by_match_for_pair_evidence ... ok
test graph::sync_writes_a_valid_row_for_every_default ... ok
test guidance::nudges_read_the_users_team_and_writes ... ok
test guidance::the_switch_and_the_resets_touch_only_the_users_own_rows ... ok
test guidance::hints_read_the_users_own_actions ... ok
test guidance::rows_belong_to_one_user_and_writes_are_idempotent ... ok
test guidance::the_welcome_shows_once_and_the_checklist_follows_the_users_team ... ok
test match_day::a_new_opponent_or_a_deleted_dossier_resets_the_brief ... ok
test guidance::visits_count_for_the_returning_rule ... ok
test match_day::deleting_a_draft_marks_its_plans_and_reviews ... ok
test match_day::player_cards_show_members_only ... ok
test match_day::prep_facts_follow_the_artefacts ... ok
test match_day::team_match_rows_carry_their_match_id ... ok
test match_day::share_links_open_one_artefact_and_stay_in_their_team ... ok
test match_day::the_next_match_is_team_scoped_and_validated ... ok
test match_thread::a_carried_match_resolves_for_the_team_only ... ok
test match_thread::a_match_id_from_the_address_falls_back_to_the_next_match ... ok
test match_thread::another_teams_match_or_series_is_not_found ... ok
test match_thread::deleting_a_match_clears_its_stamps_and_keeps_the_work ... ok
test match_thread::prep_facts_count_only_work_stamped_to_the_match ... ok
test match_thread::deleting_a_series_unlinks_reviews_and_matches ... ok
test match_thread::review_candidates_are_team_scoped ... ok
test match_thread::stamps_take_the_teams_own_match_series_and_game ... ok
test match_thread::update_draft_rechecks_a_carried_opponent ... ok
test match_thread::upcoming_matches_are_ordered_bounded_and_team_scoped ... ok
test migrations::a_newer_database_is_refused_before_the_baseline_is_applied ... ok
test match_thread::use_for_match_is_team_scoped ... ok
test migrations::a_database_from_a_newer_build_is_refused ... ok
test migrations::bug_report_statuses_are_mapped_to_v2 ... ok
test migrations::community_seed_content_is_inserted_once_and_unapproved ... ok
test migrations::a_failed_surql_migration_rolls_back_and_is_not_recorded ... ok
test migrations::link_keys_parse_only_matching_ids ... ok
test migrations::extra_memberships_are_resolved_before_the_unique_index ... ok
test migrations::fresh_database_records_every_migration_once ... ok
test migrations::legacy_rows_are_backfilled ... ok
test migrations::migrations_must_be_in_order ... ok
test migrations::payment_fees_become_a_cost_line_and_old_labels_need_approval_again ... ok
test migrations::string_ids_become_record_links ... ok
test migrations::roster_places_roles_and_dead_links_are_migrated ... ok
test migrations::scope_fields_are_indexed ... ok
test migrations::the_user_mode_field_is_dropped ... ok
test migrations::the_user_theme_field_is_dropped ... ok
test password::a_code_for_one_account_cannot_reset_another ... ok
test password::a_new_code_replaces_the_open_one ... ok
test password::a_reset_code_works_once ... ok
test password::a_wrong_or_expired_code_is_refused ... ok
test password::changing_the_password_ends_the_other_sessions ... ok
test password::concurrent_redeems_of_one_code_succeed_once ... ok
test password::deleting_an_account_removes_its_reset_codes ... ok
test password::only_an_admin_can_mint_a_code ... ok
test retention::a_freshly_cached_game_is_kept ... ok
test retention::old_match_caches_are_deleted_in_batches ... ok
test retention::old_ranked_snapshots_are_purged ... ok
test retention::syncs_and_retention_do_not_race ... ok
test sand_table::boards_keep_numbered_versions ... ok
test sand_table::imported_timeline_round_trips ... ok
test security::action_item_due_dates_are_checked ... ok
test security::action_item_is_isolated_between_teams ... ok
test security::action_item_source_review_must_belong_to_the_team ... ok
test security::admin_flag_round_trip ... ok
test security::champion_in_a_synced_game_is_private_to_its_owner ... ok
test security::concurrent_accepts_add_one_membership_mem ... ok
test security::champion_note_is_private_to_its_owner ... ok
test security::concurrent_board_saves_get_distinct_versions_mem ... ok
test security::concurrent_cross_team_accepts_keep_one_team_mem ... ok
test security::concurrent_joins_keep_one_team_mem ... ok
test security::concurrent_accepts_add_one_membership_surrealkv ... ok
test security::concurrent_board_saves_get_distinct_versions_surrealkv ... ok
test security::concurrent_cross_team_accepts_keep_one_team_surrealkv ... ok
test security::concurrent_registrations_cannot_overuse_an_invite_mem ... ok
test security::concurrent_signups_take_a_name_once_mem ... ok
test security::concurrent_joins_keep_one_team_surrealkv ... ok
test security::create_tree_node_treats_ids_as_data_not_query_text ... ok
test security::draft_is_isolated_between_teams ... ok
test security::concurrent_registrations_cannot_overuse_an_invite_surrealkv ... ok
test security::concurrent_signups_take_a_name_once_surrealkv ... ok
test security::duplicate_email_and_username_are_rejected_case_insensitively ... ok
test security::ensure_invite_is_idempotent_and_codes_are_random ... ok
test security::expired_sessions_are_not_loaded_and_are_purged ... ok
test security::invite_gate_can_only_be_disabled_in_dev ... ok
test security::game_plan_and_checklist_are_isolated_between_teams ... ok
test security::guidance_is_isolated_between_users_and_teams ... ok
test security::hand_over_and_export_stay_inside_the_tenant ... ok
test security::join_request_can_only_be_answered_by_its_team_and_once ... ok
test security::match_stamps_are_isolated_between_teams ... ok
test security::membership_and_leadership_checks ... ok
test security::opponent_and_players_are_isolated_between_teams ... ok
test security::personal_learning_is_private_to_its_owner ... ok
test security::review_is_isolated_between_teams ... ok
test security::series_is_isolated_between_teams ... ok
test security::sand_table_replays_and_boards_are_isolated_between_teams ... ok
test security::store_matches_binds_values ... ok
test security::team_codes_redeem_into_their_own_team_and_visits_stay_per_user ... ok
test security::team_games_and_pools_are_isolated_between_teams ... ok
test security::team_invites_stay_inside_their_team ... ok
test security::team_note_edits_and_deletes_can_be_undone ... ok
test security::team_note_is_isolated_between_teams ... ok
test security::team_relation_is_isolated_between_teams ... ok
test security::tree_and_nodes_are_isolated_between_teams ... ok
test security::unknown_expired_and_disabled_invites_are_rejected ... ok
test security::users_belong_to_at_most_one_team ... ok
test security::valid_invite_is_consumed_until_used_up ... ok
test team_invites::a_team_code_cannot_be_used_by_someone_already_in_a_team ... ok
test team_invites::a_team_code_registers_the_account_into_the_team ... ok
test team_invites::deleting_the_team_removes_its_codes ... ok
test team_invites::joining_a_team_deleted_after_the_use_was_taken_is_refused ... ok
test team_invites::the_leader_and_coaches_manage_invites_members_do_not ... ok
test timeline_reimport::a_failed_replace_keeps_the_old_timeline ... ok
test timeline_reimport::new_fields_are_stored_and_read_back ... ok
test timeline_reimport::the_backfill_selects_only_older_games ... FAILED
test timeline_reimport::the_replace_is_whole_and_idempotent ... ok
test timeline_reimport::the_retention_purge_removes_reimported_rows ... ok
test transactions::deleting_a_series_unlinks_its_drafts ... ok
test transactions::match_detail_cache_is_written_whole ... ok
test transactions::opponent_and_players_are_created_together ... ok
test transactions::review_and_its_action_items_are_written_together ... ok
test transactions::save_draft_writes_draft_and_actions_together ... ok
test transactions::tree_with_root_is_created_and_deleted_as_a_unit ... ok
test transactions::update_draft_keeps_old_actions_when_the_rewrite_fails ... ok

failures:

---- timeline_reimport::the_backfill_selects_only_older_games stdout ----

thread 'timeline_reimport::the_backfill_selects_only_older_games' (15205) panicked at tests/integration/timeline_reimport.rs:129:5:
assertion `left == right` failed
  left: []
 right: ["EUW1_OLD_A"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    timeline_reimport::the_backfill_selects_only_older_games

test result: FAILED. 210 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 46.05s

error: test failed, to rerun pass `--test integration`
```
