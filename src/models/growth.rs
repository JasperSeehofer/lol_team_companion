//! Growth state for the immersive "rooms" UI.
//!
//! As a user accumulates work (learnings, champion pool, ranked snapshots, …)
//! the room environment fills out in **discrete tiers** — shelves of a library
//! fill, an armory racks up, banner lighting rises. This module owns the pure
//! mapping from raw accumulation counts to those tiers so the logic is shared
//! across SSR and WASM and is unit-testable without a database.

use serde::{Deserialize, Serialize};

/// A coarse growth tier shared by every room axis. Numeric value (0..=3) is the
/// canonical form CSS `data-*-tier` attributes carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    /// Nothing yet — the room starts empty/dormant.
    Seed,
    /// First signs of life.
    Sprout,
    /// A meaningful body of work.
    Established,
    /// A rich, fully-realized space.
    Flourishing,
}

impl Tier {
    /// 0..=3 — the value written to CSS `data-*-tier` attributes.
    pub fn level(self) -> u8 {
        match self {
            Tier::Seed => 0,
            Tier::Sprout => 1,
            Tier::Established => 2,
            Tier::Flourishing => 3,
        }
    }
}

/// Raw per-user accumulation counts. All six tables are user-scoped
/// (`user record<user>` in `schema.surql`); team-scoped tables like `draft`
/// and `post_game_learning` belong to the War Room, not the Solo Journey.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrowthCounts {
    pub personal_learning: u32,
    pub champion_pool: u32,
    pub champion_note: u32,
    pub personal_goal: u32,
    pub ranked_snapshot: u32,
    pub player_match: u32,
}

/// Derived, render-ready growth state. The room reads the `*_tier` fields to
/// pick which art overlay to show; `counts` is kept for surface labels
/// ("12 learnings recorded").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrowthState {
    pub counts: GrowthCounts,
    /// Library shelf fill — driven by learnings + champion notes (knowledge).
    pub library_tier: u8,
    /// Armory rack fill — driven by champion pool breadth.
    pub armory_tier: u8,
    /// Banner / lighting — driven by ranked snapshot history (the climb).
    pub banner_tier: u8,
}

/// Map a single count onto a tier given the lower bounds for tiers 1/2/3.
/// `count >= thresholds[2]` → Flourishing, etc. Pure and total.
fn tier_for(count: u32, thresholds: [u32; 3]) -> Tier {
    if count >= thresholds[2] {
        Tier::Flourishing
    } else if count >= thresholds[1] {
        Tier::Established
    } else if count >= thresholds[0] {
        Tier::Sprout
    } else {
        Tier::Seed
    }
}

impl GrowthState {
    /// Derive discrete tiers from raw counts. Thresholds are intentionally
    /// discrete (the design calls for tiered, not continuous, growth).
    pub fn from_counts(counts: GrowthCounts) -> Self {
        let library = tier_for(counts.personal_learning + counts.champion_note, [1, 8, 25]);
        let armory = tier_for(counts.champion_pool, [1, 6, 15]);
        let banner = tier_for(counts.ranked_snapshot, [1, 10, 40]);
        GrowthState {
            counts,
            library_tier: library.level(),
            armory_tier: armory.level(),
            banner_tier: banner.level(),
        }
    }
}

impl Default for GrowthState {
    /// An empty room (all Seed) — the safe fallback when a user has no data.
    fn default() -> Self {
        GrowthState::from_counts(GrowthCounts::default())
    }
}

/// Raw per-team accumulation counts driving the War Room growth atmosphere.
/// All six tables carry `team record<team>` in `schema.surql` (draft,
/// draft_tree, opponent, ban_priority, post_game_learning, team_note);
/// `player_match` is user-scoped and deliberately excluded (D-05). Library
/// (records/tomes/analysis) and Throne (roster/progression/crown) axis
/// derivations are deferred to Phases 28/29 — this module ships only the War
/// Room derivation as the proven template (D-03).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamGrowthCounts {
    pub draft: u32,
    pub draft_tree: u32,
    pub opponent: u32,
    pub ban_priority: u32,
    pub post_game_learning: u32,
    pub team_note: u32,
}

