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

// --- Throne (blended team + personal) growth ---

/// Raw accumulation counts driving the Throne room growth atmosphere (team-scoped axes only).
/// Two axes: `team_members` (roster) and `team_wins` (crown). The third axis — `progression`
/// — is user-scoped (viewing user's ranked games) and supplied separately via `from_parts`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThroneGrowthCounts {
    /// team_member rows for the team (roster axis).
    pub team_members: u32,
    /// post_game_learning rows where win_loss = "win" (crown axis).
    pub team_wins: u32,
}

/// Derived Throne room growth state. SC1 ("the room's environment reflects roster size,
/// LP milestones, and combined team record") is driven by these tier values surfacing as
/// `data-roster-tier`/`data-progression-tier`/`data-crown-tier` on the atmosphere root.
///
/// KEY DIVERGENCE from TeamGrowthState/LibraryGrowthState: ThroneGrowthState has NO
/// `counts` field. The blended constructor takes both team counts AND the viewing user's
/// optional RankedInfo — it cannot be derived from a single `*Counts` struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThroneGrowthState {
    pub roster_tier: u8,
    pub progression_tier: u8,
    pub crown_tier: u8,
}

impl ThroneGrowthState {
    // D-05/D-07: progression derives from the viewing user's ranked games independently
    // of team membership — calibration thresholds [1,20,60], tunable without touching
    // the room UI.
    /// Blended derivation: team counts + the viewing user's optional ranked snapshot.
    /// `progression` derives independently so it advances with NO team (D-05/D-07).
    pub fn from_parts(
        team: ThroneGrowthCounts,
        ranked: Option<crate::models::user::RankedInfo>,
    ) -> Self {
        let roster = tier_for(team.team_members, [1, 3, 5]);
        let crown = tier_for(team.team_wins, [1, 5, 15]);
        let games = ranked
            .as_ref()
            .map(|r| (r.wins + r.losses).max(0) as u32)
            .unwrap_or(0);
        let progression = tier_for(games, [1, 20, 60]);
        ThroneGrowthState {
            roster_tier: roster.level(),
            progression_tier: progression.level(),
            crown_tier: crown.level(),
        }
    }
}

impl Default for ThroneGrowthState {
    /// All-zero tiers — the safe fallback for a teamless user with no ranked data.
    fn default() -> Self {
        Self::from_parts(ThroneGrowthCounts::default(), None)
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

    // --- Throne (blended team + personal) growth ---

    #[test]
    fn throne_empty_all_seed() {
        let g = ThroneGrowthState::default();
        assert_eq!(g.roster_tier, 0);
        assert_eq!(g.progression_tier, 0);
        assert_eq!(g.crown_tier, 0);
    }

    #[test]
    fn throne_roster_tracks_members() {
        // 5 members → tier-3 lower bound [1, 3, 5].
        let g = ThroneGrowthState::from_parts(
            ThroneGrowthCounts { team_members: 5, team_wins: 0 },
            None,
        );
        assert_eq!(g.roster_tier, 3);
    }

    #[test]
    fn throne_crown_tracks_wins() {
        // 15 wins → tier-3 lower bound [1, 5, 15].
        let g = ThroneGrowthState::from_parts(
            ThroneGrowthCounts { team_members: 0, team_wins: 15 },
            None,
        );
        assert_eq!(g.crown_tier, 3);
    }

    #[test]
    fn throne_progression_grows_without_team() {
        // D-05/D-07: 40 wins + 25 losses = 65 games ≥ 60 → progression_tier == 3.
        // No team → roster_tier == 0 AND crown_tier == 0.
        use crate::models::user::RankedInfo;
        let ranked = RankedInfo {
            queue_type: "RANKED_SOLO_5x5".to_string(),
            tier: "GOLD".to_string(),
            division: "II".to_string(),
            lp: 50,
            wins: 40,
            losses: 25,
        };
        let g = ThroneGrowthState::from_parts(ThroneGrowthCounts::default(), Some(ranked));
        assert_eq!(g.progression_tier, 3, "progression must advance with no team");
        assert_eq!(g.roster_tier, 0, "roster must remain 0 with no team");
        assert_eq!(g.crown_tier, 0, "crown must remain 0 with no team");
    }

    #[test]
    fn throne_state_round_trips_json() {
        let g = ThroneGrowthState {
            roster_tier: 2,
            progression_tier: 3,
            crown_tier: 1,
        };
        let json = serde_json::to_string(&g).unwrap();
        let back: ThroneGrowthState = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }
}
