//! Small in-memory rate limiter (token buckets keyed by bucket + client) and
//! per-user cooldowns. One process, one map: fine for the single-instance
//! deployment (embedded SurrealKV already pins us to one instance).
//!
//! Capacities scale with `RATE_LIMIT_MULTIPLIER` (see `config`), so e2e runs
//! that register hundreds of users from 127.0.0.1 can raise them.

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

const MAX_KEYS: usize = 50_000;

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
        if map.len() > MAX_KEYS {
            // Drop idle entries (their buckets would be full again anyway).
            map.retain(|_, s| now.saturating_duration_since(s.last) < Duration::from_secs(3600));
        }
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
        if map.len() > MAX_KEYS {
            map.retain(|_, t| now.saturating_duration_since(*t) < Duration::from_secs(3600));
        }
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
    fn message_mentions_wait() {
        assert!(slow_down_message(Duration::from_secs(30)).contains("30 seconds"));
        assert!(slow_down_message(Duration::from_secs(600)).contains("10 minutes"));
    }
}