/// Derived War Room growth state. Only the War Room axes (tactics/intel/command)
/// are derived here (D-03); Library and Throne axes are deferred to Phases 28/29.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamGrowthState {
    pub counts: TeamGrowthCounts,
    /// Tactics — drafting depth (drafts + draft trees).
    pub tactics_tier: u8,
    /// Intel — opponents scouted.
    pub intel_tier: u8,
    /// Command — ban priorities + post-game learnings (team direction).
    pub command_tier: u8,
}

impl TeamGrowthState {
    /// Derive War Room tiers from raw team counts. The thresholds below are
    /// D-09 calibration *hypotheses* flagged for beta tuning — not blocking.
    pub fn from_counts(counts: TeamGrowthCounts) -> Self {
        // D-06: tactics from drafts + draft trees.
        let tactics = tier_for(counts.draft + counts.draft_tree, [1, 5, 20]);
        // D-07: intel from opponents scouted.
        let intel = tier_for(counts.opponent, [1, 3, 10]);
        // D-08: command from ban priorities + post-game learnings.
        let command = tier_for(counts.ban_priority + counts.post_game_learning, [1, 5, 15]);
        TeamGrowthState {
            counts,
            tactics_tier: tactics.level(),
            intel_tier: intel.level(),
            command_tier: command.level(),
        }
    }
}

impl Default for TeamGrowthState {
    /// An empty team (all Seed) — the safe fallback before any team activity.
    fn default() -> Self {
        TeamGrowthState::from_counts(TeamGrowthCounts::default())
    }
}

/// Raw per-team accumulation counts driving the Library room growth atmosphere.
/// Three axes: records (match history), tomes (written knowledge), analysis
/// (analytical depth). Field names differ per source table — see `get_library_growth_counts`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryGrowthCounts {
    /// match table, scoped by team_id (records axis).
    pub match_syncs: u32,
    /// post_game_learning table, scoped by team (tomes + analysis axes).
    pub post_game_learning: u32,
    /// team_note table, scoped by team (tomes axis).
    pub team_note: u32,
}

/// Derived Library room growth state. SC4 ("a richer team sees more atmospheric
/// detail") is driven by these tier values surfacing as `data-records-tier` /
/// `data-tomes-tier` / `data-analysis-tier` on the atmosphere root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryGrowthState {
    pub counts: LibraryGrowthCounts,
    /// The archive grows as matches accumulate.
    pub records_tier: u8,
    /// Written knowledge — reviews + team notes.
    pub tomes_tier: u8,
    /// Analytical depth — post-game reviews only.
    pub analysis_tier: u8,
}

impl LibraryGrowthState {
    // D-13: calibration hypotheses, tunable without touching the room UI
    pub fn from_counts(counts: LibraryGrowthCounts) -> Self {
        let records = tier_for(counts.match_syncs, [1, 10, 30]);
        let tomes = tier_for(counts.post_game_learning + counts.team_note, [1, 5, 15]);
        let analysis = tier_for(counts.post_game_learning, [1, 5, 15]);
        LibraryGrowthState {
            counts,
            records_tier: records.level(),
            tomes_tier: tomes.level(),
            analysis_tier: analysis.level(),
        }
    }
}

