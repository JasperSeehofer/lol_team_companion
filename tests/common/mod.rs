use std::sync::Arc;
use surrealdb::{engine::local::Db, Surreal};

pub async fn test_db() -> Arc<Surreal<Db>> {
    use surrealdb::engine::local::Mem;
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    db.query(include_str!("../../schema.surql"))
        .await
        .unwrap()
        .check()
        .unwrap();
    Arc::new(db)
}
