use serde::{Deserialize, Serialize};

/// A closed-beta invite code as shown on the admin page.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Invite {
    pub id: String,
    pub code: String,
    /// Username of the admin who minted it; `None` for seed/bootstrap codes.
    pub created_by: Option<String>,
    pub created_at: String,
    pub max_uses: i64,
    pub uses: i64,
    pub expires_at: Option<String>,
    pub disabled: bool,
    pub note: Option<String>,
}

impl Invite {
    /// Human-readable state: "disabled", "used up", "expired" or "open".
    /// `now_rfc3339` is passed in so the function stays pure and testable.
    pub fn status(&self, now_rfc3339: &str) -> &'static str {
        if self.disabled {
            "disabled"
        } else if self.uses >= self.max_uses {
            "used up"
        } else if self
            .expires_at
            .as_deref()
            .is_some_and(|e| is_before(e, now_rfc3339))
        {
            "expired"
        } else {
            "open"
        }
    }
}

/// `a < b` for two RFC 3339 timestamps, compared as instants (the DB and
/// `chrono` format fractional seconds differently, so comparing the strings
/// was wrong within the expiry second). Falls back to comparing the text when
/// either side does not parse.
fn is_before(a: &str, b: &str) -> bool {
    if a.is_empty() {
        return false;
    }
    match (
        chrono::DateTime::parse_from_rfc3339(a),
        chrono::DateTime::parse_from_rfc3339(b),
    ) {
        (Ok(a), Ok(b)) => a < b,
        _ => a < b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invite() -> Invite {
        Invite {
            id: "invite:1".into(),
            code: "ABCD-EFGH-JKLM".into(),
            created_by: None,
            created_at: "2026-10-01T00:00:00Z".into(),
            max_uses: 2,
            uses: 0,
            expires_at: None,
            disabled: false,
            note: None,
        }
    }

    #[test]
    fn status_reflects_state() {
        let now = "2026-10-11T00:00:00Z";
        assert_eq!(invite().status(now), "open");
        let mut i = invite();
        i.uses = 2;
        assert_eq!(i.status(now), "used up");
        let mut i = invite();
        i.expires_at = Some("2026-10-10T00:00:00Z".into());
        assert_eq!(i.status(now), "expired");
        let mut i = invite();
        i.disabled = true;
        assert_eq!(i.status(now), "disabled");
    }

    #[test]
    fn expiry_compares_instants_not_text() {
        let mut i = invite();
        // Same second, fractional digits on one side only: as text
        // "…00.5Z" sorts before "…00Z", as instants it is later.
        i.expires_at = Some("2026-10-11T00:00:00.500Z".into());
        assert_eq!(i.status("2026-10-11T00:00:00Z"), "open");
        assert_eq!(i.status("2026-10-11T00:00:01+00:00"), "expired");
    }
}
