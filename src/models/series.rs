use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Series {
    pub id: Option<String>,
    pub name: String,
    pub team_id: String,
    pub opponent_id: Option<String>,
    pub opponent_name: Option<String>,
    /// "bo1", "bo3", or "bo5"
    pub format: String,
    pub is_fearless: bool,
    pub notes: Option<String>,
    pub created_by: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn series_round_trips_json() {
        let s = Series {
            id: Some("series:1".into()),
            name: "Semifinal".into(),
            team_id: "team:t1".into(),
            opponent_id: Some("opponent:1".into()),
            opponent_name: Some("Evil Geniuses".into()),
            format: "bo3".into(),
            is_fearless: true,
            notes: None,
            created_by: "user:u1".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Series = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
