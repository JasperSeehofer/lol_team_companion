# Testing Patterns

**Analysis Date:** 2025-03-14

## Test Framework

**Runner:**
- Tokio 1.x (`cargo test --features ssr`)
- Config: Implicit in Cargo.toml via `[dev-dependencies]`

**Playwright E2E:**
- Version: Latest (pinned in `e2e/package.json`)
- Config: `e2e/playwright.config.ts`

**Assertion Library:**
- Rust: Standard `assert!`, `assert_eq!`
- TypeScript: `@playwright/test` `expect()`

**Run Commands:**
```bash
cargo test --features ssr              # All tests (19 unit + 25 integration)
cargo test --features ssr --lib        # Unit tests only (skip integration)
cargo test --features ssr --test db_*  # Specific integration suite
just test                              # Alias for cargo test --features ssr
just verify                            # test + check + lint + fmt --check
just e2e                               # Playwright suite (requires running server)
just e2e-headed                        # Playwright with visible browser
just e2e-ui                            # Playwright interactive UI
just smoke                             # health + api-test (runtime checks)
```

## Test File Organization

**Location:**
- Unit tests: Co-located with source in `#[cfg(test)]` modules within `src/`
- Integration tests: Separate files in `tests/` directory
- E2E tests: TypeScript in `e2e/tests/`

**Unit Test Files:**
- `src/models/draft.rs` — Draft and DraftAction JSON round-trip tests
- `src/models/user.rs` — TeamMember JSON serialization tests
- `src/models/champion.rs` — Champion metadata tests
- `src/models/game_plan.rs` — GamePlan/PostGameLearning tests
- `src/models/action_item.rs` — ActionItem tests
- `src/models/team_note.rs` — TeamNote tests
- `src/models/series.rs` — Series tests
- `src/server/auth.rs` — Password hashing and verification tests
- `src/server/db.rs` — Database integration helpers tests

**Integration Test Files:**
- `tests/db_users.rs` — User creation, duplicate email validation (3 tests)
- `tests/db_teams.rs` — Team creation, join flow, roster slots, member management (8 tests)
- `tests/db_drafts.rs` — Draft CRUD, action aggregation, list operations (4 tests)
- `tests/db_champion_pool.rs` — Champion pool CRUD, tier updates, idempotence (5 tests)
- `tests/db_tree.rs` — Draft tree hierarchy, node creation, cascade delete, sort order (5 tests)
- `tests/common/mod.rs` — Shared in-memory SurrealDB setup

**E2E Test Files:**
- `e2e/tests/smoke.spec.ts` — Public page health checks, health endpoint, auth redirect
- `e2e/tests/auth.spec.ts` — Registration, login, error handling, full auth flow, logout
- `e2e/tests/pages.spec.ts` — Authenticated page load tests for all protected routes
- `e2e/tests/fixtures.ts` — Shared authedPage fixture for authenticated tests

## Test Structure

**Unit Test Pattern:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_behavior() {
        let input = create_test_data();
        let result = function_under_test(input);
        assert_eq!(result, expected);
    }
}
```

**Integration Test Pattern (Async with Setup):**
```rust
#![cfg(feature = "ssr")]
mod common;

use lol_team_companion::server::db;

async fn make_user(db: &Surreal<Db>) -> String {
    db::create_user(db, "name".into(), "email".into(), "hash".into())
        .await
        .unwrap()
}

#[tokio::test]
async fn test_db_operation() {
    let db = common::test_db().await;  // In-memory SurrealDB with schema applied
    let user_id = make_user(&db).await;
    let result = db::some_operation(&db, &user_id).await.unwrap();
    assert!(result.is_some());
}
```

**Patterns:**
- All integration tests use `#[tokio::test]` for async execution
- In-memory SurrealDB via `common::test_db()` — loads schema from `schema.surql` on each test
- Helper functions like `make_user()` and `setup()` shared within test files to reduce boilerplate
- Database state is isolated per test (fresh in-memory instance each time)

## SurrealDB Test Database Setup

**Location:** `tests/common/mod.rs`

**Initialization Pattern:**
```rust
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
```

**Key Points:**
- Uses `Mem` in-memory backend for fast, isolated tests
- Loads full schema on startup via `include_str!()` — same schema as production
- Returns `Arc<Surreal<Db>>` to match server code expectations
- No teardown needed — fresh instance per test, garbage collected after

## Mocking

**Patterns:**
- **SurrealDB:** Replaced entirely with in-memory `Mem` backend in integration tests
- **Riot API:** Not mocked — Data Dragon champion data used instead
- **Auth:** `AuthSession` created directly; session store uses in-memory for tests
- **External services:** Not called in unit/integration tests

