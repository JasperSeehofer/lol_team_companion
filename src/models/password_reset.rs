//! Password reset codes as the admin page shows them (M137 item 5). The
//! code itself exists only in [`MintedCode`], once, right after minting.

use serde::{Deserialize, Serialize};

/// A code just minted on `/admin/resets`: shown once, never stored in clear.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MintedCode {
    pub email: String,
    pub username: String,
    pub code: String,
    /// RFC 3339, UTC.
    pub expires_at: String,
}

/// One row of the recent codes list: who it was for, who made it, and
/// whether it was used. No code, no hash.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResetSummary {
    pub username: String,
    /// The admin's username; `None` for the reset-password command.
    pub created_by: Option<String>,
    pub created_at: String,
    pub expires_at: String,
    pub used_at: Option<String>,
}

impl ResetSummary {
    /// "used", "expired" or "open" at `now` (RFC 3339).
    pub fn status(&self, now: &str) -> &'static str {
        use chrono::DateTime;
        if self.used_at.is_some() {
            return "used";
        }
        match (
            DateTime::parse_from_rfc3339(&self.expires_at),
            DateTime::parse_from_rfc3339(now),
        ) {
            (Ok(exp), Ok(now)) if exp > now => "open",
            _ => "expired",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_reads_use_and_expiry() {
        let mut r = ResetSummary {
            username: "a".into(),
            created_by: None,
            created_at: "2026-10-14T10:00:00Z".into(),
            expires_at: "2026-10-14T10:15:00.250Z".into(),
            used_at: None,
        };
        assert_eq!(r.status("2026-10-14T10:14:59Z"), "open");
        assert_eq!(r.status("2026-10-14T10:15:01Z"), "expired");
        r.used_at = Some("2026-10-14T10:05:00Z".into());
        assert_eq!(r.status("2026-10-14T10:14:59Z"), "used");
    }
}
