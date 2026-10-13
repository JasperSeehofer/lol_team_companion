//! Runtime configuration, read and validated once at startup.
//!
//! [`AppConfig::from_env`] reads every variable the server uses, checks it,
//! and either returns the configuration with a list of warnings or the full
//! list of errors (a typo such as `TRUST_FORWARDED_FOR=ture` stops the
//! server instead of silently falling back to a default). [`VARIABLES`] is
//! the one table of variables: the `CLAUDE.md` environment table is checked
//! against it by a test, and so is every name `from_env` reads.
//!
//! The security part ([`SecurityConfig`]) is installed with [`init`] and read
//! anywhere with [`get`]; the rest is handed to `main`.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Clone)]
pub struct SecurityConfig {
    pub is_dev: bool,
    pub closed_beta: bool,
    pub cookie_secure: bool,
    pub session_secret: Vec<u8>,
    pub admin_emails: Vec<String>,
    pub bootstrap_invite: Option<(String, i64)>,
    pub rate_limit_multiplier: u32,
    pub trust_forwarded_for: bool,
    /// `SUPPORT_URL`: the one external support page (M7), shown only on
    /// /transparency. Accepted only as an absolute `https://` URL without
    /// credentials.
    pub support_url: Option<String>,
}

static CONFIG: OnceLock<SecurityConfig> = OnceLock::new();

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// True when `APP_ENV` or `LEPTOS_ENV` marks a development run.
pub fn is_dev_env(app_env: Option<&str>, leptos_env: Option<&str>) -> bool {
    let dev = |v: &str| {
        matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "dev" | "development" | "local"
        )
    };
    app_env.is_some_and(dev) || leptos_env.is_some_and(dev)
}

/// The invite gate is on unless this is a dev run AND `CLOSED_BETA` turns it
/// off. Outside dev the variable is ignored, so a stray `CLOSED_BETA=false`
/// cannot open a production instance.
pub fn closed_beta_enabled(is_dev: bool, closed_beta_var: Option<&str>) -> bool {
    if !is_dev {
        return true;
    }
    !matches!(closed_beta_var.and_then(parse_bool), Some(false))
}

/// `SUPPORT_URL` if it is an absolute `https://` URL with a host, without
/// credentials, whitespace or control characters; otherwise `None`.
pub fn parse_support_url(value: Option<&str>) -> Option<String> {
    let v = value?.trim();
    let rest = v.strip_prefix("https://")?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = authority.split(':').next().unwrap_or("");
    let ok = v.len() <= 500
        && !host.is_empty()
        && !authority.contains('@')
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        && !v
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '"' || c == '<' || c == '>');
    ok.then(|| v.to_string())
}

/// Parse `ADMIN_EMAILS`: comma or whitespace separated, normalised.
pub fn parse_admin_emails(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(|e| e.trim().to_lowercase())
        .filter(|e| !e.is_empty())
        .collect()
}

impl std::fmt::Debug for SecurityConfig {
    /// Never prints the session secret or the admin addresses.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecurityConfig")
            .field("is_dev", &self.is_dev)
            .field("closed_beta", &self.closed_beta)
            .field("cookie_secure", &self.cookie_secure)
            .field("session_secret", &"<redacted>")
            .field("admin_emails", &self.admin_emails.len())
            .field(
                "bootstrap_invite",
                &self.bootstrap_invite.as_ref().map(|(_, n)| n),
            )
            .field("rate_limit_multiplier", &self.rate_limit_multiplier)
            .field("trust_forwarded_for", &self.trust_forwarded_for)
            .field("support_url", &self.support_url)
            .finish()
    }
}

impl SecurityConfig {
    /// Defaults for tests and tools that never read the environment: dev
    /// mode, gate on, fixed secret.
    pub fn for_tests() -> Self {
        Self {
            is_dev: true,
            closed_beta: true,
            cookie_secure: false,
            session_secret: b"test-secret-test-secret-test-secret!".to_vec(),
            admin_emails: Vec::new(),
            bootstrap_invite: None,
            rate_limit_multiplier: 1,
            trust_forwarded_for: false,
            support_url: None,
        }
    }

    pub fn is_admin_email(&self, email: &str) -> bool {
        let email = email.trim().to_lowercase();
        self.admin_emails.contains(&email)
    }

