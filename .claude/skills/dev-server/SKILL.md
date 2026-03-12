---
name: dev-server
description: Start the dev server reliably and verify it's accessible. Use when you need the running app for browser testing, Playwright MCP verification, or API checks. Trigger when starting a dev session, when the server is down, or when "/dev-server" is invoked.
---

# Dev Server Startup

Reliable procedure for starting and connecting to the LoL Team Companion dev server. Handles the common failure modes: zombie processes, port conflicts, and schema changes not triggering rebuilds.

## Startup Procedure

### Step 1: Clean up any existing processes

Always check for and kill leftover processes before starting. Zombie `cargo-leptos` or `lol_team_companion` processes hold ports and cause silent failures.

```bash
pkill -f "cargo-leptos" 2>/dev/null
pkill -f "lol_team_companion" 2>/dev/null
sleep 2
```

Verify ports are free:

```bash
ss -tlnp | grep -E "300[23]" || echo "Ports cleared"
```

If ports are still held, wait a few more seconds. Do NOT proceed until both 3002 and 3003 are free.

### Step 2: Start the server

```bash
cargo leptos watch 2>&1   # run_in_background: true
```

### Step 3: Wait for readiness

Use an inline polling loop (NOT the `wait_for_server.sh` script, which can be rejected by the user as a long-running tool call):

```bash
for i in $(seq 1 90); do
  if curl -sf http://127.0.0.1:3002/healthz > /dev/null 2>&1; then
    echo "Server ready after ${i}s"
    exit 0
  fi
  sleep 1
done
echo "Timeout"
```

Use `timeout: 100000` on this Bash call.

- The server typically starts in 1-5s if already compiled, or 30-60s for a fresh build.
- Port **3002** is the app server. Port **3003** is the live-reload websocket (not needed for health checks).
- The health endpoint is `GET /healthz`.

### Step 4: Verify in browser

After the server is ready, navigate via Playwright MCP:

```
browser_navigate → http://127.0.0.1:3002/
browser_snapshot → confirm page renders
```

## When the Server Fails to Start

### "Serving at 127.0.0.1:3002" but port not listening

`cargo leptos watch` logs "Serving" but the process exits immediately. This happens when:
- A previous process still holds port 3003 (the reload port)
- The server binary crashes on startup (e.g., DB init failure)

**Fix:** Go back to Step 1 — kill ALL related processes, wait for ports to clear, restart.

### "Reload TCP port 127.0.0.1:3003 already in use"

This warning is often harmless (server still works on 3002). But if combined with the server not responding, it means a zombie process holds port 3003.

**Fix:** `pkill -f "cargo-leptos" && pkill -f "lol_team_companion"`, wait, restart.

### Schema changes not taking effect

`schema.surql` is embedded via `include_str!()` in `main.rs`. Cargo does NOT track `include_str!` file changes for rebuild. If you changed `schema.surql`:

```bash
touch src/main.rs   # force cargo to rebuild main.rs
```

Then restart the server. Verify by checking the server output for "Cargo finished" with a non-zero compile time (not "0.07s").

### Server starts but DB field errors persist

If `DEFINE FIELD IF NOT EXISTS` was used and the DB already has the old field type, the schema won't update. Change to `DEFINE FIELD OVERWRITE` for that specific field in `schema.surql`, then restart.

## Auth for Browser Testing

After the server is up, register a test user for authenticated page testing:

1. `browser_navigate` to `/auth/register`
2. Fill username, email, password via `browser_fill_form`
3. Click "Create Account"
4. Registration auto-logs in and redirects to `/team/dashboard`
5. All subsequent navigations in the same Playwright session are authenticated

## Key Facts

- App URL: `http://127.0.0.1:3002`
- Health check: `GET /healthz`
- Live-reload port: 3003 (websocket, not needed for testing)
- `WebFetch` tool cannot reach localhost (upgrades HTTP to HTTPS) — use `curl` via Bash or Playwright MCP
- `cargo leptos watch` auto-recompiles on `.rs` file changes but NOT on `schema.surql` changes
