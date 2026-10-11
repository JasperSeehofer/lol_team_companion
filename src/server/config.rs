//! Security-relevant runtime configuration, read once from the environment.
//!
//! | Variable | Default | Meaning |
//! |----------|---------|---------|
//! | `APP_ENV` | unset | `dev`/`development`/`local` marks a development run (as does `LEPTOS_ENV=DEV`, which `cargo leptos watch` sets) |
//! | `CLOSED_BETA` | on | `false`/`0`/`off` disables the invite gate, **only honoured in dev** |
//! | `SESSION_SECRET` | none | key for the session auth hash (HMAC); required outside dev, at least 32 bytes |
//! | `COOKIE_SECURE` | `true` outside dev, `false` in dev | `Secure` flag on the session cookie |
//! | `ADMIN_EMAILS` | empty | comma-separated emails promoted to admin when they sign in (no email verification exists: list an address only after its owner has registered it) |
//! | `BOOTSTRAP_INVITE_CODE` | unset | ensures this invite code exists at startup |
//! | `BOOTSTRAP_INVITE_MAX_USES` | `1` | max uses for that code when it is created |
//! | `RATE_LIMIT_MULTIPLIER` | `1` | scales every rate-limit capacity (e.g. `100` for e2e runs) |
//! | `TRUST_FORWARDED_FOR` | `false` | take the client IP from the last `X-Forwarded-For` hop (set behind a reverse proxy) |

use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct SecurityConfig {
    pub is_dev: bool,
    pub closed_beta: bool,
    pub cookie_secure: bool,
    pub session_secret: Vec<u8>,
    pub admin_emails: Vec<String>,
    pub bootstrap_invite: Option<(String, i64)>,
    pub rate_limit_multiplier: u32,
    pub trust_forwarded_for: bool,
}

static CONFIG: OnceLock<SecurityConfig> = OnceLock::new();

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

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

/// Parse `ADMIN_EMAILS`: comma or whitespace separated, normalised.
pub fn parse_admin_emails(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(|e| e.trim().to_lowercase())
        .filter(|e| !e.is_empty())
        .collect()
}

impl SecurityConfig {
    /// Read the configuration from the environment. Fails outside dev when
    /// `SESSION_SECRET` is missing or shorter than 32 bytes.
    pub fn from_env() -> Result<Self, String> {
        let is_dev = is_dev_env(env("APP_ENV").as_deref(), env("LEPTOS_ENV").as_deref());
        let closed_beta = closed_beta_enabled(is_dev, env("CLOSED_BETA").as_deref());
        let cookie_secure = env("COOKIE_SECURE")
            .as_deref()
            .and_then(parse_bool)
            .unwrap_or(!is_dev);

        let session_secret = match env("SESSION_SECRET") {
            Some(s) if s.len() >= 32 => s.into_bytes(),
            Some(_) if !is_dev => {
                return Err("SESSION_SECRET must be at least 32 bytes".into());
            }
            None if !is_dev => {
                return Err(
                    "SESSION_SECRET is required outside dev (set APP_ENV=dev for local runs)"
                        .into(),
                );
            }
            _ => {
                use password_hash::rand_core::{OsRng, RngCore};
                let mut bytes = vec![0u8; 32];
                OsRng.fill_bytes(&mut bytes);
                tracing::warn!(
                    "SESSION_SECRET is not set (or too short); using a random secret for this \
                     dev run. Sessions will not survive a restart."
                );
                bytes
            }
        };

        let bootstrap_invite = env("BOOTSTRAP_INVITE_CODE").map(|code| {
            let max_uses = env("BOOTSTRAP_INVITE_MAX_USES")
                .and_then(|v| v.parse::<i64>().ok())
                .filter(|n| *n >= 1)
                .unwrap_or(1);
            (code, max_uses)
        });

        let rate_limit_multiplier = env("RATE_LIMIT_MULTIPLIER")
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|n| *n >= 1)
            .unwrap_or(1);

        Ok(Self {
            is_dev,
            closed_beta,
            cookie_secure,
            session_secret,
            admin_emails: parse_admin_emails(env("ADMIN_EMAILS").as_deref()),
            bootstrap_invite,
            rate_limit_multiplier,
            trust_forwarded_for: env("TRUST_FORWARDED_FOR")
                .as_deref()
                .and_then(parse_bool)
                .unwrap_or(false),
        })
    }

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
        }
    }

    pub fn is_admin_email(&self, email: &str) -> bool {
        let email = email.trim().to_lowercase();
        self.admin_emails.contains(&email)
    }
}

/// Install the configuration. Call once at startup, before serving.
pub fn init(config: SecurityConfig) {
    let _ = CONFIG.set(config);
}

/// The installed configuration. Falls back to test defaults when `init` was
/// never called (unit and integration tests, the seed tool).
pub fn get() -> &'static SecurityConfig {
    CONFIG.get_or_init(SecurityConfig::for_tests)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
