//! Small in-memory rate limiter (token buckets keyed by bucket + client) and
//! per-user cooldowns. One process, one map: fine for the single-instance
//! deployment (embedded SurrealKV already pins us to one instance).
//!
//! Capacities scale with `RATE_LIMIT_MULTIPLIER` (see `config`), so e2e runs
//! that register hundreds of users from 127.0.0.1 can raise them.
//!
//! Idle entries are dropped by [`Limiter::purge`], which `main` runs on a
//! timer (never on the request path). [`RiotBudget`] is the separate,
//! process-wide budget of Riot API requests.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bucket {
    /// Login attempts per client IP.
    Login,
    /// Failed logins per account email (checked before, counted after a failure).
    LoginFailuresPerEmail,
    /// Registrations per client IP.
    Register,
    /// Requests that call the Riot API, per client IP.
    Riot,
    /// Image requests that miss the server cache and go to Data Dragon,
    /// per client IP (`/img/...` is public).
    ImageFetch,
    /// Draft Graph rankings, per client IP (the page asks again after
    /// each draft change, debounced).
    Graph,
    /// Share-link lookups and share-link changes, per client IP (the
    /// public `/s/{token}` page needs no sign-in).
    Share,
    /// Bug reports and wishes, per user (M7).
    Feedback,
    /// Votes and survey answers, dismissals and the survey switch, per user (M7).
    Community,
    /// Public community reads (changelog, transparency), per client IP (M7).
    PublicRead,
    /// The survey band's check on every route change, per user (M7 review).
    /// Over the limit the band simply shows nothing.
    SurveyCheck,
}

#[derive(Clone, Copy, Debug)]
struct Limit {
    /// Burst size.
    capacity: f64,
    /// Tokens added per second.
    refill_per_sec: f64,
}

fn limit(bucket: Bucket) -> Limit {
    match bucket {
        // 10 attempts, then one every 30 s.
        Bucket::Login => Limit {
            capacity: 10.0,
            refill_per_sec: 1.0 / 30.0,
        },
        // 10 failures per account, then one every 90 s (10 per 15 min).
        Bucket::LoginFailuresPerEmail => Limit {
            capacity: 10.0,
            refill_per_sec: 1.0 / 90.0,
        },
        // 5 registrations, then one every 12 min (5 per hour).
        Bucket::Register => Limit {
            capacity: 5.0,
            refill_per_sec: 1.0 / 720.0,
        },
        // 30 Riot-backed requests, then one every 2 s.
        Bucket::Riot => Limit {
            capacity: 30.0,
            refill_per_sec: 0.5,
        },
        // A cold cache on the draft page needs about 170 icons at once;
        // after that a client may start 5 upstream fetches a second.
        Bucket::ImageFetch => Limit {
            capacity: 400.0,
            refill_per_sec: 5.0,
        },
        // 60 rankings, then two a second.
        Bucket::Graph => Limit {
            capacity: 60.0,
            refill_per_sec: 2.0,
        },
        // 30 lookups, then one every 2 s: enough to open and reload a
        // shared page, far too slow to guess tokens.
        Bucket::Share => Limit {
            capacity: 30.0,
            refill_per_sec: 0.5,
        },
        // 5 reports, then one every 2 minutes.
        Bucket::Feedback => Limit {
            capacity: 5.0,
            refill_per_sec: 1.0 / 120.0,
        },
        // 30 votes or answers, then one every 2 s.
        Bucket::Community => Limit {
            capacity: 30.0,
            refill_per_sec: 0.5,
        },
        // 30 route changes, then one every 2 s.
        Bucket::SurveyCheck => Limit {
            capacity: 30.0,
            refill_per_sec: 0.5,
        },
        // 120 page loads, then two a second.
        Bucket::PublicRead => Limit {
            capacity: 120.0,
            refill_per_sec: 2.0,
        },
    }
}

#[derive(Clone, Copy, Debug)]
struct State {
    tokens: f64,
    last: Instant,
}

