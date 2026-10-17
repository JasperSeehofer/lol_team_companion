//! Return to the page after sign-in (M148 item 8): the `?next=` value on
//! `/auth/login`. Pure and shared by both targets: the browser builds the
//! sign-in link with [`login_href_for`], the sign-in server function
//! checks the value with [`safe_next`] before it redirects there.
//!
//! The value comes from the address bar, so anyone can forge it. Only a
//! path on this site is accepted (never another host, a scheme or a
//! protocol-relative `//host`), so a link cannot send a user who signs in
//! to someone else's site (an open redirect).

/// The longest `next` value accepted; longer ones go to Home.
pub const NEXT_MAX_LEN: usize = 512;

/// The sign-in page.
pub const LOGIN_PATH: &str = "/auth/login";

/// `raw` when it is a safe same-origin path to return to after sign-in,
/// otherwise `None` (the caller goes to `/`).
///
/// Accepted: a path of printable ASCII (no spaces, no control characters)
/// that starts with one `/`, has no backslash, no `://` before the query,
/// and is not under `/auth/` (a loop back to sign-in) or `/api/` (server
/// functions). The same checks run on the percent-decoded form, so
/// `/%2F%2Fevil.com` is refused like `//evil.com`.
pub fn safe_next(raw: &str) -> Option<String> {
    if raw.is_empty() || raw.len() > NEXT_MAX_LEN {
        return None;
    }
    if !path_is_safe(raw) {
        return None;
    }
    let decoded = percent_decode(raw)?;
    if !path_is_safe(&decoded) {
        return None;
    }
    Some(raw.to_string())
}

/// Where to go after sign-in: the safe `next`, or Home.
pub fn next_or_home(raw: Option<&str>) -> String {
    raw.and_then(safe_next).unwrap_or_else(|| "/".to_string())
}

/// The sign-in link for someone signed out on `path_and_query` (the
/// current `pathname + search`): `/auth/login?next=<encoded>`, or plain
/// `/auth/login` for Home, the auth pages and anything [`safe_next`]
/// refuses.
pub fn login_href_for(path_and_query: &str) -> String {
    match safe_next(path_and_query) {
        Some(p) if p != "/" && !p.starts_with("/?") && !p.starts_with("/#") => {
            format!("{LOGIN_PATH}?next={}", encode_component(&p))
        }
        _ => LOGIN_PATH.to_string(),
    }
}

/// Every check except the decoding.
fn path_is_safe(p: &str) -> bool {
    let bytes = p.as_bytes();
    // One leading slash, never two (`//host` is another site) and never a
    // slash followed by a backslash (browsers read `/\host` as `//host`).
    if bytes.first() != Some(&b'/') || matches!(bytes.get(1), Some(b'/') | Some(b'\\')) {
        return false;
    }
    // Printable ASCII only: no whitespace, no control characters (a tab or
    // newline inside a URL is dropped by browsers, which can join `/` and
    // `/evil` into `//evil`), no backslash anywhere.
    if !bytes
        .iter()
        .all(|b| (0x21..=0x7e).contains(b) && *b != b'\\')
    {
        return false;
    }
    let path = p.split(['?', '#']).next().unwrap_or_default();
    if path.contains("://") {
        return false;
    }
    let lower = path.to_ascii_lowercase();
    !(lower == "/auth"
        || lower.starts_with("/auth/")
        || lower == "/api"
        || lower.starts_with("/api/"))
}

/// Decode `%XX` escapes once. `None` when the result is not UTF-8. A `%`
/// not followed by two hex digits is kept as it is.
fn percent_decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).ok()
}

fn hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Percent-encode everything but the unreserved characters (RFC 3986),
/// like `encodeURIComponent` but stricter: `/draft` becomes `%2Fdraft`.
pub fn encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_same_origin_paths() {
        for ok in [
            "/",
            "/draft",
            "/draft?draft_id=x",
            "/match/abc#t",
            "/game-plan?plan_id=game_plan%3Aabc",
            "/team/player/u1",
        ] {
            assert_eq!(safe_next(ok).as_deref(), Some(ok), "{ok}");
        }
    }

    #[test]
    fn refuses_other_hosts_schemes_and_loops() {
        for bad in [
            "//evil.com",
            "/\\evil.com",
            "/\\/evil.com",
            "https://evil",
            "HTTPS://EVIL.COM",
            "javascript:alert(1)",
            "evil.com",
            "",
            "/%2F%2Fevil.com",
            "/%2f%2fevil.com",
            "/%5Cevil.com",
            "/%5C%5Cevil.com",
            "/\tfoo",
            "/\t/evil.com",
            "/x\n",
            " /x",
            "/foo bar",
            "/auth/login",
            "/auth/register?invite=x",
            "/AUTH/login",
            "/auth",
            "/%61uth/login",
            "/api/login_action123",
            "/x/https://evil",
            "/caf\u{e9}",
            "/%FF",
        ] {
            assert_eq!(safe_next(bad), None, "{bad:?}");
        }
        assert_eq!(safe_next(&format!("/{}", "a".repeat(NEXT_MAX_LEN))), None);
        assert!(safe_next(&format!("/{}", "a".repeat(NEXT_MAX_LEN - 1))).is_some());
    }

    #[test]
    fn next_or_home_falls_back_to_home() {
        assert_eq!(next_or_home(None), "/");
        assert_eq!(next_or_home(Some("//evil.example")), "/");
        assert_eq!(next_or_home(Some("/stats")), "/stats");
    }

    #[test]
    fn login_href_encodes_the_current_page() {
        assert_eq!(login_href_for("/draft"), "/auth/login?next=%2Fdraft");
        assert_eq!(
            login_href_for("/draft?draft_id=a b"),
            "/auth/login",
            "a value safe_next refuses gets no next"
        );
        assert_eq!(
            login_href_for("/game-plan?plan_id=x&y=1"),
            "/auth/login?next=%2Fgame-plan%3Fplan_id%3Dx%26y%3D1"
        );
        assert_eq!(login_href_for("/"), "/auth/login");
        assert_eq!(login_href_for("/?guide=1"), "/auth/login");
        assert_eq!(login_href_for("/auth/reset"), "/auth/login");
        assert_eq!(login_href_for(""), "/auth/login");
    }

    #[test]
    fn decoding_keeps_stray_percent_signs() {
        assert_eq!(percent_decode("/a%2"), Some("/a%2".to_string()));
        assert_eq!(percent_decode("/a%zz"), Some("/a%zz".to_string()));
        assert_eq!(percent_decode("/a%41"), Some("/aA".to_string()));
        assert_eq!(safe_next("/a%2").as_deref(), Some("/a%2"));
    }
}
