use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct Team {
    pub id: Option<String>,
    pub name: String,
    pub region: String,
    pub created_by: String,
    pub member_count: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TeamMembership {
    pub team_id: String,
    pub user_id: String,
    /// top / jungle / mid / bot / support / sub
    pub role: String,
}
