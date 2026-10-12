use std::sync::Arc;
use surrealdb::{engine::local::Db, Surreal};

/// Open a fresh in-memory database prepared with the baseline and every
/// migration.
pub async fn test_db() -> Arc<Surreal<Db>> {
    use surrealdb::engine::local::Mem;
    let db = open_with_retry(|| Surreal::new::<Mem>(())).await;
    lol_team_companion::server::db::prepare(&db).await.unwrap();
    Arc::new(db)
}

/// `Surreal::new` plus `use_ns("test").use_db("test")`, retried with a short
/// backoff when the embedded engine answers "Session not found": under many
/// parallel test threads a brand-new in-memory connection occasionally
/// reports that for its first request (seen in CI). Any other error fails
/// at once.
pub async fn open_with_retry<F, Fut>(open: F) -> Surreal<Db>
where
    F: Fn() -> Fut,
    Fut: std::future::IntoFuture<Output = surrealdb::Result<Surreal<Db>>>,
{
    let mut last = String::new();
    for attempt in 0..5u64 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(20 * attempt)).await;
        }
        let db = match open().await {
            Ok(db) => db,
            Err(e) if e.to_string().contains("Session not found") => {
                last = e.to_string();
                continue;
            }
            Err(e) => panic!("opening the test database failed: {e}"),
        };
        match db.use_ns("test").use_db("test").await {
            Ok(_) => return db,
            Err(e) if e.to_string().contains("Session not found") => last = e.to_string(),
            Err(e) => panic!("selecting the test namespace failed: {e}"),
        }
    }
    panic!("test database still reports a missing session after 5 attempts: {last}");
}
