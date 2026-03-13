use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Champion {
    pub id: String,
    pub name: String,
    pub title: String,
    pub tags: Vec<String>,
    pub image_full: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct ChampionPoolEntry {
    pub id: Option<String>,
    pub user_id: String,
    pub champion: String,
    pub role: String,
    pub tier: String,
    pub notes: Option<String>,
    pub comfort_level: Option<u8>,
    pub meta_tag: Option<String>,
}

/// Structured note for a champion (matchup, power spike, combo, lesson, synergy, positioning)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChampionNote {
    pub id: Option<String>,
    pub user_id: String,
    pub champion: String,
    pub role: String,
    pub note_type: String,
    pub title: String,
    pub content: String,
    pub difficulty: Option<u8>,
    pub created_at: Option<String>,
}

/// Valid note types for champion notes
pub const NOTE_TYPES: &[&str] = &[
    "matchup",
    "power_spike",
    "combo",
    "lesson",
    "synergy",
    "positioning",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChampionStatSummary {
    pub champion: String,
    pub games: i32,
    pub wins: i32,
    pub avg_kda: f64,
    pub avg_cs_per_min: f64,
}

pub fn note_type_label(note_type: &str) -> &'static str {
    match note_type {
        "matchup" => "Matchup",
        "power_spike" => "Power Spike",
        "combo" => "Combo",
        "lesson" => "Lesson Learned",
        "synergy" => "Synergy",
        "positioning" => "Positioning",
        _ => "Note",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn champion_pool_entry_tiers_round_trip() {
        for tier in ["comfort", "match_ready", "scrim_ready", "practicing", "to_practice"] {
            let entry = ChampionPoolEntry {
                id: None,
                user_id: "user:u1".into(),
                champion: "Jinx".into(),
                role: "bot".into(),
                tier: tier.to_string(),
                notes: None,
                comfort_level: Some(3),
                meta_tag: Some("strong".into()),
            };
            let json = serde_json::to_string(&entry).unwrap();
            let back: ChampionPoolEntry = serde_json::from_str(&json).unwrap();
            assert_eq!(entry, back, "tier {tier} failed round-trip");
        }
    }

    #[test]
    fn champion_pool_entry_with_notes_round_trips() {
        let entry = ChampionPoolEntry {
            id: Some("champion_pool:1".into()),
            user_id: "user:u1".into(),
            champion: "Azir".into(),
            role: "mid".into(),
            tier: "comfort".into(),
            notes: Some("Best into poke comps".into()),
            comfort_level: Some(5),
            meta_tag: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: ChampionPoolEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, back);
    }

    #[test]
    fn champion_note_round_trips() {
        let note = ChampionNote {
            id: Some("champion_note:1".into()),
            user_id: "user:u1".into(),
            champion: "Azir".into(),
            role: "mid".into(),
            note_type: "matchup".into(),
            title: "vs Syndra".into(),
            content: "Dodge Q with E shuffle".into(),
            difficulty: Some(4),
            created_at: Some("2026-03-13T00:00:00Z".into()),
        };
        let json = serde_json::to_string(&note).unwrap();
        let back: ChampionNote = serde_json::from_str(&json).unwrap();
        assert_eq!(note, back);
    }

    #[test]
    fn champion_stat_summary_round_trips_json() {
        let s = ChampionStatSummary {
            champion: "Jinx".into(),
            games: 12,
            wins: 8,
            avg_kda: 3.2,
            avg_cs_per_min: 8.1,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ChampionStatSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn champion_note_types_valid() {
        assert_eq!(NOTE_TYPES.len(), 6);
        assert_eq!(note_type_label("matchup"), "Matchup");
        assert_eq!(note_type_label("power_spike"), "Power Spike");
        assert_eq!(note_type_label("unknown"), "Note");
    }
}
