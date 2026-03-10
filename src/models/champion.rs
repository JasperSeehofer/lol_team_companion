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
}
