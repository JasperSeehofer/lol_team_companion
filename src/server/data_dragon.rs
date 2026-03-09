use crate::models::champion::Champion;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataDragonError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

const DDRAGON_VERSION: &str = "14.10.1";

pub async fn fetch_champions() -> Result<Vec<Champion>, DataDragonError> {
    let url = format!(
        "https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json",
        DDRAGON_VERSION
    );

    let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;

    let data = resp
        .get("data")
        .and_then(|d| d.as_object())
        .ok_or_else(|| DataDragonError::Parse("Missing 'data' field".into()))?;

    let champions = data
        .values()
        .filter_map(|v| {
            let id = v.get("id")?.as_str()?.to_string();
            let name = v.get("name")?.as_str()?.to_string();
            let title = v.get("title")?.as_str()?.to_string();
            let tags = v
                .get("tags")?
                .as_array()?
                .iter()
                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                .collect();
            let image_full = v
                .get("image")?
                .get("full")?
                .as_str()?
                .to_string();
            Some(Champion { id, name, title, tags, image_full })
        })
        .collect();

    Ok(champions)
}
