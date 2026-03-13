use thiserror::Error;

#[derive(Debug, Error)]
pub enum RiotError {
    #[error("Riot API error: {0}")]
    Api(String),
    #[error("riven error: {0}")]
    Riven(#[from] riven::RiotApiError),
}

pub struct MatchData {
    pub match_id: String,
    pub queue_id: i32,
    pub game_duration: i32,
    pub game_end_epoch_ms: Option<i64>,
    pub champion: String,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub cs: i32,
    pub vision_score: i32,
    pub damage: i32,
    pub win: bool,
}

pub fn has_api_key() -> bool {
    let key = std::env::var("RIOT_API_KEY").unwrap_or_default();
    !key.is_empty()
}

fn api() -> riven::RiotApi {
    let key = std::env::var("RIOT_API_KEY").unwrap_or_default();
    riven::RiotApi::new(key)
}

pub async fn get_puuid(game_name: &str, tag_line: &str) -> Result<String, RiotError> {
    let api = api();
    let account = api
        .account_v1()
        .get_by_riot_id(riven::consts::RegionalRoute::EUROPE, game_name, tag_line)
        .await?
        .ok_or_else(|| RiotError::Api(format!("Account {game_name}#{tag_line} not found")))?;
    Ok(account.puuid)
}

pub async fn fetch_match_history(
    puuid: &str,
    queue_id: Option<i32>,
) -> Result<Vec<MatchData>, RiotError> {
    let api = api();

    let queue_filter = queue_id.map(|q| riven::consts::Queue::from(q as u16));

    let match_ids = api
        .match_v5()
        .get_match_ids_by_puuid(
            riven::consts::RegionalRoute::EUROPE,
            puuid,
            Some(20),
            None,
            queue_filter,
            None,
            None,
            None,
        )
        .await?;

    let mut results = Vec::new();

    for mid in match_ids {
        let Some(m) = api
            .match_v5()
            .get_match(riven::consts::RegionalRoute::EUROPE, &mid)
            .await?
        else {
            continue;
        };

        let participant = m.info.participants.iter().find(|p| p.puuid == puuid);

        let Some(p) = participant else { continue };

        results.push(MatchData {
            match_id: mid,
            queue_id: u16::from(m.info.queue_id) as i32,
            game_duration: m.info.game_duration as i32,
            game_end_epoch_ms: m.info.game_end_timestamp,
            champion: p.champion_name.clone(),
            kills: p.kills,
            deaths: p.deaths,
            assists: p.assists,
            cs: p.total_minions_killed + p.neutral_minions_killed,
            vision_score: p.vision_score,
            damage: p.total_damage_dealt_to_champions,
            win: p.win,
        });
    }

    Ok(results)
}

pub async fn fetch_player_champions(puuid: &str, count: usize) -> Result<Vec<String>, RiotError> {
    let api = api();
    let match_ids = api
        .match_v5()
        .get_match_ids_by_puuid(
            riven::consts::RegionalRoute::EUROPE,
            puuid,
            Some(count as i32),
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    let mut champions = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for mid in match_ids {
        let Some(m) = api
            .match_v5()
            .get_match(riven::consts::RegionalRoute::EUROPE, &mid)
            .await?
        else {
            continue;
        };
        if let Some(p) = m.info.participants.iter().find(|p| p.puuid == puuid) {
            if seen.insert(p.champion_name.clone()) {
                champions.push(p.champion_name.clone());
            }
        }
    }

    Ok(champions)
}
