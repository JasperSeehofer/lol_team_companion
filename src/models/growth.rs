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
}
