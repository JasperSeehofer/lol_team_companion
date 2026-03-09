use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: Option<String>,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub riot_puuid: Option<String>,
    pub riot_summoner_name: Option<String>,
}

/// Subset of user data safe to expose to the client
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PublicUser {
    pub id: String,
    pub username: String,
    pub riot_summoner_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct TeamMember {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub riot_summoner_name: Option<String>,
}