/// Token-bucket store. `Limiter::global()` is the process-wide instance; tests
/// build their own.
pub struct Limiter {
    multiplier: f64,
    buckets: Mutex<HashMap<(Bucket, String), State>>,
    cooldowns: Mutex<HashMap<(&'static str, String), Instant>>,
}

/// Cooldown entries older than this are dropped by [`Limiter::purge`]
/// (every cooldown period in use is far shorter).
const COOLDOWN_RETENTION: Duration = Duration::from_secs(3600);

impl Limiter {
    pub fn new(multiplier: u32) -> Self {
        Self {
            multiplier: multiplier.max(1) as f64,
            buckets: Mutex::new(HashMap::new()),
            cooldowns: Mutex::new(HashMap::new()),
        }
    }

    pub fn global() -> &'static Limiter {
        static GLOBAL: std::sync::OnceLock<Limiter> = std::sync::OnceLock::new();
        GLOBAL.get_or_init(|| Limiter::new(crate::server::config::get().rate_limit_multiplier))
    }

    fn refreshed(&self, bucket: Bucket, state: Option<State>, now: Instant) -> State {
        let l = limit(bucket);
        let capacity = l.capacity * self.multiplier;
        match state {
            None => State {
                tokens: capacity,
                last: now,
            },
            Some(s) => {
                let elapsed = now.saturating_duration_since(s.last).as_secs_f64();
                State {
                    tokens: (s.tokens + elapsed * l.refill_per_sec * self.multiplier).min(capacity),
                    last: now,
                }
            }
        }
    }

    fn wait_for_token(&self, bucket: Bucket, tokens: f64) -> Duration {
        let rate = limit(bucket).refill_per_sec * self.multiplier;
        Duration::from_secs_f64(((1.0 - tokens) / rate).max(1.0))
    }