**What to Mock:**
- Async dependencies (if testing component interactions)
- External API calls (when not testing the integration itself)

**What NOT to Mock:**
- Database layer — use real in-memory SurrealDB instead
- Schema validation — run actual schema load to catch field/type errors
- Core business logic — test against real implementations to catch integration bugs

## Fixtures and Factories

**Test Data Factories:**

Unit test example (JSON round-trip):
```rust
#[test]
fn draft_round_trips_json() {
    let d = Draft {
        id: Some("draft:1".into()),
        name: "Test Draft".into(),
        team_id: "team:t1".into(),
        created_by: "user:u1".into(),
        opponent: Some("Team Evil".into()),
        our_side: "blue".into(),
        actions: vec![],
        tags: vec!["teamfight".into()],
    };
    let json = serde_json::to_string(&d).unwrap();
    let back: Draft = serde_json::from_str(&json).unwrap();
    assert_eq!(d, back);
}
```

Integration test factory (from `tests/db_drafts.rs`):
```rust
fn sample_actions(draft_id: &str) -> Vec<DraftAction> {
    vec![
        DraftAction {
            id: None,
            draft_id: draft_id.into(),
            phase: "ban1".into(),
            side: "blue".into(),
            champion: "Azir".into(),
            order: 0,
            comment: None,
        },
        DraftAction {
            id: None,
            draft_id: draft_id.into(),
            phase: "pick1".into(),
            side: "blue".into(),
            champion: "Jinx".into(),
            order: 1,
            comment: Some("strong ADC".into()),
        },
    ]
}
```

**Location:**
- Unit test fixtures: Inline within `#[test]` functions
- Integration test factories: Defined at module level (e.g., `sample_actions()`, `make_user()`, `setup()`)
- E2E fixtures: Shared via `fixtures.ts` providing `authedPage` fixture

## Coverage

**Requirements:** Not enforced (no coverage targets in CI)

**Distribution (Approximate):**
- **Models:** ~90% — serialization, conversion, JSON round-trips all tested
- **Database layer:** ~80% — CRUD operations and complex queries tested via integration tests
- **Auth:** ~70% — password hashing, verification tested; session management partly covered by e2e
- **Server handlers:** Minimal direct testing — covered via e2e instead
- **UI/Components:** Zero unit tests — covered by e2e tests only

**Measure Coverage:**
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --features ssr --exclude-files tests/ --timeout 300
```

## Test Types

**Unit Tests (25 total):**
- Scope: Individual functions, serialization, type conversions
- Approach: Run within source file `#[cfg(test)]` blocks via `cargo test --lib`
- Examples: Draft/DraftAction JSON round-trips, TeamMember serialization, password hashing, champion validation

**Integration Tests (25 total):**
- Scope: Database operations, multi-step workflows, constraint enforcement
- Approach: Run via `cargo test --features ssr --test db_*` using in-memory SurrealDB
- Examples: User creation + duplicate detection, team join flow, roster slot assignment, draft CRUD, champion pool management, draft tree hierarchy

**E2E Tests (3 suites, 22 test cases):**

`smoke.spec.ts` (4 tests):
- Public pages load without JS errors
- Health endpoint returns ok
- Unauthenticated redirect from protected routes

`auth.spec.ts` (8 tests):
- Registration form visibility and success
- Login form visibility and error handling
- Full auth flow (register → auto-login → profile accessible)
- Logout clears session

`pages.spec.ts` (10 tests):
- All protected routes load without errors: `/profile`, `/team/dashboard`, `/team/roster`, `/team-builder`, `/draft`, `/tree-drafter`, `/stats`, `/champion-pool`, `/game-plan`, `/post-game`

## Common Patterns

**Async Testing:**

Standard pattern (used in all 25 integration tests):
```rust
#[tokio::test]
async fn test_async_operation() {
    let db = common::test_db().await;
    let result = db::some_async_fn(&db, args).await;
    assert!(result.is_ok());
}
```

**Error Testing:**

From `tests/db_teams.rs`:
```rust
#[tokio::test]
async fn test_join_team_already_member_fails() {
    let db = common::test_db().await;
    let u1 = make_user(&db, "dup_member").await;
    let team_id = db::create_team(&db, &u1, "Gamma".into(), "NA".into())
        .await
        .unwrap();
    let result = db::join_team(&db, &u1, &team_id).await;
    assert!(result.is_err(), "joining a team you're already in should fail");
}
```

**Complex Workflow Testing:**

