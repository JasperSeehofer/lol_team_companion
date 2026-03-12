use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use surrealdb::{engine::local::Db, types::SurrealValue, Surreal};
use tower_sessions_core::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};

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

impl SurrealSessionStore {
    pub fn new(db: Arc<Surreal<Db>>, table: String) -> Self {
        Self { db, table }
    }
}

#[async_trait]
impl SessionStore for SurrealSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        // Loop to handle the unlikely case of an ID collision.
        loop {
            let id_str = record.id.to_string();
            let data = rmp_serde::to_vec(&*record)
                .map_err(|e| session_store::Error::Encode(e.to_string()))?;

            // CREATE fails (returns no row) if the record already exists.
            let mut result = self
                .db
                .query(format!(
                    "CREATE type::record('{}', $id) SET data = $data",
                    self.table
                ))
                .bind(("id", id_str))
                .bind(("data", data))
                .await
                .map_err(|e| session_store::Error::Backend(e.to_string()))?;

            let created: Option<SessionRow> = result
                .take(0)
                .map_err(|e| session_store::Error::Backend(e.to_string()))?;

            if created.is_some() {
                return Ok(());
            }

            // Collision — generate a new ID and retry.
            record.id = Id::default();
        }
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let id_str = record.id.to_string();
        let data =
            rmp_serde::to_vec(record).map_err(|e| session_store::Error::Encode(e.to_string()))?;

        self.db
            .query(format!(
                "UPSERT type::record('{}', $id) SET data = $data",
                self.table
            ))
            .bind(("id", id_str))
            .bind(("data", data))
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let id_str = session_id.to_string();

        let mut result = self
            .db
            .query(format!(
                "SELECT data FROM type::record('{}', $id)",
                self.table
            ))
            .bind(("id", id_str))
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        let row: Option<SessionRow> = result
            .take(0)
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        match row {
            Some(r) => {
                let record: Record = rmp_serde::from_slice(&r.data)
                    .map_err(|e| session_store::Error::Decode(e.to_string()))?;
                Ok(Some(record))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let id_str = session_id.to_string();

        self.db
            .query(format!("DELETE type::record('{}', $id)", self.table))
            .bind(("id", id_str))
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for SurrealSessionStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        Ok(())
    }
}
