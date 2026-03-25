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
    pub mode: String,
    pub riot_region: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RankedInfo {
    pub queue_type: String,
    pub tier: String,
    pub division: String,
    pub lp: i32,
    pub wins: i32,
    pub losses: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct TeamMember {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub roster_type: String,
    pub riot_summoner_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JoinRequest {
    pub id: String,
    pub team_id: String,
    pub user_id: String,
    pub username: String,
    pub riot_summoner_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_member_round_trips_json() {
        let m = TeamMember {
            user_id: "user:abc".into(),
            username: "player1".into(),
            role: "mid".into(),
            roster_type: "starter".into(),
            riot_summoner_name: Some("SummonerX".into()),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: TeamMember = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn team_member_no_summoner_round_trips() {
        let m = TeamMember {
            user_id: "user:1".into(),
            username: "ghost".into(),
            role: "sub".into(),
            roster_type: "sub".into(),
            riot_summoner_name: None,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: TeamMember = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn join_request_round_trips_json() {
        let r = JoinRequest {
            id: "join_request:1".into(),
            team_id: "team:t1".into(),
            user_id: "user:u1".into(),
            username: "alice".into(),
            riot_summoner_name: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: JoinRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
