use chrono::{DateTime, Utc};

/// The message of a server-function error as users should read it: without
/// the "error running server function: " wrapper Leptos puts in front.
pub fn error_text(e: &impl std::fmt::Display) -> String {
    let msg = e.to_string();
    msg.strip_prefix("error running server function: ")
        .unwrap_or(&msg)
        .to_string()
}

/// Formats a UTC ISO 8601 timestamp string into a human-readable relative or absolute string.
///
/// Examples:
/// - "just now" (< 1 minute)
/// - "3 minutes ago" (< 1 hour)
/// - "2 hours ago" (< 24 hours)
/// - "19 Mar, 14:30" (same year, older than 24 hours)
/// - "5 Dec 2025, 14:30" (different year)
/// - Returns the input string unchanged on parse failure.
pub fn format_timestamp(s: &str) -> String {
    format_timestamp_with_now(s, Utc::now())
}

fn format_timestamp_with_now(s: &str, now: DateTime<Utc>) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    let dt = match s.parse::<DateTime<Utc>>() {
        Ok(d) => d,
        Err(_) => return s.to_string(),
    };

    let age = now.signed_duration_since(dt);

    // Future timestamps (clock skew) — fall through to absolute
    if age.num_seconds() < 0 {
        return format_absolute(dt, now);
    }

    if age.num_seconds() < 60 {
        return "just now".to_string();
    }

    let mins = age.num_minutes();
    if mins < 60 {
        if mins == 1 {
            return "1 minute ago".to_string();
        }
        return format!("{} minutes ago", mins);
    }

    let hours = age.num_hours();
    if hours < 24 {
        if hours == 1 {
            return "1 hour ago".to_string();
        }
        return format!("{} hours ago", hours);
    }

    format_absolute(dt, now)
}

fn format_absolute(dt: DateTime<Utc>, now: DateTime<Utc>) -> String {
    if dt.format("%Y").to_string() == now.format("%Y").to_string() {
        // Same year: "19 Mar, 14:30"
        // %-d gives unpadded day on Linux
        dt.format("%-d %b, %H:%M").to_string()
    } else {
        // Different year: "5 Dec 2025, 14:30"
        dt.format("%-d %b %Y, %H:%M").to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn error_text_strips_the_wrapper() {
        assert_eq!(
            error_text(&"error running server function: Not found"),
            "Not found"
        );
        assert_eq!(error_text(&"Plain message"), "Plain message");
    }

    use super::*;
    use chrono::Utc;

    fn make_now(s: &str) -> DateTime<Utc> {
        s.parse::<DateTime<Utc>>().unwrap()
    }

    #[test]
    fn test_just_now() {
        let ts = "2026-03-19T14:30:00.000000000Z";
        let now = make_now("2026-03-19T14:30:30Z");
        assert_eq!(format_timestamp_with_now(ts, now), "just now");
    }

    #[test]
    fn test_minutes_ago_plural() {
        let ts = "2026-03-19T14:00:00Z";
        let now = make_now("2026-03-19T14:23:00Z");
        assert_eq!(format_timestamp_with_now(ts, now), "23 minutes ago");
    }

    #[test]
    fn test_minutes_ago_singular() {
        let ts = "2026-03-19T14:00:00Z";
        let now = make_now("2026-03-19T14:01:30Z");
        assert_eq!(format_timestamp_with_now(ts, now), "1 minute ago");
    }

    #[test]
    fn test_hours_ago_plural() {
        let ts = "2026-03-19T08:00:00Z";
        let now = make_now("2026-03-19T14:00:00Z");
        assert_eq!(format_timestamp_with_now(ts, now), "6 hours ago");
    }

    #[test]
    fn test_hours_ago_singular() {
        let ts = "2026-03-19T08:00:00Z";
        let now = make_now("2026-03-19T09:00:00Z");
        assert_eq!(format_timestamp_with_now(ts, now), "1 hour ago");
    }

    #[test]
    fn test_same_year_absolute() {
        let ts = "2026-03-19T14:30:00Z";
        let now = make_now("2026-03-20T15:00:00Z"); // next day, same year
        assert_eq!(format_timestamp_with_now(ts, now), "19 Mar, 14:30");
    }

    #[test]
    fn test_different_year_absolute() {
        let ts = "2025-12-05T14:30:00Z";
        let now = make_now("2026-03-22T10:00:00Z");
        assert_eq!(format_timestamp_with_now(ts, now), "5 Dec 2025, 14:30");
    }

    #[test]
    fn test_parse_failure_returns_input() {
        let ts = "invalid";
        let now = make_now("2026-03-22T10:00:00Z");
        assert_eq!(format_timestamp_with_now(ts, now), "invalid");
    }

    #[test]
    fn test_empty_string_returns_empty() {
        let ts = "";
        let now = make_now("2026-03-22T10:00:00Z");
        assert_eq!(format_timestamp_with_now(ts, now), "");
    }

    #[test]
    fn test_future_timestamp_falls_to_absolute() {
        // A timestamp 5 minutes in the future — should not show "just now"
        let ts = "2026-03-22T10:10:00Z";
        let now = make_now("2026-03-22T10:05:00Z");
        // Since ts is in same year as now, falls through to absolute
        let result = format_timestamp_with_now(ts, now);
        assert_eq!(result, "22 Mar, 10:10");
    }
}
