//! Times in the viewer's own time zone. The server does not know it, so the
//! first render (server and hydration) shows UTC with a "UTC" mark, and the
//! browser switches to local time right after hydration. Dates entered in
//! the browser are converted to UTC the same way (`local_input_to_utc`).

use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Utc};
use leptos::prelude::*;

/// How a [`LocalTime`] reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeStyle {
    /// "Today 19:00", "Tomorrow 19:00", "Sat 11 Oct, 19:00".
    #[default]
    When,
    /// "Sat 19:00".
    Short,
    /// "Saturday 11 October 2026".
    LongDate,
    /// "11 Oct".
    DayMonth,
}

/// Parse an RFC 3339 time.
pub fn parse_utc(iso: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(iso.trim())
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

/// Format `t` in its own zone, relative to `now` in that zone.
pub fn format_time<Tz: TimeZone>(t: &DateTime<Tz>, now: &DateTime<Tz>, style: TimeStyle) -> String
where
    Tz::Offset: std::fmt::Display,
{
    match style {
        TimeStyle::When => {
            let day = t.date_naive();
            let today = now.date_naive();
            let clock = t.format("%H:%M");
            if day == today {
                format!("Today {clock}")
            } else if Some(day) == today.succ_opt() {
                format!("Tomorrow {clock}")
            } else if Some(day) == today.pred_opt() {
                format!("Yesterday {clock}")
            } else if day.year() == today.year() {
                format!("{}, {clock}", t.format("%a %-d %b"))
            } else {
                format!("{}, {clock}", t.format("%a %-d %b %Y"))
            }
        }
        TimeStyle::Short => t.format("%a %H:%M").to_string(),
        TimeStyle::LongDate => t.format("%A %-d %B %Y").to_string(),
        TimeStyle::DayMonth => t.format("%-d %b").to_string(),
    }
}

/// The viewer's local calendar day as `YYYY-MM-DD` (browser only; the
/// server's own day before hydration).
pub fn local_today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// A `datetime-local` style value ("2026-10-11T19:00") in the viewer's zone
/// as RFC 3339 UTC.
pub fn local_input_to_utc(date: &str, time: &str) -> Option<String> {
    let naive = NaiveDateTime::parse_from_str(&format!("{date}T{time}"), "%Y-%m-%dT%H:%M").ok()?;
    let local = Local.from_local_datetime(&naive).earliest()?;
    Some(local.with_timezone(&Utc).to_rfc3339())
}

/// The date and time fields (`YYYY-MM-DD`, `HH:MM`) for a UTC time in the
/// viewer's zone, to prefill an editor.
pub fn utc_to_local_inputs(iso: &str) -> Option<(String, String)> {
    let t = parse_utc(iso)?.with_timezone(&Local);
    Some((
        t.format("%Y-%m-%d").to_string(),
        t.format("%H:%M").to_string(),
    ))
}

/// A time written for the viewer: UTC (marked) until the browser takes over,
/// then local.
///
/// A11y: a `<time>` element with the machine-readable `datetime`; the text
/// changes once after hydration, without moving focus.
#[component]
pub fn LocalTime(
    #[prop(into)] iso: String,
    #[prop(optional)] style: TimeStyle,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let parsed = parse_utc(&iso);
    let text = move || match parsed {
        None => String::new(),
        Some(t) if hydrated.get() => format_time(&t.with_timezone(&Local), &Local::now(), style),
        Some(t) => format!("{} UTC", format_time(&t, &Utc::now(), style)),
    };
    view! { <time datetime=iso class=class>{text}</time> }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    #[test]
    fn when_reads_relative_days() {
        let tz = FixedOffset::east_opt(2 * 3600).unwrap();
        let now = tz.with_ymd_and_hms(2026, 10, 11, 9, 0, 0).unwrap();
        let t = tz.with_ymd_and_hms(2026, 10, 11, 19, 0, 0).unwrap();
        assert_eq!(format_time(&t, &now, TimeStyle::When), "Today 19:00");
        let t = tz.with_ymd_and_hms(2026, 10, 12, 19, 0, 0).unwrap();
        assert_eq!(format_time(&t, &now, TimeStyle::When), "Tomorrow 19:00");
        let t = tz.with_ymd_and_hms(2026, 10, 17, 18, 30, 0).unwrap();
        assert_eq!(format_time(&t, &now, TimeStyle::When), "Sat 17 Oct, 18:30");
        assert_eq!(format_time(&t, &now, TimeStyle::Short), "Sat 18:30");
        assert_eq!(format_time(&t, &now, TimeStyle::DayMonth), "17 Oct");
        let t = tz.with_ymd_and_hms(2027, 1, 2, 18, 0, 0).unwrap();
        assert_eq!(
            format_time(&t, &now, TimeStyle::When),
            "Sat 2 Jan 2027, 18:00"
        );
    }

    #[test]
    fn rfc3339_round_trips() {
        let t = parse_utc("2026-10-11T17:00:00Z").unwrap();
        assert_eq!(t.to_rfc3339(), "2026-10-11T17:00:00+00:00");
        assert!(parse_utc("nonsense").is_none());
    }
}
