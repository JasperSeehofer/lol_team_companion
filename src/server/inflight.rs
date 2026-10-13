//! Requests in flight (M11). Every HTTP request is registered from the
//! moment it arrives until its response body has been sent completely, or
//! the client went away. A watchdog ([`spawn_watchdog`]) logs requests that
//! have run longer than [`SLOW`], with the last step each one reached and the
//! database handles held, so a request that never answers leaves a trace in
//! the log instead of silence.
//!
//! Steps are short static labels noted with [`note`] by the code a request
//! passes through (session load, user load, server functions borrowing the
//! database, the response head, the body).

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use axum::body::{Body, Bytes, HttpBody};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

/// A request running longer than this is reported by the watchdog.
pub const SLOW: Duration = Duration::from_secs(15);

/// Words of the watchdog's line for a request without a complete response.
/// CI's "No stalled requests in the server log" step greps for them (a unit
/// test keeps `.github/workflows/ci.yml` and these in step).
pub const NO_COMPLETE_RESPONSE: &str = "no complete response";
/// How the watchdog describes a request whose client stopped reading (not a
/// server stall; CI ignores those lines).
pub const CLIENT_NOT_READING: &str = "the client is not reading (no poll since the last frame)";

/// The id of one request, in its extensions (and so in the `Parts` that
/// Leptos provides as context).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

tokio::task_local! {
    static CURRENT: RequestId;
}

#[derive(Debug)]
struct Entry {
    method: String,
    path: String,
    started: Instant,
    steps: Vec<(&'static str, Duration)>,
    reported: bool,
    body: BodyProgress,
}

/// How far the response body got: tells a server that has nothing more to
/// send (the last poll was `Pending`) from a client that stopped reading
/// (the last poll returned a frame and hyper has not asked again).
#[derive(Debug, Default, Clone, Copy)]
struct BodyProgress {
    frames: u64,
    bytes: u64,
    /// The last poll returned `Pending`.
    waiting_on_server: bool,
    last_poll: Option<Duration>,
}

fn table() -> &'static Mutex<HashMap<u64, Entry>> {
    static TABLE: OnceLock<Mutex<HashMap<u64, Entry>>> = OnceLock::new();
    TABLE.get_or_init(Default::default)
}

fn lock() -> std::sync::MutexGuard<'static, HashMap<u64, Entry>> {
    table().lock().unwrap_or_else(|p| p.into_inner())
}

/// The request this code runs for: the task-local set by [`track`] (the
/// middleware layers and the handler's own task), else the `Parts` Leptos
/// provides as context (server functions run during a render, which may be
/// spawned onto another task).
pub fn current() -> Option<RequestId> {
    if let Ok(id) = CURRENT.try_with(|id| *id) {
        return Some(id);
    }
    leptos::prelude::use_context::<axum::http::request::Parts>()
        .and_then(|p| p.extensions.get::<RequestId>().copied())
}

/// Record that the current request reached `step`. Keeps the last 12 steps.
pub fn note(step: &'static str) {
    if let Some(id) = current() {
        note_for(id, step);
    }
}

/// [`note`] for a known request (from a task without its context).
pub fn note_for(id: RequestId, step: &'static str) {
    let mut map = lock();
    if let Some(e) = map.get_mut(&id.0) {
        let at = e.started.elapsed();
        e.steps.push((step, at));
        if e.steps.len() > 12 {
            e.steps.remove(0);
        }
    }
}

/// Removes the request from the table when dropped: when the body ends, or
/// when hyper drops the response (the client went away).
struct Done(RequestId);

impl Drop for Done {
    fn drop(&mut self) {
        let entry = lock().remove(&self.0 .0);
        if let Some(e) = entry {
            if e.reported {
                tracing::warn!(
                    "slow request {} {} finished after {:.1}s",
                    e.method,
                    e.path,
                    e.started.elapsed().as_secs_f64()
                );
            }
        }
    }
}

/// The response body, which ends the request's entry when it is complete.
struct TrackedBody {
    inner: Body,
    done: Option<Done>,
}

