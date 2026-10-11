use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use surrealdb::{engine::local::Db, types::SurrealValue, Surreal};
use tower_sessions_core::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};

/// tower-sessions store backed by a SurrealDB table.
///
/// Each row holds the MessagePack-encoded `Record` in `data` and its expiry as
/// unix seconds in `expires_at`. `load` treats an expired row as absent (and
/// deletes it); `delete_expired` purges all expired rows and is run
/// periodically from `main`.
#[derive(Clone)]
pub struct SurrealSessionStore {
    db: Arc<Surreal<Db>>,
    table: String,
}

impl fmt::Debug for SurrealSessionStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SurrealSessionStore")
            .field("table", &self.table)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct SessionRow {
    data: Vec<u8>,
}

fn backend(e: impl fmt::Display) -> session_store::Error {
    session_store::Error::Backend(e.to_string())
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

impl SurrealSessionStore {
    pub fn new(db: Arc<Surreal<Db>>, table: String) -> Self {
        Self { db, table }
    }

    /// Write `record` (insert or replace).
    async fn write(&self, record: &Record, create_only: bool) -> session_store::Result<bool> {
        let data =
            rmp_serde::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;
        let query = if create_only {
            // CREATE fails (returns no row) if the record already exists.
            "CREATE type::record($tb, $id) SET data = $data, expires_at = $expires_at"
        } else {
            "UPSERT type::record($tb, $id) SET data = $data, expires_at = $expires_at"
        };
        let mut result = self
            .db
            .query(query)
            .bind(("tb", self.table.clone()))
            .bind(("id", record.id.to_string()))
            .bind(("data", data))
            .bind(("expires_at", record.expiry_date.unix_timestamp()))
            .await
            .map_err(backend)?;
        let row: Option<SessionRow> = result.take(0).map_err(backend)?;
        Ok(row.is_some())
    }
}

#[async_trait]
impl SessionStore for SurrealSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        // Loop to handle the unlikely case of an ID collision.
        loop {
            if self.write(record, true).await.unwrap_or(false) {
                return Ok(());
            }
            record.id = Id::default();
        }
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        self.write(record, false).await.map(|_| ())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let mut result = self
            .db
            .query("SELECT data FROM type::record($tb, $id)")
            .bind(("tb", self.table.clone()))
            .bind(("id", session_id.to_string()))
            .await
            .map_err(backend)?;

        let row: Option<SessionRow> = result.take(0).map_err(backend)?;
        let Some(row) = row else {
            return Ok(None);
        };
        let record: Record = rmp_serde::from_slice(&row.data)
            .map_err(|e| session_store::Error::Decode(e.to_string()))?;

        if record.expiry_date.unix_timestamp() <= now_unix() {
            // Expired: behave as if it were gone, and remove it.
            self.delete(session_id).await?;
            return Ok(None);
        }
        Ok(Some(record))
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        self.db
            .query("DELETE type::record($tb, $id)")
            .bind(("tb", self.table.clone()))
            .bind(("id", session_id.to_string()))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for SurrealSessionStore {
    /// Delete expired rows. Rows without `expires_at` predate M1 (and carry a
    /// session auth hash that no longer matches), so they go too.
    async fn delete_expired(&self) -> session_store::Result<()> {
        self.db
            .query("DELETE type::table($tb) WHERE expires_at = NONE OR expires_at <= $now")
            .bind(("tb", self.table.clone()))
            .bind(("now", now_unix()))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }
}
