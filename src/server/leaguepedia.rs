use thiserror::Error;

#[derive(Debug, Error)]
pub enum LeaguepediaError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

#[derive(Debug, serde::Deserialize)]
pub struct ProGame {
    pub blue_team: String,
    pub red_team: String,
    pub winner: String,
    pub blue_picks: Vec<String>,
    pub red_picks: Vec<String>,
}

/// Fetch recent tournament games from Leaguepedia Cargo API.
/// Returns an empty list on error (non-critical feature).
pub async fn fetch_recent_games(tournament: &str, limit: u32) -> Result<Vec<ProGame>, LeaguepediaError> {
    let url = format!(
        "https://lol.fandom.com/api.php?action=cargoquery&format=json&limit={limit}&tables=PicksAndBansS7&fields=Team1,Team2,Winner,Team1Picks,Team2Picks&where=Tournament=%27{tournament}%27&order_by=GameId+DESC",
    );

    let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;

    let items = resp
        .get("cargoquery")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let games = items
        .into_iter()
        .filter_map(|item| {
            let title = item.get("title")?;
            let parse_picks = |field: &str| -> Vec<String> {
                title.get(field)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };
            Some(ProGame {
                blue_team: title.get("Team1")?.as_str()?.to_string(),
                red_team: title.get("Team2")?.as_str()?.to_string(),
                winner: title.get("Winner")?.as_str()?.to_string(),
                blue_picks: parse_picks("Team1Picks"),
                red_picks: parse_picks("Team2Picks"),
            })
        })
        .collect();

    Ok(games)
}
