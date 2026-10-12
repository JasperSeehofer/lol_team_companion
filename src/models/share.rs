//! Share links (M5): read-only views of one game plan, review or player
//! card, opened without signing in. Only what the shared artefact holds
//! crosses the wire: no ids of other records, no team id.

use serde::{Deserialize, Serialize};

/// What a link shares.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShareKind {
    GamePlan,
    Review,
    Player,
}

impl ShareKind {
    pub fn key(self) -> &'static str {
        match self {
            ShareKind::GamePlan => "game_plan",
            ShareKind::Review => "review",
            ShareKind::Player => "player",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "game_plan" => Some(ShareKind::GamePlan),
            "review" => Some(ShareKind::Review),
            "player" => Some(ShareKind::Player),
            _ => None,
        }
    }
    /// The table the target lives in.
    pub fn table(self) -> &'static str {
        match self {
            ShareKind::GamePlan => "game_plan",
            ShareKind::Review => "post_game_learning",
            ShareKind::Player => "user",
        }
    }
    /// "game plan", "review", "player card".
    pub fn noun(self) -> &'static str {
        match self {
            ShareKind::GamePlan => "game plan",
            ShareKind::Review => "review",
            ShareKind::Player => "player card",
        }
    }
}

/// Lifetimes a link can be given, in days.
pub const SHARE_DAYS: [u32; 3] = [1, 7, 30];

/// A share link as its team sees it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ShareLink {
    pub id: String,
    pub token: String,
    pub kind: ShareKind,
    pub created_by_name: String,
    pub created_at: Option<String>,
    pub expires_at: String,
    pub revoked: bool,
}

/// The path a token opens.
pub fn share_path(token: &str) -> String {
    format!("/s/{token}")
}

/// A shared game plan.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SharedPlan {
    pub name: String,
    pub our_champions: Vec<String>,
    pub enemy_champions: Vec<String>,
    pub win_conditions: Vec<String>,
    pub objective_priority: Vec<String>,
    pub teamfight_strategy: String,
    pub early_game: Option<String>,
    /// (role word, brief) for each role with a brief.
    pub role_briefs: Vec<(String, String)>,
    pub notes: Option<String>,
}

/// A shared review.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SharedReview {
    pub won: Option<bool>,
    pub rating: Option<u8>,
    pub what_went_well: Vec<String>,
    pub improvements: Vec<String>,
    pub action_items: Vec<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

/// A pool entry on a player card.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CardChampion {
    pub champion: String,
    /// "comfort", "match_ready", ...
    pub tier: String,
}

/// A player card: name, position, top pool champions, recent form.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlayerCard {
    pub username: String,
    /// The team's name for the card's caption.
    pub team_name: String,
    /// Stored position (top ... support) or "unassigned".
    pub role: String,
    /// starter, substitute, coach, member.
    pub place: String,
    pub champions: Vec<CardChampion>,
    /// Last five stored games, oldest first.
    pub form: Vec<bool>,
    /// (champion, games, wins) for the most played champions in stored games.
    pub most_played: Vec<(String, u32, u32)>,
}

/// What a share link opens.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SharedView {
    GamePlan(SharedPlan),
    Review(SharedReview),
    Player(PlayerCard),
}

/// The public page's payload: the artefact plus when the link expires.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Shared {
    pub view: SharedView,
    pub expires_at: String,
}

/// Rank of a pool tier, best first (unknown tiers last).
pub fn tier_rank(tier: &str) -> usize {
    match tier {
        "comfort" => 0,
        "match_ready" => 1,
        "scrim_ready" => 2,
        "practicing" => 3,
        "to_practise" | "to_practice" => 4,
        _ => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip() {
        for k in [ShareKind::GamePlan, ShareKind::Review, ShareKind::Player] {
            assert_eq!(ShareKind::parse(k.key()), Some(k));
        }
        assert_eq!(ShareKind::parse("draft"), None);
        assert_eq!(share_path("abc"), "/s/abc");
    }
}
