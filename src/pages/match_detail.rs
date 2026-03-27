use leptos::prelude::*;

#[server]
pub async fn fetch_match_detail(
    match_id: String,
) -> Result<crate::models::match_data::MatchDetail, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::{db, riot};
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    if match_id.is_empty() {
        return Err(ServerFnError::new("No match ID provided"));
    }

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    // Cache hit path
    if let Some((participants, timeline_events, game_duration, game_mode)) =
        db::get_cached_match_detail(&surreal, &match_id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        let user_pid = participants
            .iter()
            .find(|p| p.puuid == user.riot_puuid.as_deref().unwrap_or(""))
            .map(|p| p.participant_id)
            .unwrap_or(1);
        let performance = riot::compute_performance(&participants, user_pid, game_duration);

        return Ok(crate::models::match_data::MatchDetail {
            match_id: match_id.clone(),
            game_duration,
            game_mode,
            participants,
            timeline_events,
            user_participant_id: user_pid,
            user_puuid: user.riot_puuid.clone().unwrap_or_default(),
            performance,
        });
    }

    // Cache miss: fetch from Riot API
    if !riot::has_api_key() {
        return Err(ServerFnError::new(
            "Riot API key not configured. Ask an admin to add a RIOT_API_KEY.",
        ));
    }

    let platform = riot::platform_route_from_str(user.riot_region.as_deref().unwrap_or("EUW"));
    let full = riot::fetch_full_match_detail(&match_id, platform)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_pid = full
        .participants
        .iter()
        .find(|p| p.puuid == user.riot_puuid.as_deref().unwrap_or(""))
        .map(|p| p.participant_id)
        .unwrap_or(1);
    let performance = riot::compute_performance(&full.participants, user_pid, full.game_duration);

    // Store in cache (non-fatal on failure)
    if let Err(e) = db::store_match_detail(
        &surreal,
        &match_id,
        &full.participants,
        &full.raw_timeline_events,
        full.game_duration,
        &full.game_mode,
    )
    .await
    {
        tracing::warn!("Failed to cache match detail for {match_id}: {e}");
    }

    Ok(crate::models::match_data::MatchDetail {
        match_id,
        game_duration: full.game_duration,
        game_mode: full.game_mode,
        participants: full.participants,
        timeline_events: full.raw_timeline_events,
        user_participant_id: user_pid,
        user_puuid: user.riot_puuid.unwrap_or_default(),
        performance,
    })
}

// Placeholder component -- will be fully implemented in Plan 02
#[component]
pub fn MatchDetailPage() -> impl IntoView {
    view! { <p>"Match detail page placeholder"</p> }
}
