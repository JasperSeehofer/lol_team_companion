---
paths: ["**/db.rs", "**/schema.surql", "**/session_store.rs"]
description: SurrealDB 3.x patterns and gotchas for lol_team_companion
---

# SurrealDB Patterns

## Critical Gotchas

1. **`type::record()` not `type::thing()`** — `type::thing()` was removed in SurrealDB 2.x. Always use `type::record('table', $key)`.

2. **Strip the table prefix before passing keys** — `type::record('user', $key)` expects just the key part. Always strip: `user_id.strip_prefix("user:").unwrap_or(&user_id).to_string()`.

3. **RecordId deserialization** — Never deserialize SurrealDB query results as `serde_json::Value`. Create `Db*` structs with `surrealdb::RecordId` fields, then convert to app-facing structs with `String` IDs. See `DbTeam` → `Team` in `db.rs`.

4. **`.bind()` requires `'static`** — Always pass owned `String` values to `.bind()`, never `&str`.

5. **No `string()` cast in SurQL** — SurrealDB 2.x removed `string()`. Use `.to_sql()` on `RecordId` in Rust code instead.

6. **`SurrealValue` derive** — Use `#[derive(SurrealValue)]` from `surrealdb-types-derive` on DB result structs.

## Query Patterns

27. **`.check()` on write queries** — After `CREATE`/`UPDATE`/`DELETE`, call `.check()` to surface constraint violations and query errors. Without it, a failed write silently returns `Ok`:
    ```rust
    db.query("CREATE ...").bind(...).await?.check()?;
    ```
    Read queries (`SELECT`) surface errors via `response.take()` instead.

28. **`take(0).unwrap_or_default()` for list queries** — For queries that return `Vec<T>`, use `unwrap_or_default()` rather than `?` so an empty result doesn't error:
    ```rust
    let rows: Vec<MyStruct> = result.take(0).unwrap_or_default();
    ```
    For single-record lookups returning `Option<T>`, use `result.take(0)?` instead.

29. **Batch multiple queries in one call** — Chain statements in a single `.query()` to avoid round-trips. Index results by statement order:
    ```rust
    let mut r = db.query("SELECT ...; SELECT ...;").await?;
    let teams: Vec<DbTeam> = r.take(0).unwrap_or_default();
    let members: Vec<DbTeamMember> = r.take(1).unwrap_or_default();
    ```

30. **`DEFINE FIELD IF NOT EXISTS` for all schema fields** — Schema is re-applied on every startup. `IF NOT EXISTS` makes it idempotent so existing records are never affected by schema re-runs. Never omit it.

31. **Use `BEGIN`/`COMMIT` for multi-step writes** — When multiple `CREATE`/`UPDATE` statements must succeed or fail together, wrap in a transaction:
    ```rust
    db.query("BEGIN TRANSACTION; CREATE ...; CREATE ...; COMMIT TRANSACTION;")
        .bind(...).await?.check()?;
    ```

40. **`ORDER BY` only on selected fields in partial SELECTs** — SurrealDB 3.x rejects `ORDER BY <field>` if the field is not included in a partial `SELECT` clause. Either add the field to the `SELECT` or use `SELECT *`.