impl Default for LibraryGrowthState {
    /// An empty library (all Seed) — the safe fallback when a team has no data.
    fn default() -> Self {
        LibraryGrowthState::from_counts(LibraryGrowthCounts::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_counts_are_all_seed() {
        let g = GrowthState::default();
        assert_eq!(g.library_tier, 0);
        assert_eq!(g.armory_tier, 0);
        assert_eq!(g.banner_tier, 0);
    }

    #[test]
    fn tier_for_respects_lower_bounds() {
        let t = [1, 8, 25];
        assert_eq!(tier_for(0, t), Tier::Seed);
        assert_eq!(tier_for(1, t), Tier::Sprout); // exactly the tier-1 bound
        assert_eq!(tier_for(7, t), Tier::Sprout);
        assert_eq!(tier_for(8, t), Tier::Established); // exactly the tier-2 bound
        assert_eq!(tier_for(24, t), Tier::Established);
        assert_eq!(tier_for(25, t), Tier::Flourishing); // exactly the tier-3 bound
        assert_eq!(tier_for(1000, t), Tier::Flourishing);
    }

    #[test]
    fn library_combines_learnings_and_notes() {
        // 5 learnings + 4 notes = 9 → crosses the tier-2 bound (8).
        let g = GrowthState::from_counts(GrowthCounts {
            personal_learning: 5,
            champion_note: 4,
            ..Default::default()
        });
        assert_eq!(g.library_tier, 2);
    }

    #[test]
    fn armory_tracks_pool_size() {
        let g = GrowthState::from_counts(GrowthCounts {
            champion_pool: 6,
            ..Default::default()
        });
        assert_eq!(g.armory_tier, 2);
        let g = GrowthState::from_counts(GrowthCounts {
            champion_pool: 15,
            ..Default::default()
        });
        assert_eq!(g.armory_tier, 3);
    }

    #[test]
    fn banner_tracks_ranked_history() {
        let g = GrowthState::from_counts(GrowthCounts {
            ranked_snapshot: 40,
            ..Default::default()
        });
        assert_eq!(g.banner_tier, 3);
    }

    #[test]
    fn growth_state_round_trips_json() {
        let g = GrowthState::from_counts(GrowthCounts {
            personal_learning: 12,
            champion_pool: 7,
            ranked_snapshot: 30,
            ..Default::default()
        });
        let json = serde_json::to_string(&g).unwrap();
        let back: GrowthState = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }

    // --- Team-scoped (War Room) growth ---

    #[test]
    fn team_empty_counts_are_all_seed() {
        let g = TeamGrowthState::default();
        assert_eq!(g.tactics_tier, 0);
        assert_eq!(g.intel_tier, 0);
        assert_eq!(g.command_tier, 0);
    }

    #[test]
    fn team_tactics_combines_draft_and_tree() {
        // 3 drafts + 2 draft trees = 5 → crosses the tier-2 bound (5).
        let g = TeamGrowthState::from_counts(TeamGrowthCounts {
            draft: 3,
            draft_tree: 2,
            ..Default::default()
        });
        assert_eq!(g.tactics_tier, 2);
    }

    #[test]
    fn team_intel_tracks_opponents() {
        // 10 opponents → tier-3 bound (10).
        let g = TeamGrowthState::from_counts(TeamGrowthCounts {
            opponent: 10,
            ..Default::default()
        });
        assert_eq!(g.intel_tier, 3);
    }

    #[test]
    fn team_command_combines_bans_and_learnings() {
        // 3 ban priorities + 12 post-game learnings = 15 → tier-3 bound (15).
        let g = TeamGrowthState::from_counts(TeamGrowthCounts {
            ban_priority: 3,
            post_game_learning: 12,
            ..Default::default()
        });
        assert_eq!(g.command_tier, 3);
    }

    #[test]
    fn team_growth_state_round_trips_json() {
        let g = TeamGrowthState::from_counts(TeamGrowthCounts {
            draft: 4,
            opponent: 5,
            post_game_learning: 8,
            ..Default::default()
        });
        let json = serde_json::to_string(&g).unwrap();
        let back: TeamGrowthState = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }

    // --- Library (team-scoped) growth ---

    #[test]
    fn library_empty_counts_are_all_seed() {
        let g = LibraryGrowthState::default();
        assert_eq!(g.records_tier, 0);
        assert_eq!(g.tomes_tier, 0);
        assert_eq!(g.analysis_tier, 0);
    }

    #[test]
    fn library_records_tracks_match_syncs() {
        // 10 syncs → exactly the tier-2 lower bound (10).
        let g = LibraryGrowthState::from_counts(LibraryGrowthCounts {
            match_syncs: 10,
            ..Default::default()
        });
        assert_eq!(g.records_tier, 2);
    }

    #[test]
    fn library_tomes_combines_reviews_and_notes() {
        // 3 reviews + 2 notes = 5 → tier-2 lower bound (5).
        let g = LibraryGrowthState::from_counts(LibraryGrowthCounts {
            post_game_learning: 3,
            team_note: 2,
            ..Default::default()
        });
        assert_eq!(g.tomes_tier, 2);
    }

    #[test]
    fn library_analysis_tracks_post_game_reviews() {
        // 15 reviews → tier-3 lower bound (15).
        let g = LibraryGrowthState::from_counts(LibraryGrowthCounts {
            post_game_learning: 15,
            ..Default::default()
        });
        assert_eq!(g.analysis_tier, 3);
    }
}
