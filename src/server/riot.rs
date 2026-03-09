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
    pub champion: String,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub cs: i32,
    pub vision_score: i32,
    pub damage: i32,
    pub win: bool,
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

pub async fn fetch_match_history(puuid: &str, queue_id: i32) -> Result<Vec<MatchData>, RiotError> {
    let api = api();

    let match_ids = api
        .match_v5()
        .get_match_ids_by_puuid(
            riven::consts::RegionalRoute::EUROPE,
            puuid,
            Some(20),
            None,
            Some(riven::consts::Queue::from(queue_id as u16)),
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

        let participant = m
            .info
            .participants
            .iter()
            .find(|p| p.puuid == puuid);

        let Some(p) = participant else { continue };

        results.push(MatchData {
            match_id: mid,
            queue_id,
            game_duration: m.info.game_duration as i32,
            champion: p.champion_name.clone(),
            kills: p.kills as i32,
            deaths: p.deaths as i32,
            assists: p.assists as i32,
            cs: (p.total_minions_killed + p.neutral_minions_killed) as i32,
            vision_score: p.vision_score as i32,
            damage: p.total_damage_dealt_to_champions as i32,
            win: p.win,
        });
    }

    Ok(results)
}