    /// Whether a user is an admin right now: the stored flag is only honoured
    /// while the email is still listed in `ADMIN_EMAILS`.
    pub fn grants_admin(&self, stored_flag: bool, email: &str) -> bool {
        stored_flag && self.is_admin_email(email)
    }
}

/// One environment variable: what it is called, its default and what it
/// does. `server` is false for variables only the test tooling reads (they
/// are listed in the `CLAUDE.md` table too).
pub struct EnvVar {
    pub name: &'static str,
    pub default: &'static str,
    pub meaning: &'static str,
    pub server: bool,
}

/// Every variable, in the order of the `CLAUDE.md` table. A test keeps the
/// two in sync, and another checks that [`AppConfig::from_lookup`] reads
/// exactly the `server` ones.
pub const VARIABLES: &[EnvVar] = &[
    EnvVar { name: "RIOT_API_KEY", default: "none", meaning: "Riot API key; without it Riot features report that they are off", server: true },
    EnvVar { name: "RIOT_GLOBAL_BUDGET_PER_SEC", default: "20", meaning: "process-wide Riot requests per second (1 to 100,000)", server: true },
    EnvVar { name: "RIOT_GLOBAL_BUDGET_PER_MIN", default: "50", meaning: "process-wide Riot requests per minute, two-minute burst (1 to 1,000,000)", server: true },
    EnvVar { name: "SURREAL_DATA_DIR", default: "./data", meaning: "database directory (SurrealKV, one process only)", server: true },
    EnvVar { name: "BUG_REPORT_INBOX_PATH", default: "./docs/inbox/bug-reports.md", meaning: "bug-report export written on every start", server: true },
    EnvVar { name: "RUST_LOG", default: "info,lol_team_companion=debug", meaning: "tracing filter (an invalid filter is reported and the default used)", server: true },
    EnvVar { name: "APP_ENV", default: "unset (production)", meaning: "dev, development or local marks a development run (so does LEPTOS_ENV=DEV)", server: true },
    EnvVar { name: "LEPTOS_ENV", default: "unset (production)", meaning: "DEV marks a development run (set by cargo leptos watch); the image sets PROD", server: true },
    EnvVar { name: "SESSION_SECRET", default: "none; random per run in dev", meaning: "HMAC key for the session auth hash, at least 32 bytes and 8 distinct characters; required outside dev", server: true },
    EnvVar { name: "CLOSED_BETA", default: "true", meaning: "false disables the invite gate, honoured only in dev", server: true },
    EnvVar { name: "COOKIE_SECURE", default: "true outside dev", meaning: "Secure flag on the session cookie", server: true },
    EnvVar { name: "ADMIN_EMAILS", default: "empty", meaning: "comma-separated admin emails; removing one revokes admin", server: true },
    EnvVar { name: "BOOTSTRAP_INVITE_CODE", default: "unset", meaning: "invite code ensured at startup (first admin, e2e)", server: true },
    EnvVar { name: "BOOTSTRAP_INVITE_MAX_USES", default: "1", meaning: "uses of that code (1 to 1,000,000)", server: true },
    EnvVar { name: "RATE_LIMIT_MULTIPLIER", default: "1", meaning: "scales every rate-limit capacity (1 to 1,000; e2e uses 100)", server: true },
    EnvVar { name: "TRUST_FORWARDED_FOR", default: "false", meaning: "take the client IP from the last X-Forwarded-For hop (behind Caddy)", server: true },
    EnvVar { name: "SUPPORT_URL", default: "unset", meaning: "the external support page on /transparency, https only", server: true },
    EnvVar { name: "SHUTDOWN_DRAIN_SECS", default: "10", meaning: "how long requests in flight may finish after SIGTERM (0 to 300)", server: true },
    EnvVar { name: "SHUTDOWN_DB_SECS", default: "5", meaning: "how long to wait for the database to close before exiting (0 to 60)", server: true },
    EnvVar { name: "GITHUB_TOKEN", default: "unset", meaning: "not read by the app; warned about outside dev", server: true },
    EnvVar { name: "E2E_INVITE_CODE", default: "E2E-INVITE-CODE", meaning: "e2e: invite code the fixtures register with", server: false },
    EnvVar { name: "E2E_EXPECT_PRODUCTION", default: "unset", meaning: "e2e: the server under test runs in production mode", server: false },
    EnvVar { name: "BASE_URL", default: "http://127.0.0.1:3020", meaning: "e2e: server under test", server: false },
    EnvVar { name: "PLAYWRIGHT_BROWSERS_PATH", default: "Playwright default", meaning: "e2e: browser location", server: false },
    EnvVar { name: "AGENT_BROWSER_EXECUTABLE_PATH", default: "none", meaning: "agent-browser screenshots", server: false },
];