impl HttpBody for TrackedBody {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let polled = Pin::new(&mut self.inner).poll_frame(cx);
        if let Some(done) = &self.done {
            let mut map = lock();
            if let Some(e) = map.get_mut(&done.0 .0) {
                e.body.last_poll = Some(e.started.elapsed());
                match &polled {
                    Poll::Pending => e.body.waiting_on_server = true,
                    Poll::Ready(Some(Ok(frame))) => {
                        e.body.waiting_on_server = false;
                        e.body.frames += 1;
                        e.body.bytes += frame.data_ref().map(|d| d.len() as u64).unwrap_or(0);
                    }
                    _ => {}
                }
            }
        }
        if let Poll::Ready(None) | Poll::Ready(Some(Err(_))) = &polled {
            self.done.take();
        }
        polled
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

/// Axum middleware (outermost layer): registers the request, runs the rest
/// of the stack with the request id as task-local, and keeps the entry until
/// the response body is complete.
pub async fn track(mut req: Request, next: Next) -> Response {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = RequestId(NEXT.fetch_add(1, Ordering::Relaxed));
    lock().insert(
        id.0,
        Entry {
            method: req.method().to_string(),
            path: req.uri().path().to_string(),
            started: Instant::now(),
            steps: Vec::new(),
            reported: false,
            body: BodyProgress::default(),
        },
    );
    req.extensions_mut().insert(id);
    let done = Done(id);
    let response = CURRENT.scope(id, next.run(req)).await;
    note_for(id, "response head");
    let (parts, body) = response.into_parts();
    Response::from_parts(
        parts,
        Body::new(TrackedBody {
            inner: body,
            done: Some(done),
        }),
    )
}

/// Requests older than `slow` that were not reported yet, marked reported:
/// (method, path, age, steps).
fn take_slow(slow: Duration) -> Vec<(String, String, Duration, String)> {
    let mut out = Vec::new();
    for e in lock().values_mut() {
        let age = e.started.elapsed();
        if age >= slow && !e.reported {
            e.reported = true;
            let mut steps = e
                .steps
                .iter()
                .map(|(s, at)| format!("{s} @{:.2}s", at.as_secs_f64()))
                .collect::<Vec<_>>()
                .join(", ");
            let b = e.body;
            steps.push_str(&format!(
                "; body: {} frames, {} bytes, last poll {}, {}",
                b.frames,
                b.bytes,
                b.last_poll
                    .map(|t| format!("@{:.2}s", t.as_secs_f64()))
                    .unwrap_or_else(|| "never".into()),
                if b.last_poll.is_none() {
                    "not polled yet"
                } else if b.waiting_on_server {
                    "the server has nothing more to send yet (stream pending)"
                } else {
                    CLIENT_NOT_READING
                }
            ));
            out.push((e.method.clone(), e.path.clone(), age, steps));
        }
    }
    out
}

/// Who holds `db::invariant_lock` (the global write lock) and since when:
/// a request (its id) or a task outside any request (`None`).
fn holder() -> std::sync::MutexGuard<'static, Option<(Option<RequestId>, Instant)>> {
    static HOLDER: Mutex<Option<(Option<RequestId>, Instant)>> = Mutex::new(None);
    HOLDER.lock().unwrap_or_else(|p| p.into_inner())
}

/// `db::invariant_lock` is about to wait for the lock.
pub fn invariant_lock_wait() {
    note("invariant lock wait");
}

/// `db::invariant_lock` got the lock.
pub fn invariant_lock_taken() {
    let id = current();
    if let Some(id) = id {
        note_for(id, "invariant lock held");
    }
    *holder() = Some((id, Instant::now()));
}

/// The guard of `db::invariant_lock` was dropped.
pub fn invariant_lock_released() {
    let held = holder().take();
    if let Some((Some(id), _)) = held {
        note_for(id, "invariant lock released");
    }
}

