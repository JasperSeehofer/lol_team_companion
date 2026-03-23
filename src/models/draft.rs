use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct Draft {
    pub id: Option<String>,
    pub name: String,
    pub team_id: String,
    pub created_by: String,
    pub opponent: Option<String>,
    pub notes: Option<String>,
    pub rating: Option<String>,
    /// "blue" or "red" — which side is our team
    pub our_side: String,
    pub actions: Vec<DraftAction>,
    pub comments: Vec<String>,
    /// Composition tags like "teamfight", "split-push", etc.
    pub tags: Vec<String>,
    /// Win condition notes for this draft
    pub win_conditions: Option<String>,
    /// "Watch out for" notes
    pub watch_out: Option<String>,
    /// Series this draft belongs to (if any)
    pub series_id: Option<String>,
    /// Game number within the series
    pub game_number: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct DraftAction {
    pub id: Option<String>,
    pub draft_id: String,
    /// ban1 / pick1 / ban2 / pick2
    pub phase: String,
    /// blue / red
    pub side: String,
    pub champion: String,
    pub order: i32,
    /// Per-pick rationale comment
    pub comment: Option<String>,
    /// Lane role assignment (top/jungle/mid/bot/support)
    #[serde(default)]
    pub role: Option<String>,
}

/// Best-effort role guess from Data Dragon champion tags.
/// Priority: Marksman > Support > Assassin > Mage > Tank/Fighter > fallback mid.
pub fn guess_role_from_tags(tags: &[String]) -> &'static str {
    if tags.iter().any(|t| t == "Marksman") {
        return "bot";
    }
    if tags.iter().any(|t| t == "Support") {
        return "support";
    }
    if tags.iter().any(|t| t == "Assassin") {
        return "mid";
    }
    if tags.iter().any(|t| t == "Mage") {
        return "mid";
    }
    if tags.iter().any(|t| t == "Tank") || tags.iter().any(|t| t == "Fighter") {
        return "top";
    }
    "mid" // fallback
}

/// Community Dragon SVG URL for a lane role icon.
pub fn role_icon_url(role: &str) -> &'static str {
    match role {
        "top" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-top.svg",
        "jungle" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-jungle.svg",
        "mid" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-middle.svg",
        "bot" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-bottom.svg",
        "support" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-utility.svg",
        _ => "",
    }
}

// ---------------------------------------------------------------------------
// Draft Trees
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DraftTree {
    pub id: Option<String>,
    pub name: String,
    pub team_id: String,
    pub created_by: String,
    pub opponent: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DraftTreeNode {
    pub id: Option<String>,
    pub tree_id: String,
    pub parent_id: Option<String>,
    pub label: String,
    pub notes: Option<String>,
    pub is_improvised: bool,
    pub sort_order: i32,
    pub actions: Vec<DraftAction>,
    pub children: Vec<DraftTreeNode>,
}

// ---------------------------------------------------------------------------
// Ban Priority
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BanPriority {
    pub id: Option<String>,
    pub team_id: String,
    pub champion: String,
    pub rank: i32,
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_round_trips_json() {
        let d = Draft {
            id: Some("draft:1".into()),
            name: "Test Draft".into(),
            team_id: "team:t1".into(),
            created_by: "user:u1".into(),
            opponent: Some("Team Evil".into()),
            notes: None,
            rating: None,
            our_side: "blue".into(),
            actions: vec![],
            comments: vec!["nice play".into()],
            tags: vec!["teamfight".into()],
            win_conditions: None,
            watch_out: None,
            series_id: None,
            game_number: None,
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: Draft = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn draft_action_round_trips_json() {
        let a = DraftAction {
            id: None,
            draft_id: "draft:1".into(),
            phase: "ban1".into(),
            side: "blue".into(),
            champion: "Azir".into(),
            order: 0,
            comment: None,
            role: None,
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: DraftAction = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn draft_action_with_role_round_trips_json() {
        let a = DraftAction {
            id: None,
            draft_id: "draft:1".into(),
            phase: "pick_Pick 1".into(),
            side: "blue".into(),
            champion: "Jinx".into(),
            order: 6,
            comment: None,
            role: Some("bot".into()),
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: DraftAction = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn draft_action_deserializes_without_role_field() {
        // Backward compatibility: old saved drafts lack the "role" key
        let json = r#"{"id":null,"draft_id":"draft:1","phase":"ban1","side":"blue","champion":"Azir","order":0,"comment":null}"#;
        let a: DraftAction = serde_json::from_str(json).unwrap();
        assert_eq!(a.role, None);
    }

    #[test]
    fn guess_role_marksman_returns_bot() {
        assert_eq!(guess_role_from_tags(&["Marksman".into()]), "bot");
    }

    #[test]
    fn guess_role_support_returns_support() {
        assert_eq!(guess_role_from_tags(&["Support".into()]), "support");
    }

    #[test]
    fn guess_role_assassin_returns_mid() {
        assert_eq!(guess_role_from_tags(&["Assassin".into()]), "mid");
    }

    #[test]
    fn guess_role_mage_returns_mid() {
        assert_eq!(guess_role_from_tags(&["Mage".into()]), "mid");
    }

    #[test]
    fn guess_role_tank_returns_top() {
        assert_eq!(guess_role_from_tags(&["Tank".into()]), "top");
    }

    #[test]
    fn guess_role_fighter_returns_top() {
        assert_eq!(guess_role_from_tags(&["Fighter".into()]), "top");
    }

    #[test]
    fn guess_role_marksman_priority_over_assassin() {
        assert_eq!(guess_role_from_tags(&["Marksman".into(), "Assassin".into()]), "bot");
    }

    #[test]
    fn guess_role_empty_tags_returns_mid() {
        assert_eq!(guess_role_from_tags(&[]), "mid");
    }

    #[test]
    fn draft_tree_round_trips_json() {
        let t = DraftTree {
            id: Some("draft_tree:1".into()),
            name: "My Tree".into(),
            team_id: "team:t1".into(),
            created_by: "user:u1".into(),
            opponent: None,
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: DraftTree = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn draft_tree_node_with_children_round_trips_json() {
        let node = DraftTreeNode {
            id: Some("draft_tree_node:1".into()),
            tree_id: "draft_tree:1".into(),
            parent_id: None,
            label: "Root".into(),
            notes: Some("root note".into()),
            is_improvised: false,
            sort_order: 0,
            actions: vec![DraftAction {
                id: None,
                draft_id: String::new(),
                phase: "pick1".into(),
                side: "blue".into(),
                champion: "Jinx".into(),
                order: 1,
                comment: Some("strong ADC pick".into()),
                role: None,
            }],
            children: vec![],
        };
        let json = serde_json::to_string(&node).unwrap();
        let back: DraftTreeNode = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }
}