/// Everything the server reads from its environment, validated once at
/// startup by [`AppConfig::from_env`]. `security` is installed with
/// [`init`]; the rest is handed to `main`.
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub security: SecurityConfig,
    pub data_dir: PathBuf,
    pub bug_inbox_path: PathBuf,
    pub riot: RiotConfig,
    pub shutdown: ShutdownConfig,
}

#[derive(Clone)]
pub struct RiotConfig {
    pub api_key: Option<String>,
    pub budget_per_sec: u32,
    pub budget_per_min: u32,
}

impl std::fmt::Debug for RiotConfig {
    /// Never prints the key.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RiotConfig")
            .field("api_key", &self.api_key.as_ref().map(|_| "<set>"))
            .field("budget_per_sec", &self.budget_per_sec)
            .field("budget_per_min", &self.budget_per_min)
            .finish()
    }
}

/// The two bounded waits of a graceful shutdown (`main.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShutdownConfig {
    /// How long requests in flight may run after SIGTERM or SIGINT.
    pub drain: Duration,
    /// How long to wait for SurrealKV to close before exiting anyway.
    pub db_close: Duration,
}

pub const DEFAULT_DATA_DIR: &str = "./data";
pub const DEFAULT_RUST_LOG: &str = "info,lol_team_companion=debug";

/// Edit distance between two variable names (insertions, deletions and
/// substitutions), for spotting misspelt names.
fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != *cb);
            cur[j + 1] = sub.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The setting a set variable `name` probably meant: one of ours within two
/// edits (but not itself).
fn misspelt_setting(name: &str) -> Option<&'static str> {
    if VARIABLES.iter().any(|v| v.name == name) {
        return None;
    }
    VARIABLES
        .iter()
        .filter(|v| v.server && v.name.len() >= 6)
        .find(|v| edit_distance(name, v.name) <= 2)
        .map(|v| v.name)
}

/// Reads variables through `lookup`, records every problem (so one start
/// reports all mistakes) and every name it read (for the table test).
struct Reader<'a> {
    lookup: &'a dyn Fn(&str) -> Option<String>,
    errors: Vec<String>,
    warnings: Vec<String>,
    read: std::cell::RefCell<BTreeSet<&'static str>>,
}

impl Reader<'_> {
    fn raw(&self, name: &'static str) -> Option<String> {
        self.read.borrow_mut().insert(name);
        (self.lookup)(name)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    fn flag(&mut self, name: &'static str) -> Option<bool> {
        let value = self.raw(name)?;
        let parsed = parse_bool(&value);
        if parsed.is_none() {
            self.errors.push(format!(
                "{name}={value:?} is not a boolean (use true or false)"
            ));
        }
        parsed
    }

    fn number<T>(&mut self, name: &'static str, default: T, range: std::ops::RangeInclusive<T>) -> T
    where
        T: std::str::FromStr + PartialOrd + Copy + std::fmt::Display,
    {
        let Some(value) = self.raw(name) else {
            return default;
        };
        match value.parse::<T>() {
            Ok(n) if range.contains(&n) => n,
            _ => {
                self.errors.push(format!(
                    "{name}={value:?} must be a whole number from {} to {}",
                    range.start(),
                    range.end()
                ));
                default
            }
        }
    }

    fn is_set(&self, name: &'static str) -> bool {
        self.raw(name).is_some()
    }
}

fn distinct_bytes(s: &str) -> usize {
    let mut seen = [false; 256];
    s.bytes().for_each(|b| seen[b as usize] = true);
    seen.iter().filter(|b| **b).count()
}

fn plausible_email(e: &str) -> bool {
    let mut parts = e.split('@');
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(local), Some(domain), None)
            if !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
    )
}

