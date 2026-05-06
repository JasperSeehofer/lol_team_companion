# Phase 20 — Deploy Infrastructure (Shared CAX11) (SPEC seed)

**Status:** SEED — produced by the v1.2 → v1.3 pivot on 2026-05-06. Run `/gsd-spec-phase 20` to expand.

**Milestone:** v1.3 Launch Readiness

## Goal (one sentence)

Get the production binary running on feynman-lookup's existing Hetzner CAX11 alongside the existing service, served via Caddy at the new domain.

## Decisions baked in (DECIDED in pivot — do NOT re-debate without explicit user prompt)

- **Same CAX11 as feynman-lookup** — no new VPS provisioning. The cloud-init.yml is already applied; we add to the existing host.
- **New domain** — TLD chosen in Phase 21 (e.g. `lol-companion.gg` placeholder); flag the `[[values-charter#2]]` "EU tools preferred" value if non-EU TLD is chosen.
- **Cross-compile locally** — `cargo zigbuild --target aarch64-unknown-linux-gnu.2.36` (Debian 12 glibc pin per `[[cross-project-memory]]`). Same toolchain as feynman-lookup.
- **New systemd unit** — `lol_team_companion.service`, user `lol`, listens on `127.0.0.1:3001`.
- **Caddy stanza** — new block routes the new domain to `127.0.0.1:3001`. No firewall changes; Caddy already exposes 80/443.
- **Manual deploy via `just deploy`** — no CI/CD push. Matches feynman-lookup pattern.
- **Secrets** — `EnvironmentFile=/etc/lol_team_companion.env`, populated post-deploy via SSH heredoc. Never committed; never in shell history.

## In-scope

1. **Adapt feynman-lookup infra files** at `/home/jasper/Repositories/feynman-lookup/infra/` into this project's `infra/` directory:
   - `Caddyfile.fragment` — a stanza to be appended on the host's `/etc/caddy/Caddyfile`
   - `lol_team_companion.service` — systemd unit (copy template, swap user/binary/port/env-file)
   - `RUNBOOK.md` — deploy/restart/rollback playbook
2. **Create `infra/setup-host.sh`** — idempotent script to be run once on the existing CAX11: creates user `lol`, mkdir `/srv/lol`, sets permissions, installs the systemd unit, validates Caddy config, reloads.
3. **`just deploy` recipe** — cross-compile, package WASM (`cp crate.wasm crate_bg.wasm` rename), rsync to `/srv/lol/`, ssh `systemctl restart lol_team_companion.service`. Pre-deploy: cargo checks both targets. Post-deploy smoke: `/healthz` + a server-fn endpoint + `curl -sI host/pkg/lol_team_companion_bg.wasm | grep "200 OK"`.
4. **DNS** — once domain is registered (Phase 21), point A/AAAA records at the CAX11's IP. Wait for Caddy to provision Let's Encrypt cert.
5. **Caddy CSP** — copy feynman-lookup's hardened CSP; document the `'unsafe-inline'` exception for Leptos 0.8 per-request script nonces with a TODO for nonce middleware (per `[[cross-project-memory]]`).

## Out of scope

- New VPS provisioning (decided: reuse CAX11)
- CI/CD push-to-deploy (decided: manual)
- Database server separation (single SurrealKV instance per app, file-backed in `/srv/lol/data`)
- Monitoring/alerting (deferred to post-launch)

## Success criteria (verify with `/gsd-verify-work 20`)

1. `just deploy` completes end-to-end without manual intervention (assuming a registered domain and populated env file)
2. New domain serves over HTTPS with Let's Encrypt cert auto-provisioned by Caddy
3. WASM `_bg.wasm` rename handled in deploy recipe
4. Pre-deploy smoke: both `cargo check` targets pass before rsync starts
5. Post-deploy smoke (per feynman 2026-04-16 incident):
   - `/healthz` returns 200 with valid JSON
   - A server-fn endpoint returns 200 (catches binary-arch mismatch — `status=203/EXEC` from x86 binary)
   - `curl -sI host/pkg/lol_team_companion_bg.wasm | grep "200 OK"` (catches broken WASM hydrate)
   - Smoke fails the deploy script (non-zero exit) if any check fails
6. Rollback documented: keep last-known-good binary at `/srv/lol/bin/last-good`; `just rollback` swaps and restarts.
7. RUNBOOK.md covers: first deploy, regular deploy, rollback, secret rotation, log access (`journalctl -u lol_team_companion`), backup-restore (handed off to Phase 19 script), DNS change, cert troubleshooting.

## Hard NOs

- No `--no-verify` on git operations (per project guardrails)
- No deploy from the main branch without all CI checks green
- No SSH password auth (key-only, per `[[cross-project-memory]]`)
- No public-readable `/etc/lol_team_companion.env` (mode 0600 root:lol)

## Required reading before discuss-phase

1. `/home/jasper/Repositories/feynman-lookup/infra/Caddyfile`
2. `/home/jasper/Repositories/feynman-lookup/infra/feynman-lookup.service`
3. `/home/jasper/Repositories/feynman-lookup/infra/RUNBOOK.md`
4. `/home/jasper/Repositories/feynman-lookup/justfile` (deploy recipe section)
5. `[[cross-project-memory]]` — all Hetzner / Leptos-deploy entries
6. `[[cross-project-incidents]]` 2026-04-16 — feynman post-deploy WASM 404 + binary arch mismatch incidents

## Plans

TBD — produced by `/gsd-plan-phase 20`. Likely 2-3 plans:
- 20-01: infra/ files + setup-host.sh + first cold deploy
- 20-02: just deploy recipe + post-deploy smoke
- 20-03: RUNBOOK + rollback + secret rotation

## Depends on

- Phase 19 (Production Hardening) — env-driven config required
- Phase 21 (Compliance & Transparency) — domain registration; can proceed in parallel until DNS step

---

This SPEC was seeded by the pivot.
