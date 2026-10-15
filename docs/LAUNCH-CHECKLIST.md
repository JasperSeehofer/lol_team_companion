# Launch checklist

Everything to tick before the first outside team gets an invite code (M8 gate). Each line names who does it: **owner** (holds the VM, domain, keys, legal responsibility), **agent** (code and docs), or **both**. Do not skip a line; write "n/a because ..." instead. State after the M8 build (12 Oct 2026): agent lines that are done are ticked with their evidence; everything else is open.

## 1. Identity and legal

- [x] **owner** Final product name chosen: no "League of Legends", "LoL", "Riot" or Riot IP (MISSION 4). Name updated in the shell `<title>`, footer disclaimer (`RIOT_DISCLAIMER` keeps Riot's exact wording with the new name), nav, Impressum, privacy page, `riot.txt` application. *Done: "Team Companion" (owner, 12 Oct 2026); applied everywhere in the polish pass (`smoke.spec.ts` checks the title and the nav brand, a unit test the disclaimer).*
- [x] **owner** Domain registered; `COMPANION_DOMAIN` set; HTTPS works. *Domain `app.team-companion.eu`, certificate contact `info@team-companion.eu` (owner, 12 Oct 2026; set as the example values in `deploy/env.production.example`). Done 13 Oct 2026: A and AAAA records point at the VM, Let's Encrypt certificate issued on the first start, HTTPS answers over IPv4 and IPv6.*
- [x] **owner + agent** Impressum (`/legal/impressum`) is real: name, postal address, email, and whatever else § 5 DDG requires for the owner's situation. No placeholders. *Done (polish pass): the owner's facts (private person: name, address, email; no second contact, no responsible-for-content line, no registration or VAT ID); `legal.spec.ts` checks there is no placeholder.*
- [ ] **owner + agent** Privacy page (`/legal/datenschutz`) is real. *Agent part done (M8):* it lists what is stored (the list is checked against `schema.surql` in both directions by `every_table_is_accounted_for`), purposes, Riot and Data Dragon, the hosting and backup providers (placeholders), the one session cookie, the access and application logs, rate limiting in memory, retention (backups 30 days), rights with the export and deletion links. *Polish pass:* controller, privacy contact, hosting (Hetzner Helsinki) and backup storage (Hetzner Falkenstein), access logs 14 days (Caddy rolls and deletes to match) and the supervisory authority (LfDI Baden-Württemberg) are filled in. *Review:* the legal basis per purpose is filled in with drafted defaults (service: contract; Riot link, syncs, share links, bug reports and votes: the user's own action under the contract; scouting opponents, logs, rate limits and backups: legitimate interest), flagged by the callout. *Owner, still open:* the adviser confirms the legal bases (and the two drafted terms clauses), data processing agreements with Hetzner in place (not claimed on the page), application-log retention (today: size-rotated, 5 x 10 MB); then remove the "Partly a draft" callout.
- [ ] **owner + agent** Terms of use page exists. *Agent part done (M8): `/legal/terms`, linked in the footer. Polish pass: the owner's pricing statement word for word; liability and governing law drafted for the owner and on the page.* *Owner:* the adviser confirms the two drafted paragraphs (section "Liability and law").
- [x] **agent** Account deletion works end to end: profile "Delete my account" (username + password), personal rows deleted, team records kept under "Former member", sessions ended; a leader with members hands over first (team page "Make team leader"), a leader alone deletes the team. Evidence: `tests/integration/account.rs`, `security.rs::hand_over_and_export_stay_inside_the_tenant`, `e2e/tests/account.spec.ts`.
- [x] **agent** Bug reports survive account deletion, anonymised (owner's decision, 12 Oct 2026): moved to "Former member", the reporter's text, element, window size and read marker removed, an id in the page address cut off (own player card, share link, game); the privacy page says so. Evidence: `account.rs::anonymised_rows_lose_what_their_author_wrote`, `tests/integration/account.rs::bug_reports_survive_deletion_anonymised`.
- [x] **agent** Cached Riot games are deleted after 12 months (owner's decision, 12 Oct 2026; nightly task, `MATCH_CACHE_RETENTION_DAYS`); stated on the privacy page and in `docs/RIOT-REGISTRATION-CHECK.md`. Evidence: `tests/integration/retention.rs`.
- [x] **agent** Account export works: `GET /account/export` (profile "Download my data"). Evidence: `export_holds_every_section_and_nothing_of_others`, `account.spec.ts`.
- [ ] **owner** Support page and money wording follow the money policy (MISSION 4) and the owner's confirmation after the Riot and tax questions; or the support link stays off for launch. *Owner, 12 Oct 2026: off for launch (`SUPPORT_URL` unset); support shown gross with payment fees as a cost line.*
- [x] **agent** Footer shows Riot's legal disclaimer on every page (unit test on the wording; `smoke.spec.ts` and `legal.spec.ts` check it renders).

## 2. Riot

- [x] **owner** The existing Riot registration compared with `docs/RIOT-REGISTRATION-CHECK.md` and updated where it differs; production key approved (if not already), key in `RIOT_API_KEY`. *Done 14 Oct 2026: owner pasted the text and answers into the Riot portal and the checks passed (owner answered "done"). 13 Oct 2026: key live on the VM; the description rewritten and shortened to the portal's field limit (about 1,450 characters, with monetisation, volume and retention as its last paragraph; check doc, section 7); the owner agreed to the text and the three answers and pastes them into the portal.*
- [x] **owner + agent** `riot.txt` with the verification code from the portal served at `https://app.team-companion.eu/riot.txt`. *Agent part done: `public/riot.txt` is served (checked by `legal.spec.ts`).* *Owner, 13 Oct 2026: the portal shows no verification section for app 834192, so there is no code; the file names the app instead (ships with the next image).*
- [x] **owner** `RIOT_GLOBAL_BUDGET_PER_SEC` / `_PER_MIN` set for the production limits shown in the portal (template comment: 45 / 2400 for 500 per 10 s and 30,000 per 10 min). *Done 13 Oct 2026: key in `.env`, budget 45 / 2400, the startup log shows `riot_key=true`.*
- [x] **both** A linked account's match history, a match detail with timeline, ranked data and an opponent scouting lookup work in production. *Done 13 Oct 2026: match history, a match detail with timeline, the ranked standing on `/me` and an opponent lookup all work with the production key (three bug reports filed on the detail page, among them recall events shown as item undo; product work for the next session).*

## 3. Deployment and configuration

- [x] **owner** VM per `docs/RUNBOOK.md` 2.1: firewall (22 restricted, 80, 443), SSH keys only, unattended upgrades. *Done 13 Oct 2026: Ubuntu 24.04, `ufw` with 22/tcp, 80/tcp, 443/tcp, 443/udp; SSH password login off, root with a key only; unattended security upgrades; 2 GB swap; Docker 29 from Docker's repository with log rotation.* Port 22 is open to every address (restrict it in the Hetzner firewall if wanted).
- [x] **owner** `.env` and `deploy/backup.env` are `chmod 600`, not in git, and backed up in the password manager (with the rclone crypt passphrases, the Storage Box sub-account passwords and `SESSION_SECRET`). *Done 13 Oct 2026: both files are `600` and untracked; the password manager holds the two crypt passphrases, `SESSION_SECRET`, both sub-account passwords, and copies of `.env`, `deploy/backup.env`, `compose.override.yaml`, `deploy/backup-ssh/` (key and pinned host key), the watcher's `watch.systemd.env` from the monitoring server and the laptop's drill `backup.env`, so a new VM, monitoring server or drill machine can be rebuilt from the vault alone.*
- [x] **owner** `SESSION_SECRET` from `openssl rand -hex 32`; `APP_ENV` unset; `RATE_LIMIT_MULTIPLIER` unset (the e2e value 100 must never reach production); `CLOSED_BETA` unset. *Done 13 Oct 2026: generated on the VM; none of the three is set.*
- [x] **both** The startup log line `security configuration` shows `dev=false closed_beta=true cookie_secure=true admin_emails=1` (or the intended count). *Done 13 Oct 2026: exactly that, plus `trust_forwarded_for=true rate_limit_multiplier=1 riot_key=true`.*
- [x] **both** The response headers in production show HSTS, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`, the CSP report-only header, and the session cookie with `Secure; HttpOnly; SameSite=Strict`. *Done 13 Oct 2026: headers checked with curl; the cookie flags checked by the owner in the browser after signing in.*
- [ ] **both** CSP: a full browser walkthrough (every page, sign-in, draft board, dialogs) shows no CSP violations in the console; then switch `Content-Security-Policy-Report-Only` to `Content-Security-Policy` in `deploy/Caddyfile`.
- [x] **both** Rate limiting sees real client addresses (RUNBOOK 2.5 check), not Caddy's. *Done 13 Oct 2026, after a fix: IPv6 visitors arrived as `172.18.0.0` because the compose network had no IPv6; `enable_ipv6: true` on the `web` network (this PR) fixed it, and the access log shows the client's own address over both.*
- [x] **agent** Graceful shutdown: SIGTERM drains for up to 10 s, closes the database, exits 0. Evidence: CI `image` job ("Graceful stop, restart on the same volume"), RUNBOOK section 12 item 12.
- [x] **agent** Typed config: a typo in a boolean or number fails startup with every error listed (`config.rs` tests, RUNBOOK section 12 item 11).

## 4. Accounts and access

- [x] **owner** Admin account registered with the bootstrap invite **before** its email went into `ADMIN_EMAILS` (no email verification exists: listing an unregistered address lets whoever registers it first become admin). *Done 13 Oct 2026 in that order; the log shows `admin_emails=1` and the owner sees the admin pages.*
- [x] **owner** `BOOTSTRAP_INVITE_CODE` removed from `.env` after the admin registered; the bootstrap code is disabled or used up on `/admin/invites`. *Done 13 Oct 2026: removed (with `_MAX_USES`); its single use went to the admin registration.*
- [ ] **owner** Invite codes minted on `/admin/invites`, one per team, with a sensible use count (team size plus coach plus one) and an expiry.
- [ ] **owner** Restore-drill account registered (no team, not admin).
- [ ] **owner** A second person (or a sealed note) can reach the VM and the password manager if the owner is unavailable.

## 5. Backups (verified, not assumed)

- [x] **owner** Hetzner Storage Box ready (owner's decision, 12 Oct 2026; RUNBOOK 5.1): SSH support and external reachability on; a backup sub-account limited to its own directory and a read-only sub-account on the same directory for the watcher; block A of `deploy/backup.env` filled in (password obscured, or an SSH key), host key pinned. *Owner, 13 Oct 2026: the box is `u663454.your-storagebox.de` in Falkenstein (Germany), which matches the privacy page; SSH support and external reachability are on, both sub-accounts exist (backups: read-write `u663454-sub1`; watcher: read-only; home directory `companion`). Done the same day on the VM: block A with an SSH key (`deploy/backup-ssh/`, mounted through an untracked `compose.override.yaml`), the host key pinned and its fingerprints matched against Hetzner's, `rclone lsd offsite:` lists the box.* No object lock or versioning on a Storage Box: the trade-off is written down in RUNBOOK 5.2.
- [x] **both** First manual backup ran: `docker compose run --rm backup now` ends with `backup <id> done` and "remote copy verified". *Done 13 Oct 2026: `20261013T094926Z-manual`, 54 KB, 1 s downtime, verified; the sidecar is running with its next run at 02:30 UTC.*
- [ ] **both** The nightly run happened at least twice (`docker compose logs --since 72h backup`).
- [ ] **both** Restore drill passed on the VM **and** once on another machine (`scripts/restore-drill.md`); results recorded. *Done 13 Oct 2026: VM (`20261013T103138Z-pre-drill`, PASSED, 4 s) and the owner's laptop (same backup, PASSED, 3 s, read with the read-only sub-account); both recorded in `scripts/restore-drill.md`.*
- [x] **both** `restore.sh apply` rehearsed once on a staging VM or a copy of the stack (never first-time in an incident). *Done 13 Oct 2026 on a throwaway compose project on the owner's laptop: restored, healthy in 7 s, old data kept beside it, the owner signed in; recorded in `scripts/restore-drill.md`.*
- [x] **owner** Crypt passphrases retrieved from the password manager and tested by the off-VM drill. *Done 13 Oct 2026: the laptop drill decrypted the backup with the passphrases typed from the password manager.*

## 6. Monitoring and logs

- [x] **owner** Uptime check on `https://app.team-companion.eu/healthz` from the second Hetzner server, alerting by email: `scripts/healthcheck-remote.sh` by cron or systemd timer every 5 minutes (RUNBOOK 2.7), alert path tested once. *Agent part done: the script, cron and systemd examples, harness checks. 13 Oct 2026: the systemd timer runs every 5 minutes on the second server (`brain`, scripts from v1.2 in `/opt/companion-watch`, state in `/var/lib/companion-watch`) and reports ok. Done: the alerting path is live since M138 (`docs/ALERTING.md`); first real alert: none yet.*
- [ ] **owner** Backup dead-man's switch: `scripts/backup-watch.sh remote` twice a day from the second server with the read-only sub-account (RUNBOOK 2.7), or `HEALTHCHECK_PING_URL`. *Agent part done: the script (alerts after 36 h), examples, harness checks. 13 Oct 2026: runs on `brain` at 07:15 and 19:15 UTC with the read-only `u663454-sub2` (password obscured, host key pinned) and reports the newest backup; delivery of an alert waits for the owner's application there (`docs/ALERTING.md`).*
- [ ] **owner** Disk usage alert at 80 %.
- [ ] **both** Error log review: run the app for a day with real use and read `docker compose logs app | grep -E "ERROR|WARN"`. No recurring errors; no passwords, session ids or full emails in the log at `RUST_LOG=info` (grep for `@` and for known test emails). Fix what shows up.
- [x] **both** The bug-report widget works in production and reports land in `/data/inbox/bug-reports.md` after a restart. *Done 13 Oct 2026: three reports from the owner exported on the next restart.*

## 7. Performance smoke

- [ ] **both** Cold page loads on a throttled connection (Chrome "Fast 4G"): home, sign-in, team dashboard, draft. Note the WASM size (`/pkg/lol_team_companion.wasm`, compressed transfer size in the network panel) and time to interactive. Budget proposal: under 3 MB compressed WASM, interactive under 4 s on Fast 4G.
- [ ] **both** Compression works: `curl -sI -H 'Accept-Encoding: zstd, gzip' https://app.team-companion.eu/pkg/lol_team_companion.wasm` shows `Content-Encoding`.
- [ ] **both** Caching: `/pkg/*` revalidates (`304` on reload), `/img/*` served from cache on the second load, `/fonts/*` cached.
- [ ] **both** A small load test against the VM (for example `hey -z 60s -c 20 https://app.team-companion.eu/` and the same for `/healthz`): no errors, p95 under 500 ms, app memory stays well under its 1.5 GB limit (`docker stats`).
- [ ] **both** Phone width walkthrough of plans, drafts, reviews (read-only views, MISSION 4).

## 8. Product readiness (Release 1 definition of done, MISSION 6)

- [ ] **agent** Every DoD item 1 to 9 checked against the code, with evidence (CI run, screenshots, test names) on the M8 demo page.
- [ ] **agent** CI green on the release commit: fmt, clippy both targets, unit and integration tests, design lint, e2e suites, the container image job and the ops scripts job.
- [ ] **agent** `CHANGELOG.md` has a release entry (the M8 entry is under Unreleased; the version number is the owner's call at the gate); `CLAUDE.md` and `docs/ARCHITECTURE.md` describe the container, config and shutdown as built (done in M8).
- [ ] **owner** Gate: the owner's go for launch (MISSION 5, M8 is a STRONG gate). Merge to main, tag the release, deploy that tag.

## 9. Invite the first teams

- [ ] **owner** Discord server ready, with a short "how to report a bug" note.
- [ ] **owner** Each team lead gets their invite link (`https://app.team-companion.eu/auth/register?invite=<code>`) and a two-line note: what the closed beta is, how to report bugs, that data may be reset only with notice.
- [ ] **both** First week: check logs and bug reports daily; first backup drill after launch within 30 days.