impl AppConfig {
    /// Read and validate the process environment. `Err` lists every error;
    /// `Ok` carries the warnings to log.
    pub fn from_env() -> Result<(Self, Vec<String>), Vec<String>> {
        let names: Vec<String> = std::env::vars_os()
            .filter_map(|(k, _)| k.into_string().ok())
            .collect();
        Self::from_parts(&|name: &str| std::env::var(name).ok(), &names)
    }

    /// [`from_env`](Self::from_env) over any lookup (tests pass a map).
    pub fn from_lookup(
        lookup: &dyn Fn(&str) -> Option<String>,
    ) -> Result<(Self, Vec<String>), Vec<String>> {
        Self::from_parts(lookup, &[])
    }

    /// `names`: every variable that is set, to spot misspelt names.
    fn from_parts(
        lookup: &dyn Fn(&str) -> Option<String>,
        names: &[String],
    ) -> Result<(Self, Vec<String>), Vec<String>> {
        let (result, _) = Self::read_all(lookup, names);
        result
    }

    #[allow(clippy::type_complexity)]
    fn read_all(
        lookup: &dyn Fn(&str) -> Option<String>,
        names: &[String],
    ) -> (
        Result<(Self, Vec<String>), Vec<String>>,
        BTreeSet<&'static str>,
    ) {
        let mut r = Reader {
            lookup,
            errors: Vec::new(),
            warnings: Vec::new(),
            read: Default::default(),
        };

        // --- run mode -----------------------------------------------------
        let app_env = r.raw("APP_ENV");
        let leptos_env = r.raw("LEPTOS_ENV");
        let is_dev = is_dev_env(app_env.as_deref(), leptos_env.as_deref());
        if let Some(v) = &app_env {
            let known = matches!(
                v.to_ascii_lowercase().as_str(),
                "dev" | "development" | "local" | "prod" | "production"
            );
            if !known {
                r.warnings.push(format!(
                    "APP_ENV={v:?} is not a known value; running in production mode"
                ));
            }
        }
        if is_dev && app_env.is_none() && !cfg!(debug_assertions) {
            r.warnings.push(
                "LEPTOS_ENV marks this release build as a development run (no SESSION_SECRET \
                 requirement, insecure cookie); set LEPTOS_ENV=PROD for a deployment"
                    .into(),
            );
        }

        // --- security -------------------------------------------------------
        let closed_beta_var = r.flag("CLOSED_BETA");
        let closed_beta = closed_beta_enabled(is_dev, r.raw("CLOSED_BETA").as_deref());
        if !is_dev && closed_beta_var == Some(false) {
            r.warnings
                .push("CLOSED_BETA=false is ignored outside dev: the invite gate stays on".into());
        }

        let cookie_secure = r.flag("COOKIE_SECURE").unwrap_or(!is_dev);
        if !is_dev && !cookie_secure {
            r.warnings.push(
                "COOKIE_SECURE=false outside dev: the session cookie also travels over plain \
                 HTTP (acceptable only for local test runs)"
                    .into(),
            );
        }

        let session_secret = match r.raw("SESSION_SECRET") {
            Some(s) if s.len() >= 32 => {
                if distinct_bytes(&s) < 8 {
                    let msg = "SESSION_SECRET looks like a placeholder (fewer than 8 distinct \
                               characters); generate one with: openssl rand -hex 32";
                    if is_dev {
                        r.warnings.push(msg.into());
                    } else {
                        r.errors.push(msg.into());
                    }
                }
                s.into_bytes()
            }
            Some(_) if !is_dev => {
                r.errors
                    .push("SESSION_SECRET must be at least 32 bytes (openssl rand -hex 32)".into());
                Vec::new()
            }
            None if !is_dev => {
                r.errors.push(
                    "SESSION_SECRET is required outside dev (set APP_ENV=dev for local runs)"
                        .into(),
                );
                Vec::new()
            }
            _ => {
                use password_hash::rand_core::{OsRng, RngCore};
                let mut bytes = vec![0u8; 32];
                OsRng.fill_bytes(&mut bytes);
                r.warnings.push(
                    "SESSION_SECRET is not set (or too short); using a random secret for this \
                     dev run. Sessions will not survive a restart."
                        .into(),
                );
                bytes
            }
        };

        let admin_emails = parse_admin_emails(r.raw("ADMIN_EMAILS").as_deref());
        for e in admin_emails.iter().filter(|e| !plausible_email(e)) {
            r.errors.push(format!(
                "ADMIN_EMAILS contains {e:?}, which is not an email address"
            ));
        }
        if admin_emails.is_empty() && !is_dev {
            r.warnings
                .push("ADMIN_EMAILS is empty: nobody can open the admin pages".into());
        }

        let invite_code = r.raw("BOOTSTRAP_INVITE_CODE");
        let max_uses = r.number("BOOTSTRAP_INVITE_MAX_USES", 1i64, 1..=1_000_000);
        if invite_code.is_none() && r.is_set("BOOTSTRAP_INVITE_MAX_USES") {
            r.warnings.push(
                "BOOTSTRAP_INVITE_MAX_USES is set without BOOTSTRAP_INVITE_CODE; ignored".into(),
            );
        }
        if let (Some(code), false) = (&invite_code, is_dev) {
            if code.len() < 12 {
                r.warnings
                    .push("BOOTSTRAP_INVITE_CODE is shorter than 12 characters (guessable)".into());
            }
            if max_uses > 10 {
                r.warnings.push(format!(
                    "BOOTSTRAP_INVITE_MAX_USES={max_uses} outside dev (an e2e setting?); \
                     the bootstrap code is meant for the first admin"
                ));
            }
        }

        let rate_limit_multiplier = r.number("RATE_LIMIT_MULTIPLIER", 1u32, 1..=1000);
        if rate_limit_multiplier > 1 && !is_dev {
            r.warnings.push(format!(
                "RATE_LIMIT_MULTIPLIER={rate_limit_multiplier} outside dev: sign-in, \
                 registration and Riot limits are relaxed (an e2e setting)"
            ));
        }

        let trust_forwarded_for = r.flag("TRUST_FORWARDED_FOR").unwrap_or(false);

        let support_url = {
            let raw = r.raw("SUPPORT_URL");
            let url = parse_support_url(raw.as_deref());
            if let (Some(v), None) = (&raw, &url) {
                r.errors.push(format!(
                    "SUPPORT_URL={v:?} must be an absolute https:// URL without credentials"
                ));
            }
            url
        };

        // --- paths ------------------------------------------------------------
        let data_dir = PathBuf::from(
            r.raw("SURREAL_DATA_DIR")
                .unwrap_or_else(|| DEFAULT_DATA_DIR.into()),
        );
        if data_dir.exists() && !data_dir.is_dir() {
            r.errors.push(format!(
                "SURREAL_DATA_DIR={} exists but is not a directory",
                data_dir.display()
            ));
        }
        let bug_inbox_path = PathBuf::from(
            r.raw("BUG_REPORT_INBOX_PATH")
                .unwrap_or_else(|| crate::server::bug_report_export::DEFAULT_INBOX_PATH.into()),
        );
        if bug_inbox_path.is_dir() {
            r.errors.push(format!(
                "BUG_REPORT_INBOX_PATH={} is a directory; it must name a file",
                bug_inbox_path.display()
            ));
        }
        // Read here for the table test; main.rs builds the filter itself.
        let _ = r.raw("RUST_LOG");

        // --- Riot ---------------------------------------------------------------
        use crate::server::rate_limit::RiotBudget;
        let api_key = r.raw("RIOT_API_KEY");
        match &api_key {
            None => r.warnings.push(
                "RIOT_API_KEY is not set: Riot features will report that they are off".into(),
            ),
            Some(k) if !k.starts_with("RGAPI-") => r
                .warnings
                .push("RIOT_API_KEY does not start with RGAPI- (is it a Riot key?)".into()),
            Some(_) => {}
        }
        let budget_per_sec = r.number(
            "RIOT_GLOBAL_BUDGET_PER_SEC",
            RiotBudget::DEFAULT_PER_SEC,
            1..=100_000,
        );
        let budget_per_min = r.number(
            "RIOT_GLOBAL_BUDGET_PER_MIN",
            RiotBudget::DEFAULT_PER_MIN,
            1..=1_000_000,
        );
        if !is_dev
            && api_key.is_some()
            && !r.is_set("RIOT_GLOBAL_BUDGET_PER_SEC")
            && !r.is_set("RIOT_GLOBAL_BUDGET_PER_MIN")
        {
            r.warnings.push(
                "Riot budget at development-key defaults (20/s, 50/min); raise \
                 RIOT_GLOBAL_BUDGET_PER_SEC and _PER_MIN for a production key"
                    .into(),
            );
        }

        // --- shutdown -------------------------------------------------------------
        let shutdown = ShutdownConfig {
            drain: Duration::from_secs(r.number("SHUTDOWN_DRAIN_SECS", 10u64, 0..=300)),
            db_close: Duration::from_secs(r.number("SHUTDOWN_DB_SECS", 5u64, 0..=60)),
        };

        // --- strays ---------------------------------------------------------------
        if r.is_set("GITHUB_TOKEN") && !is_dev {
            r.warnings.push(
                "GITHUB_TOKEN is set but the app never reads it; keep it out of the \
                 production environment"
                    .into(),
            );
        }
        for name in names {
            if let Some(meant) = misspelt_setting(name) {
                r.warnings.push(format!(
                    "{name} is set but is not a Team Companion setting; did you mean {meant}?"
                ));
            }
        }

        let read = r.read.take();
        if !r.errors.is_empty() {
            return (Err(r.errors), read);
        }
        let security = SecurityConfig {
            is_dev,
            closed_beta,
            cookie_secure,
            session_secret,
            admin_emails,
            bootstrap_invite: invite_code.map(|code| (code, max_uses)),
            rate_limit_multiplier,
            trust_forwarded_for,
            support_url,
        };
        (
            Ok((
                AppConfig {
                    security,
                    data_dir,
                    bug_inbox_path,
                    riot: RiotConfig {
                        api_key,
                        budget_per_sec,
                        budget_per_min,
                    },
                    shutdown,
                },
                r.warnings,
            )),
            read,
        )
    }
}

