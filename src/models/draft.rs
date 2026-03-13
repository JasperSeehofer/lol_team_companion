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
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: DraftAction = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
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
            }],
            children: vec![],
        };
        let json = serde_json::to_string(&node).unwrap();
        let back: DraftTreeNode = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }
}
