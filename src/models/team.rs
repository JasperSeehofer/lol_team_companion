use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use surrealdb::types::SurrealValue;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(surrealdb_types_derive::SurrealValue))]
pub struct Team {
    pub id: Option<String>,
    pub name: String,
    pub region: String,
    pub created_by: String,
    pub member_count: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TeamMembership {
    pub team_id: String,
    pub user_id: String,
    /// top / jungle / mid / bot / support / sub
    pub role: String,
}

/// M160 item 3: may the actor move the target member to a roster place?
/// The leader may place anyone, including onto "coach" and including the
/// leader themselves (hand-over is a separate, leader-only action). A
/// coach may place only plain members and substitutes — never the leader,
/// never another coach, and never onto "coach": only the leader makes or
/// unmakes coaches. The caller gates membership first
/// (`authz::require_leader_or_coach`), so a plain member never reaches this
/// rule; with `actor_is_leader = false` the actor is a coach.
pub fn may_set_place(
    actor_is_leader: bool,
    target_is_leader: bool,
    target_is_coach: bool,
    new_place_is_coach: bool,
) -> bool {
    if actor_is_leader {
        return true;
    }
    !target_is_leader && !target_is_coach && !new_place_is_coach
}

#[cfg(test)]
mod tests {
    use super::may_set_place;

    #[test]
    fn the_leader_may_place_anyone_anywhere() {
        // All eight target/place combinations pass for the leader.
        for target_is_leader in [false, true] {
            for target_is_coach in [false, true] {
                for new_place_is_coach in [false, true] {
                    assert!(
                        may_set_place(true, target_is_leader, target_is_coach, new_place_is_coach),
                        "leader, target_is_leader={target_is_leader}, target_is_coach={target_is_coach}, new_place_is_coach={new_place_is_coach}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_coach_places_only_plain_members_and_never_makes_coaches() {
        // The one allowed combination: a plain member or substitute to any
        // non-coach place (starter slot, bench, off the roster).
        assert!(may_set_place(false, false, false, false));
        // Never the leader, never another coach, never onto "coach".
        assert!(
            !may_set_place(false, true, false, false),
            "coach moves the leader"
        );
        assert!(
            !may_set_place(false, false, true, false),
            "coach moves a coach"
        );
        assert!(
            !may_set_place(false, false, false, true),
            "coach makes a coach"
        );
        assert!(!may_set_place(false, true, false, true));
        assert!(!may_set_place(false, false, true, true));
        assert!(!may_set_place(false, true, true, false));
        assert!(!may_set_place(false, true, true, true));
    }
}