/// Install the configuration. Call once at startup, before anything reads
/// it. Panics when a configuration is already installed: that means `get()`
/// ran first and silently installed the test defaults (dev mode, public
/// secret), which must never serve traffic.
pub fn init(config: SecurityConfig) {
    if CONFIG.set(config).is_err() {
        panic!("security configuration was read before config::init; refusing to start");
    }
}

/// The installed configuration. Falls back to test defaults when `init` was
/// never called (unit and integration tests, the seed tool).
pub fn get() -> &'static SecurityConfig {
    CONFIG.get_or_init(SecurityConfig::for_tests)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: std::collections::HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| map.get(k).cloned()
    }

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn production_without_secret_fails() {
        let errors = AppConfig::from_lookup(&lookup(&[])).unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.contains("SESSION_SECRET is required")));
    }

    #[test]
    fn every_error_is_reported_at_once() {
        let errors = AppConfig::from_lookup(&lookup(&[
            ("TRUST_FORWARDED_FOR", "ture"),
            ("RATE_LIMIT_MULTIPLIER", "1O"),
            ("ADMIN_EMAILS", "admin.example.com"),
            ("SHUTDOWN_DRAIN_SECS", "-1"),
        ]))
        .unwrap_err();
        // four typos + the missing secret
        assert_eq!(errors.len(), 5, "{errors:?}");
        assert!(errors[0].contains("SESSION_SECRET"), "{errors:?}");
        assert!(errors
            .iter()
            .any(|e| e.contains("TRUST_FORWARDED_FOR=\"ture\"")));
    }

    #[test]
    fn typos_are_errors_in_dev_too() {
        let errors = AppConfig::from_lookup(&lookup(&[
            ("APP_ENV", "dev"),
            ("COOKIE_SECURE", "flase"),
            ("RIOT_GLOBAL_BUDGET_PER_SEC", "45/s"),
            ("SUPPORT_URL", "http://ko-fi.com/x"),
        ]))
        .unwrap_err();
        assert_eq!(errors.len(), 3, "{errors:?}");
    }

    #[test]
    fn placeholder_secret_is_refused_outside_dev() {
        let errors =
            AppConfig::from_lookup(&lookup(&[("SESSION_SECRET", &"a".repeat(40))])).unwrap_err();
        assert!(errors[0].contains("placeholder"), "{errors:?}");
    }

    #[test]
    fn e2e_settings_in_production_warn_but_start() {
        // The CI e2e environment: production mode on plain http.
        let (cfg, warnings) = AppConfig::from_lookup(&lookup(&[
            ("SESSION_SECRET", "ci-only-session-secret-0123456789abcdef"),
            ("RATE_LIMIT_MULTIPLIER", "100"),
            ("BOOTSTRAP_INVITE_CODE", "E2E-INVITE-CODE"),
            ("BOOTSTRAP_INVITE_MAX_USES", "100000"),
            ("COOKIE_SECURE", "false"),
            ("ADMIN_EMAILS", "e2e-admin@test.invalid"),
            ("RIOT_API_KEY", "RGAPI-ci-dummy"),
        ]))
        .unwrap();
        assert!(!cfg.security.is_dev);
        assert_eq!(cfg.security.rate_limit_multiplier, 100);
        assert_eq!(
            cfg.security.bootstrap_invite,
            Some(("E2E-INVITE-CODE".into(), 100_000))
        );
        for needle in [
            "RATE_LIMIT_MULTIPLIER",
            "BOOTSTRAP_INVITE_MAX_USES",
            "COOKIE_SECURE=false",
            "development-key defaults",
        ] {
            assert!(
                warnings.iter().any(|w| w.contains(needle)),
                "{needle}: {warnings:?}"
            );
        }
    }

    #[test]
    fn dev_run_needs_nothing() {
        let (cfg, _) = AppConfig::from_lookup(&lookup(&[("APP_ENV", "dev")])).unwrap();
        assert!(cfg.security.is_dev);
        assert!(!cfg.security.cookie_secure);
        assert_eq!(cfg.security.session_secret.len(), 32);
        assert_eq!(cfg.data_dir, PathBuf::from("./data"));
        assert_eq!(
            cfg.shutdown,
            ShutdownConfig {
                drain: Duration::from_secs(10),
                db_close: Duration::from_secs(5)
            }
        );
        assert_eq!(cfg.riot.budget_per_sec, 20);
    }

    #[test]
    fn values_are_parsed() {
        let (cfg, _) = AppConfig::from_lookup(&lookup(&[
            ("SESSION_SECRET", SECRET),
            ("TRUST_FORWARDED_FOR", "true"),
            ("RIOT_GLOBAL_BUDGET_PER_SEC", "45"),
            ("RIOT_GLOBAL_BUDGET_PER_MIN", "2400"),
            ("SHUTDOWN_DRAIN_SECS", "2"),
            ("SURREAL_DATA_DIR", "/data/db"),
            ("SUPPORT_URL", "https://ko-fi.com/companion"),
        ]))
        .unwrap();
        assert!(cfg.security.trust_forwarded_for);
        assert_eq!(
            (cfg.riot.budget_per_sec, cfg.riot.budget_per_min),
            (45, 2400)
        );
        assert_eq!(cfg.shutdown.drain, Duration::from_secs(2));
        assert_eq!(cfg.data_dir, PathBuf::from("/data/db"));
        assert!(cfg.security.support_url.is_some());
    }

    #[test]
    fn misspelt_names_warn() {
        let names: Vec<String> = [
            "TRUST_FORWARD_FOR",
            "SESION_SECRET",
            "PATH",
            "SESSION_INGRESS_URL",
            "TRUST_FORWARDED_FOR",
        ]
        .map(String::from)
        .to_vec();
        let (_, warnings) = AppConfig::from_parts(&lookup(&[("APP_ENV", "dev")]), &names).unwrap();
        let hits: Vec<&String> = warnings
            .iter()
            .filter(|w| w.contains("did you mean"))
            .collect();
        assert_eq!(hits.len(), 2, "{warnings:?}");
        assert!(hits
            .iter()
            .any(|w| w.contains("TRUST_FORWARD_FOR") && w.contains("TRUST_FORWARDED_FOR?")));
        assert!(hits.iter().any(|w| w.contains("SESION_SECRET")));
    }

    #[test]
    fn secrets_are_not_in_debug_output() {
        let (cfg, _) = AppConfig::from_lookup(&lookup(&[
            ("SESSION_SECRET", SECRET),
            ("RIOT_API_KEY", "RGAPI-secret-value"),
            ("ADMIN_EMAILS", "boss@example.com"),
        ]))
        .unwrap();
        let dbg = format!("{cfg:?}");
        for secret in ["0123456789abcdef", "secret-value", "boss@example.com"] {
            assert!(!dbg.contains(secret), "{dbg}");
        }
    }

    /// The server reads exactly the `server` variables of [`VARIABLES`].
    #[test]
    fn the_table_lists_what_is_read() {
        let (_, read) = AppConfig::read_all(&lookup(&[]), &[]);
        let listed: BTreeSet<&str> = VARIABLES
            .iter()
            .filter(|v| v.server)
            .map(|v| v.name)
            .collect();
        assert_eq!(read, listed);
    }

    /// The environment table in CLAUDE.md names exactly the variables of
    /// [`VARIABLES`]: add a variable to both or neither.
    #[test]
    fn claude_md_env_table_matches() {
        let doc = include_str!("../../CLAUDE.md");
        let section = doc
            .split("### Environment variables")
            .nth(1)
            .expect("CLAUDE.md has an environment table")
            .split("\n#")
            .next()
            .unwrap_or_default();
        let in_doc: BTreeSet<&str> = section
            .lines()
            .filter_map(|l| l.strip_prefix("| `"))
            .flat_map(|l| {
                let cell = l.split(" |").next().unwrap_or_default();
                cell.split(" / ")
                    .map(|n| n.trim_matches('`'))
                    .collect::<Vec<_>>()
            })
            .collect();
        let listed: BTreeSet<&str> = VARIABLES.iter().map(|v| v.name).collect();
        assert_eq!(
            in_doc, listed,
            "CLAUDE.md environment table vs config::VARIABLES"
        );
    }

    #[test]
    fn support_url_must_be_plain_https() {
        assert_eq!(
            parse_support_url(Some(" https://ko-fi.com/companion ")),
            Some("https://ko-fi.com/companion".into())
        );
        assert!(parse_support_url(Some("https://github.com/sponsors/x?tab=1")).is_some());
        assert!(parse_support_url(Some("http://ko-fi.com/x")).is_none());
        assert!(parse_support_url(Some("https://user:pw@ko-fi.com/x")).is_none());
        assert!(parse_support_url(Some("https://")).is_none());
        assert!(parse_support_url(Some("javascript:alert(1)")).is_none());
        assert!(parse_support_url(Some("https://ko-fi.com/a b")).is_none());
        assert!(parse_support_url(Some("https://ko-fi.com/\"x")).is_none());
        assert!(parse_support_url(None).is_none());
    }

    #[test]
    fn dev_detection() {
        assert!(is_dev_env(Some("dev"), None));
        assert!(is_dev_env(None, Some("DEV")));
        assert!(is_dev_env(Some("Development"), None));
        assert!(!is_dev_env(None, None));
        assert!(!is_dev_env(Some("prod"), Some("PROD")));
    }

    #[test]
    fn gate_is_on_outside_dev_whatever_the_variable_says() {
        assert!(closed_beta_enabled(false, None));
        assert!(closed_beta_enabled(false, Some("false")));
        assert!(closed_beta_enabled(false, Some("0")));
    }

    #[test]
    fn gate_can_be_turned_off_only_in_dev() {
        assert!(closed_beta_enabled(true, None));
        assert!(closed_beta_enabled(true, Some("true")));
        assert!(closed_beta_enabled(true, Some("garbage")));
        assert!(!closed_beta_enabled(true, Some("false")));
        assert!(!closed_beta_enabled(true, Some("off")));
    }

    #[test]
    fn admin_emails_are_normalised() {
        let list = parse_admin_emails(Some(" Admin@Example.com, other@x.io  third@y.de,"));
        assert_eq!(list, vec!["admin@example.com", "other@x.io", "third@y.de"]);
        assert!(parse_admin_emails(None).is_empty());
        let mut cfg = SecurityConfig::for_tests();
        cfg.admin_emails = list;
        assert!(cfg.is_admin_email("ADMIN@example.com "));
        assert!(!cfg.is_admin_email("nobody@example.com"));
    }

    #[test]
    fn admin_requires_flag_and_current_listing() {
        let mut cfg = SecurityConfig::for_tests();
        cfg.admin_emails = vec!["admin@example.com".into()];
        assert!(cfg.grants_admin(true, "Admin@Example.com"));
        assert!(
            !cfg.grants_admin(false, "admin@example.com"),
            "not yet promoted"
        );
        // Removed from ADMIN_EMAILS: the stale stored flag no longer counts.
        cfg.admin_emails.clear();
        assert!(!cfg.grants_admin(true, "admin@example.com"));
    }
}
