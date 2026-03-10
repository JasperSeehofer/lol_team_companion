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
    pub actions: Vec<DraftAction>,
    pub comments: Vec<String>,
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