From `tests/db_teams.rs` — multi-step join request flow:
```rust
#[tokio::test]
async fn test_join_request_flow_accept() {
    let db = common::test_db().await;
    let owner = make_user(&db, "owner_accept").await;
    let joiner = make_user(&db, "joiner_accept").await;
    let team_id = db::create_team(&db, &owner, "Delta".into(), "EUW".into())
        .await
        .unwrap();

    db::create_join_request(&db, &joiner, &team_id).await.unwrap();
    let requests = db::list_pending_join_requests(&db, &team_id).await.unwrap();
    assert_eq!(requests.len(), 1);

    db::respond_to_join_request(&db, &requests[0].id, true, &team_id)
        .await
        .unwrap();

    let team_id_for_joiner = db::get_user_team_id(&db, &joiner).await.unwrap();
    assert_eq!(team_id_for_joiner, Some(team_id.clone()));

    let remaining = db::list_pending_join_requests(&db, &team_id).await.unwrap();
    assert!(remaining.is_empty());
}
```

**E2E Form Testing:**

From `e2e/tests/auth.spec.ts`:
```typescript
test("can register a new account", async ({ page }) => {
  await page.goto("/auth/register");
  await page.fill("input[name=username]", TEST_USERNAME);
  await page.fill("input[name=email]", TEST_EMAIL);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("button[type=submit]");

  await page.waitForLoadState("networkidle");
  expect(page.url()).not.toContain("/auth/register");
});
```

**E2E Auth Fixture:**

From `e2e/tests/fixtures.ts`:
```typescript
async function authenticatePage(page: Page): Promise<void> {
  await page.goto("/auth/register");
  await page.fill("input[name=username]", username);
  await page.fill("input[name=email]", email);
  await page.fill("input[name=password]", TEST_PASSWORD);
  await page.click("button[type=submit]");

  await page.waitForURL("**/team/dashboard", { timeout: 20000 });
  await page.waitForLoadState("load");
  await page.waitForTimeout(500);
}

export const test = base.extend<Fixtures>({
  authedPage: async ({ page }, use) => {
    await authenticatePage(page);
    await use(page);
  },
});
```

## Coverage Gaps

**Critical Gaps:**

1. **Server-side rendering validation**
   - What's not tested: SSR-only code paths in page components
   - Files: `src/pages/*.rs` (each page has `#[server]` functions)
   - Risk: Server function errors, type mismatches, DB query issues not caught until runtime
   - Priority: Medium — E2E tests cover observable behavior but not internal error handling

2. **Leptos component reactivity**
   - What's not tested: Signal updates, Effects, Resource synchronization
   - Files: All component files in `src/pages/`, `src/components/`
   - Risk: Stale data in signals, missed Effect triggers, incorrect reactive dependencies
   - Priority: Medium — UI freezes or state bugs only visible in browser; e2e tests would catch UI breaks but not logic errors

3. **External API integrations (Riot API)**
   - What's not tested: Actual Riot API calls, rate limiting, error handling
   - Files: `src/server/riot.rs`
   - Risk: Invalid response parsing, network errors, API key issues
   - Priority: Low — API layer mostly pass-through; errors would surface during smoke tests

4. **Tree assembly algorithm edge cases**
   - What's not tested: Deeply nested hierarchies (>10 levels), unusual sort_order patterns, performance at scale
   - Files: `src/server/db.rs` (get_tree_nodes function)
   - Risk: Incorrect parent-child mapping, lost nodes
   - Priority: Low — Current tests cover basic and moderate nesting

5. **Draft action ordering**
   - What's not tested: Correct ban/pick phase sequencing, rotation rule validation
   - Files: `src/models/draft.rs`, `src/server/db.rs`
   - Risk: Phase order bugs (pick before ban), duplicate phases
   - Priority: Medium — Draft logic is critical but tested only via e2e

6. **Champion pool concurrent edits**
   - What's not tested: Bulk tier updates, conflicting changes during concurrent user edits
   - Files: `src/server/db.rs` (update_champion_tier)
   - Risk: Data loss during concurrent updates, invalid tier values
   - Priority: Low — Single-user edits work; multi-user conflicts not validated

7. **Session security**
   - What's not tested: Session expiration, CSRF protection, cookie flags, secure cookie transmission
   - Files: `src/server/session_store.rs`, `src/main.rs`
   - Risk: Session hijacking, cross-site request forgery
   - Priority: High — No explicit CSRF tests; relies on tower-sessions + axum defaults

8. **Error message content**
   - What's not tested: Specific error messages returned to clients, sensitive info in errors
   - Files: All `#[server]` functions in `src/pages/`
   - Risk: Unhelpful errors leak to UI; API keys or SQL queries exposed
   - Priority: Medium — Currently returns generic ServerFnError messages; no validation of error text

---

*Testing analysis: 2025-03-14*