    /// Take one token. `Err(wait)` when the bucket is empty.
    pub fn hit_at(&self, bucket: Bucket, key: &str, now: Instant) -> Result<(), Duration> {
        let mut map = self.buckets.lock().unwrap_or_else(|p| p.into_inner());
        let entry_key = (bucket, key.to_string());
        let mut state = self.refreshed(bucket, map.get(&entry_key).copied(), now);
        let result = if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            Ok(())
        } else {
            Err(self.wait_for_token(bucket, state.tokens))
        };
        map.insert(entry_key, state);
        result
    }

    /// Like `hit_at` but without taking a token.
    pub fn check_at(&self, bucket: Bucket, key: &str, now: Instant) -> Result<(), Duration> {
        let map = self.buckets.lock().unwrap_or_else(|p| p.into_inner());
        let state = self.refreshed(bucket, map.get(&(bucket, key.to_string())).copied(), now);
        if state.tokens >= 1.0 {
            Ok(())
        } else {
            Err(self.wait_for_token(bucket, state.tokens))
        }
    }

    pub fn hit(&self, bucket: Bucket, key: &str) -> Result<(), Duration> {
        self.hit_at(bucket, key, Instant::now())
    }

    pub fn check(&self, bucket: Bucket, key: &str) -> Result<(), Duration> {
        self.check_at(bucket, key, Instant::now())
    }

    /// Allow `name` for `key` at most once per `period`. Records the attempt
    /// when allowed; `Err(remaining)` otherwise.
    pub fn cooldown_at(
        &self,
        name: &'static str,
        key: &str,
        period: Duration,
        now: Instant,
    ) -> Result<(), Duration> {
        let mut map = self.cooldowns.lock().unwrap_or_else(|p| p.into_inner());
        let entry_key = (name, key.to_string());
        if let Some(last) = map.get(&entry_key) {
            let elapsed = now.saturating_duration_since(*last);
            if elapsed < period {
                return Err(period - elapsed);
            }
        }
        map.insert(entry_key, now);
        Ok(())
    }

    pub fn cooldown(
        &self,
        name: &'static str,
        key: &str,
        period: Duration,
    ) -> Result<(), Duration> {
        self.cooldown_at(name, key, period, Instant::now())
    }

    /// Drop entries that no longer limit anyone: buckets that have refilled
    /// to capacity (a missing entry means a full bucket, so this changes no
    /// decision) and cooldowns older than an hour. Returns how many entries
    /// were removed. Runs from a timer in `main` (see [`spawn_purge_task`]).
    pub fn purge_at(&self, now: Instant) -> usize {
        let mut removed = 0;
        {
            let mut map = self.buckets.lock().unwrap_or_else(|p| p.into_inner());
            let before = map.len();
            map.retain(|(bucket, _), state| {
                let capacity = limit(*bucket).capacity * self.multiplier;
                self.refreshed(*bucket, Some(*state), now).tokens < capacity
            });
            removed += before - map.len();
        }
        {
            let mut map = self.cooldowns.lock().unwrap_or_else(|p| p.into_inner());
            let before = map.len();
            map.retain(|_, t| now.saturating_duration_since(*t) < COOLDOWN_RETENTION);
            removed += before - map.len();
        }
        removed
    }

    pub fn purge(&self) -> usize {
        self.purge_at(Instant::now())
    }

    /// Number of live entries (buckets + cooldowns), for tests and logs.
    pub fn len(&self) -> usize {
        let b = self.buckets.lock().unwrap_or_else(|p| p.into_inner()).len();
        let c = self
            .cooldowns
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .len();
        b + c
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Purge [`Limiter::global`] every `every` on a background task.
pub fn spawn_purge_task(every: Duration) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(every);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            let removed = Limiter::global().purge();
            if removed > 0 {
                tracing::debug!(removed, "rate limiter purge");
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Process-wide Riot API budget
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Window {
    capacity: f64,
    refill_per_sec: f64,
    tokens: f64,
    last: Instant,
}

impl Window {
    fn new(capacity: f64, refill_per_sec: f64, now: Instant) -> Self {
        Self {
            capacity,
            refill_per_sec,
            tokens: capacity,
            last: now,
        }
    }

    fn refresh(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        self.last = now;
    }

    /// Time until `n` tokens are available (zero when they already are).
    fn wait_for(&self, n: f64) -> Duration {
        if self.tokens >= n {
            Duration::ZERO
        } else {
            Duration::from_secs_f64((n - self.tokens) / self.refill_per_sec)
        }
    }
}

/// One budget of Riot API requests for the whole process, on top of the
/// per-IP `Bucket::Riot`, so a few heavy users cannot use up the app's key.
///
/// Two token buckets mirror Riot's two application rate limits: a short one
/// (default 20 requests per second) and a long one (default 50 per minute,
/// with a burst of two minutes' worth: the development key's 100 requests per
/// 2 minutes). A request needs a token from both. Configure with
/// `RIOT_GLOBAL_BUDGET_PER_SEC` and `RIOT_GLOBAL_BUDGET_PER_MIN` (a
/// production key allows far more).
pub struct RiotBudget {
    windows: Mutex<[Window; 2]>,
}

impl RiotBudget {
    pub const DEFAULT_PER_SEC: u32 = 20;
    pub const DEFAULT_PER_MIN: u32 = 50;

    pub fn new(per_sec: u32, per_min: u32) -> Self {
        Self::new_at(per_sec, per_min, Instant::now())
    }

    pub fn new_at(per_sec: u32, per_min: u32, now: Instant) -> Self {
        let per_sec = per_sec.max(1) as f64;
        let per_min = per_min.max(1) as f64;
        Self {
            windows: Mutex::new([
                Window::new(per_sec, per_sec, now),
                Window::new(per_min * 2.0, per_min / 60.0, now),
            ]),
        }
    }

    /// Read `RIOT_GLOBAL_BUDGET_PER_SEC` / `RIOT_GLOBAL_BUDGET_PER_MIN`.
    pub fn from_env() -> Self {
        let read = |name: &str, default: u32| {
            std::env::var(name)
                .ok()
                .and_then(|v| v.trim().parse::<u32>().ok())
                .filter(|n| *n >= 1)
                .unwrap_or(default)
        };
        Self::new(
            read("RIOT_GLOBAL_BUDGET_PER_SEC", Self::DEFAULT_PER_SEC),
            read("RIOT_GLOBAL_BUDGET_PER_MIN", Self::DEFAULT_PER_MIN),
        )
    }

    /// The most requests the budget can ever hold at once.
    pub fn max_burst(&self) -> u32 {
        let w = self.windows.lock().unwrap_or_else(|p| p.into_inner());
        w[1].capacity as u32
    }

    /// Take one request token. `Err(wait)` when the budget is spent.
    pub fn try_take_at(&self, now: Instant) -> Result<(), Duration> {
        let mut w = self.windows.lock().unwrap_or_else(|p| p.into_inner());
        for win in w.iter_mut() {
            win.refresh(now);
        }
        let wait = w
            .iter()
            .map(|win| win.wait_for(1.0))
            .max()
            .unwrap_or_default();
        if wait.is_zero() {
            for win in w.iter_mut() {
                win.tokens -= 1.0;
            }
            Ok(())
        } else {
            Err(wait)
        }
    }

    pub fn try_take(&self) -> Result<(), Duration> {
        self.try_take_at(Instant::now())
    }

    /// Whether `requests` tokens are available in the long window right now
    /// (capped at its burst, so a large job can still start on an idle
    /// budget), without taking any. `Err(wait)` until they are. Used before a
    /// multi-request job so it fails fast instead of stalling half-way.
    pub fn headroom_at(&self, requests: u32, now: Instant) -> Result<(), Duration> {
        let mut w = self.windows.lock().unwrap_or_else(|p| p.into_inner());
        w[1].refresh(now);
        let need = (requests as f64).min(w[1].capacity);
        let wait = w[1].wait_for(need);
        if wait.is_zero() {
            Ok(())
        } else {
            Err(wait)
        }
    }

    pub fn headroom(&self, requests: u32) -> Result<(), Duration> {
        self.headroom_at(requests, Instant::now())
    }
}

/// User-facing message for a limited request.
pub fn slow_down_message(wait: Duration) -> String {
    let secs = wait.as_secs().max(1);
    if secs < 120 {
        format!("Too many requests. Please slow down and try again in {secs} seconds.")
    } else {
        format!(
            "Too many requests. Please slow down and try again in {} minutes.",
            secs.div_ceil(60)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_allows_burst_then_limits_then_refills() {
        let l = Limiter::new(1);
        let t0 = Instant::now();
        for _ in 0..5 {
            assert!(l.hit_at(Bucket::Register, "1.2.3.4", t0).is_ok());
        }
        let wait = l.hit_at(Bucket::Register, "1.2.3.4", t0).unwrap_err();
        assert!(wait >= Duration::from_secs(700), "{wait:?}");
        // Other clients are unaffected.
        assert!(l.hit_at(Bucket::Register, "5.6.7.8", t0).is_ok());
        // One token back after 12 minutes.
        let t1 = t0 + Duration::from_secs(721);
        assert!(l.hit_at(Bucket::Register, "1.2.3.4", t1).is_ok());
        assert!(l.hit_at(Bucket::Register, "1.2.3.4", t1).is_err());
    }

    #[test]
    fn check_does_not_consume() {
        let l = Limiter::new(1);
        let t0 = Instant::now();
        for _ in 0..20 {
            assert!(l
                .check_at(Bucket::LoginFailuresPerEmail, "a@b.c", t0)
                .is_ok());
        }
        for _ in 0..10 {
            l.hit_at(Bucket::LoginFailuresPerEmail, "a@b.c", t0)
                .unwrap();
        }
        assert!(l
            .check_at(Bucket::LoginFailuresPerEmail, "a@b.c", t0)
            .is_err());
    }

    #[test]
    fn multiplier_scales_capacity() {
        let l = Limiter::new(10);
        let t0 = Instant::now();
        for _ in 0..50 {
            assert!(l.hit_at(Bucket::Register, "ip", t0).is_ok());
        }
        assert!(l.hit_at(Bucket::Register, "ip", t0).is_err());
    }

    #[test]
    fn cooldown_blocks_until_period_passes() {
        let l = Limiter::new(1);
        let t0 = Instant::now();
        let p = Duration::from_secs(60);
        assert!(l.cooldown_at("sync", "user:a", p, t0).is_ok());
        let rem = l
            .cooldown_at("sync", "user:a", p, t0 + Duration::from_secs(10))
            .unwrap_err();
        assert_eq!(rem, Duration::from_secs(50));
        assert!(l.cooldown_at("sync", "user:b", p, t0).is_ok());
        assert!(l.cooldown_at("sync", "user:a", p, t0 + p).is_ok());
    }

    #[test]
    fn purge_drops_only_entries_that_no_longer_limit() {
        let l = Limiter::new(1);
        let t0 = Instant::now();
        // Spent bucket: must survive until it has refilled.
        for _ in 0..5 {
            l.hit_at(Bucket::Register, "spent", t0).unwrap();
        }
        // Barely used Riot bucket: full again after 2 s.
        l.hit_at(Bucket::Riot, "light", t0).unwrap();
        l.cooldown_at("sync", "user:a", Duration::from_secs(60), t0)
            .unwrap();
        assert_eq!(l.len(), 3);

        assert_eq!(l.purge_at(t0 + Duration::from_secs(3)), 1, "light refilled");
        assert!(l
            .hit_at(Bucket::Register, "spent", t0 + Duration::from_secs(3))
            .is_err());

        let later = t0 + Duration::from_secs(3601);
        assert_eq!(l.purge_at(later), 2, "refilled bucket and stale cooldown");
        assert!(l.is_empty());
    }

    #[test]
    fn riot_budget_has_a_short_and_a_long_window() {
        let t0 = Instant::now();
        let b = RiotBudget::new_at(20, 50, t0);
        assert_eq!(b.max_burst(), 100);
        // 20 per second...
        for _ in 0..20 {
            b.try_take_at(t0).unwrap();
        }
        let wait = b.try_take_at(t0).unwrap_err();
        assert!(wait <= Duration::from_millis(60), "{wait:?}");
        // ...and about 100 per two minutes overall: keep taking at full speed
        // for 10 s and the long window caps the total.
        let mut taken = 20;
        for sec in 1..=10 {
            let t = t0 + Duration::from_secs(sec);
            while b.try_take_at(t).is_ok() {
                taken += 1;
            }
        }
        assert!((100..=110).contains(&taken), "took {taken}");
        let wait = b.try_take_at(t0 + Duration::from_secs(10)).unwrap_err();
        assert!(
            !wait.is_zero() && wait <= Duration::from_millis(1300),
            "long window is spent (one token per 1.2 s): {wait:?}"
        );
    }

    #[test]
    fn riot_budget_headroom_is_capped_at_the_burst() {
        let t0 = Instant::now();
        let b = RiotBudget::new_at(20, 50, t0);
        assert!(
            b.headroom_at(500, t0).is_ok(),
            "an idle budget lets a big job start"
        );
        for _ in 0..20 {
            b.try_take_at(t0).unwrap();
        }
        let wait = b.headroom_at(100, t0).unwrap_err();
        assert!(wait >= Duration::from_secs(20), "{wait:?}");
        assert!(b.headroom_at(80, t0).is_ok());
    }

    #[test]
    fn message_mentions_wait() {
        assert!(slow_down_message(Duration::from_secs(30)).contains("30 seconds"));
        assert!(slow_down_message(Duration::from_secs(600)).contains("10 minutes"));
    }
}
