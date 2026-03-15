use crate::models::champion::Champion;
use thiserror::Error;

/// Normalize a champion name input to its canonical Data Dragon ID.
///
/// Lookup order:
/// 1. Exact match on `c.id == input`
/// 2. Case-insensitive display name match: `c.name.to_lowercase() == input.to_lowercase()`
/// 3. Fuzzy match by stripping non-alphanumeric chars and lowercasing both sides
///
/// Returns `Some(canonical_id)` or `None` if no champion matches.
#[cfg(feature = "ssr")]
pub fn normalize_champion_name(input: &str, champions: &[Champion]) -> Option<String> {
    // Pass 1: exact ID match
    if let Some(c) = champions.iter().find(|c| c.id == input) {
        return Some(c.id.clone());
    }

    // Pass 2: case-insensitive display name match
    let input_lower = input.to_lowercase();
    if let Some(c) = champions.iter().find(|c| c.name.to_lowercase() == input_lower) {
        return Some(c.id.clone());
    }

    // Pass 3: fuzzy match — strip non-alphanumeric chars and lowercase both
    let input_stripped: String = input.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
    if input_stripped.is_empty() {
        return None;
    }
    champions.iter().find(|c| {
        let id_stripped: String = c.id.chars().filter(|ch| ch.is_alphanumeric()).collect::<String>().to_lowercase();
        id_stripped == input_stripped
    }).map(|c| c.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_champions() -> Vec<Champion> {
        vec![
            Champion {
                id: "KSante".into(),
                name: "K'Sante".into(),
                title: "The Pride of Nazumah".into(),
                tags: vec!["Tank".into()],
                image_full: "KSante.png".into(),
            },
            Champion {
                id: "AurelionSol".into(),
                name: "Aurelion Sol".into(),
                title: "The Star Forger".into(),
                tags: vec!["Mage".into()],
                image_full: "AurelionSol.png".into(),
            },
            Champion {
                id: "Jinx".into(),
                name: "Jinx".into(),
                title: "The Loose Cannon".into(),
                tags: vec!["Marksman".into()],
                image_full: "Jinx.png".into(),
            },
            Champion {
                id: "Belveth".into(),
                name: "Bel'Veth".into(),
                title: "The Empress of the Void".into(),
                tags: vec!["Fighter".into()],
                image_full: "Belveth.png".into(),
            },
        ]
    }

    #[test]
    fn test_normalize_champion_name() {
        let champs = sample_champions();

        // exact ID match
        assert_eq!(normalize_champion_name("KSante", &champs), Some("KSante".into()));
        assert_eq!(normalize_champion_name("Jinx", &champs), Some("Jinx".into()));

        // case-insensitive display name match
        assert_eq!(normalize_champion_name("K'Sante", &champs), Some("KSante".into()));
        assert_eq!(normalize_champion_name("aurelion sol", &champs), Some("AurelionSol".into()));
        assert_eq!(normalize_champion_name("JINX", &champs), Some("Jinx".into()));

        // fuzzy stripped lowercase match
        assert_eq!(normalize_champion_name("ksante", &champs), Some("KSante".into()));
        assert_eq!(normalize_champion_name("bel'veth", &champs), Some("Belveth".into()));

        // no match
        assert_eq!(normalize_champion_name("NotAChampion", &champs), None);
        assert_eq!(normalize_champion_name("", &champs), None);
    }
}

#[derive(Debug, Error)]
pub enum DataDragonError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

async fn fetch_latest_version() -> Result<String, DataDragonError> {
    let versions: Vec<String> =
        reqwest::get("https://ddragon.leagueoflegends.com/api/versions.json")
            .await?
            .json()
            .await?;
    versions
        .into_iter()
        .next()
        .ok_or_else(|| DataDragonError::Parse("Empty versions list".into()))
}

pub async fn fetch_champions() -> Result<Vec<Champion>, DataDragonError> {
    let version = fetch_latest_version().await?;
    let url = format!(
        "https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json",
        version
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
            let image_filename = v.get("image")?.get("full")?.as_str()?.to_string();
            let image_full = format!(
                "https://ddragon.leagueoflegends.com/cdn/{}/img/champion/{}",
                version, image_filename
            );
            Some(Champion {
                id,
                name,
                title,
                tags,
                image_full,
            })
        })
        .collect();

    Ok(champions)
}
