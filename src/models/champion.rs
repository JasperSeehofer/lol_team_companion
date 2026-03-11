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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn champion_pool_entry_tiers_round_trip() {
        for tier in ["s", "a", "b", "c", "situational"] {
            let entry = ChampionPoolEntry {
                id: None,
                user_id: "user:u1".into(),
                champion: "Jinx".into(),
                role: "bot".into(),
                tier: tier.to_string(),
                notes: None,
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
            tier: "s".into(),
            notes: Some("Best into poke comps".into()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: ChampionPoolEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, back);
    }
}