/// The lock's holder for the watchdog line: "free", or for how long and by
/// which request (with that request's method, path and last steps).
fn invariant_lock_state() -> String {
    let held = *holder();
    match held {
        None => "free".into(),
        Some((None, since)) => format!(
            "held for {:.1}s by a task outside any request",
            since.elapsed().as_secs_f64()
        ),
        Some((Some(id), since)) => {
            let map = lock();
            let who = map
                .get(&id.0)
                .map(|e| {
                    let last = e
                        .steps
                        .iter()
                        .rev()
                        .take(3)
                        .map(|(s, at)| format!("{s} @{:.2}s", at.as_secs_f64()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{} {} (last steps: {last})", e.method, e.path)
                })
                .unwrap_or_else(|| "a request that already ended".into());
            format!(
                "held for {:.1}s by request {} {who}",
                since.elapsed().as_secs_f64(),
                id.0
            )
        }
    }
}

/// Number of requests in flight (tests and the shutdown log).
pub fn in_flight() -> usize {
    lock().len()
}

/// Every 5 s, log each request that has been running for longer than
/// [`SLOW`] (once per request), with its steps and the database handles
/// held at that moment.
pub fn spawn_watchdog() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let slow = take_slow(SLOW);
            if slow.is_empty() {
                continue;
            }
            let held = crate::server::authz::held_handles();
            let lock_state = invariant_lock_state();
            for (method, path, age, steps) in slow {
                tracing::warn!(
                    "slow request {method} {path}: {NO_COMPLETE_RESPONSE} after {:.0}s; steps: [{steps}]; database handles held: {held:?}; invariant lock: {lock_state}",
                    age.as_secs_f64()
                );
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_request_stays_registered_until_its_body_is_sent() {
        use axum::routing::get;
        use tower::ServiceExt;
        let app = axum::Router::new()
            .route(
                "/x",
                get(|| async {
                    note("handler");
                    "hello"
                }),
            )
            .layer(axum::middleware::from_fn(track));
        let response = app
            .oneshot(Request::builder().uri("/x").body(Body::empty()).unwrap())
            .await
            .unwrap();
        // The head is out, the body is not read yet: still in flight, with
        // the handler's step noted.
        let waiting: Vec<_> = lock()
            .values()
            .filter(|e| e.path == "/x")
            .map(|e| e.steps.iter().map(|(s, _)| *s).collect::<Vec<_>>())
            .collect();
        assert!(waiting
            .iter()
            .any(|s| s == &vec!["handler", "response head"]));
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(&body[..], b"hello");
        assert!(
            !lock().values().any(|e| e.path == "/x"),
            "the entry is gone once the body ended"
        );
    }

    /// CI fails the e2e job on a watchdog line for a request the server
    /// never completed: its grep must match the words the watchdog writes,
    /// or the step would pass whatever happened.
    #[test]
    fn the_ci_stall_check_matches_the_watchdog_line() {
        let ci = include_str!("../../.github/workflows/ci.yml");
        assert!(
            ci.contains(&format!("'slow request .*{NO_COMPLETE_RESPONSE}'")),
            "ci.yml greps for another wording than the watchdog writes"
        );
        let short = CLIENT_NOT_READING.split(" (").next().unwrap();
        assert!(
            ci.contains(&format!("grep -v '{short}'")),
            "ci.yml filters slow clients with another wording"
        );
    }

    /// The watchdog says who holds the global write lock (a request waiting
    /// for it would otherwise only show "invariant lock wait").
    #[tokio::test]
    async fn the_invariant_lock_holder_is_named() {
        let id = RequestId(9_000_000_002);
        lock().insert(
            id.0,
            Entry {
                method: "POST".into(),
                path: "/api/holder-test".into(),
                started: Instant::now(),
                steps: Vec::new(),
                reported: false,
                body: BodyProgress::default(),
            },
        );
        {
            let _guard = CURRENT
                .scope(id, async { crate::server::db::invariant_lock().await })
                .await;
            let state = invariant_lock_state();
            assert!(
                state.contains("POST /api/holder-test") && state.contains("invariant lock held"),
                "{state}"
            );
        }
        let steps: Vec<_> = lock()
            .remove(&id.0)
            .unwrap()
            .steps
            .iter()
            .map(|(s, _)| *s)
            .collect();
        assert_eq!(
            steps,
            vec![
                "invariant lock wait",
                "invariant lock held",
                "invariant lock released"
            ]
        );
    }

    #[test]
    fn slow_requests_are_reported_once() {
        let id = 9_000_000_001;
        lock().insert(
            id,
            Entry {
                method: "GET".into(),
                path: "/slow-test".into(),
                started: Instant::now() - Duration::from_secs(30),
                steps: vec![("session loaded", Duration::from_millis(3))],
                reported: false,
                body: BodyProgress::default(),
            },
        );
        let first = take_slow(SLOW);
        assert!(first
            .iter()
            .any(|(_, p, _, s)| p == "/slow-test" && s.contains("session loaded")));
        assert!(!take_slow(SLOW).iter().any(|(_, p, _, _)| p == "/slow-test"));
        lock().remove(&id);
    }
}
