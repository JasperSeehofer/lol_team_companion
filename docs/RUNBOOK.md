# Runbook

How to run Team Companion in production: one small VM, Docker Compose, Caddy for HTTPS, nightly encrypted backups with 30-day retention (MISSION section 4) to the owner's Hetzner Storage Box over SFTP (S3-compatible object storage works too, section 5.2). The owner holds the VM, the domain, the Riot key, the Storage Box and a second server for monitoring (section 2.7); nothing here adds a paid service beyond those.

Files: `Dockerfile`, `.dockerignore`, `compose.yaml`, `deploy/Caddyfile`, `deploy/env.production.example`, `deploy/backup.env.example`, `deploy/backup/{Dockerfile,entrypoint.sh}`, `scripts/backup.sh`, `scripts/restore.sh`, `scripts/restore-drill.md`, `scripts/smoke_signin.sh`, and for the monitoring server `scripts/healthcheck-remote.sh` and `scripts/backup-watch.sh`. What is verified where: section 12.

## Contents

1. Shape of the deployment
2. First deploy
3. Building the image
4. Updates and rollback
5. Backups (5.1 Hetzner Storage Box, 5.2 trade-offs)
6. Restore
7. Logs
8. Secrets and keys
9. Failure recovery (disk full, lock, crash loop)
10. Migrations and database versions
11. Incident checklist
12. Verification checklist

---

## 1. Shape of the deployment

```
internet --443/80--> caddy (TLS, headers, compression, access log)
                       |  http, X-Forwarded-For = client address
                       v
                     app :3020 (not published)  --- volume "companion_data" at /data
                       |                              /data/db     SurrealKV (exclusive lock)
                       |                              /data/inbox  bug-report export
                       +--> Riot API, Data Dragon (outbound HTTPS)
backup (daily, BACKUP_AT UTC): stop app -> tar /data/db,/data/inbox -> start app -> upload (rclone crypt) -> verify -> prune
```

- One app process per database. SurrealKV takes an OS lock on `/data/db/LOCK`; a second process (a second app container, the seed tool, a shell that opens the database) cannot open it. Never scale `app` beyond one replica.
- The app trusts the last `X-Forwarded-For` hop (`TRUST_FORWARDED_FOR=true`, forced in `compose.yaml`). That is only safe because the app port is not published. Do not add `ports:` to `app`.
- Recovery point: up to 24 hours (daily backups). Recovery time: about 15 minutes with a working VM, about an hour on a new VM.

## 2. First deploy

### 2.1 VM

**This deployment:** a Hetzner Cloud VM in Helsinki, IPv4 `37.27.20.30`, IPv6 block `2a01:4f9:c013:554a::/64` with the address `2a01:4f9:c013:554a::1`. Public host name `app.team-companion.eu`; certificate contact `info@team-companion.eu`. The examples below use these values.

- **Size:** 2 vCPU, 4 GB RAM, 40 GB disk is enough for the closed beta (for example Hetzner CX22/CPX21 or similar; EU region for DACH users). The app is limited to 1.5 GB and Caddy and the backup sidecar to 256 MB each.
- **Building on the VM** (section 3) needs more: the release build links a large binary and runs LTO for the WASM. Add 4 GB of swap if you build there, or build elsewhere and copy the image.
- Debian 13 or Ubuntu 24.04 LTS, automatic security updates on (`unattended-upgrades`).
- Firewall: allow 22 (preferably restricted to your IPs), 80 and 443 TCP, 443 UDP (HTTP/3). Nothing else. Docker publishes ports past `ufw`, which is fine here because only Caddy publishes any.
- SSH with keys only, root login off.
- Provider snapshots (optional, small extra cost): a weekly VM snapshot is a second line of defence. It is crash-consistent for SurrealKV, not a replacement for the cold backups.

### 2.2 Docker

```bash
# Official Docker packages (https://docs.docker.com/engine/install/)
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker "$USER"     # log out and in again
docker compose version              # Compose v2 plugin
```

Set log rotation defaults for anything not in `compose.yaml` in `/etc/docker/daemon.json`: `{"log-driver": "json-file", "log-opts": {"max-size": "10m", "max-file": "5"}}`, then `sudo systemctl restart docker`.

### 2.3 DNS

Create `A` (and `AAAA` if the VM has IPv6) records for the chosen host name pointing at the VM: here `app.team-companion.eu A 37.27.20.30` and `app.team-companion.eu AAAA 2a01:4f9:c013:554a::1`. Wait until `dig +short app.team-companion.eu` (and `dig +short AAAA app.team-companion.eu`) answers with the VM's addresses **before** the first `docker compose up`: Caddy requests the certificate on start, and repeated failures hit Let's Encrypt rate limits.

### 2.4 Checkout and settings

```bash
sudo mkdir -p /opt/companion && sudo chown "$USER" /opt/companion
git clone <repo-url> /opt/companion && cd /opt/companion
git checkout <release-tag-or-commit>

cp deploy/env.production.example .env && chmod 600 .env
cp deploy/backup.env.example deploy/backup.env && chmod 600 deploy/backup.env
```

Fill in `.env`:

| Setting | Value |
|---|---|
| `COMPANION_DOMAIN`, `ACME_EMAIL` | host name; contact for certificate notices |
| `SESSION_SECRET` | `openssl rand -hex 32` (64 hex characters; hex avoids `$`, which compose would interpolate). Required: the server refuses to start without at least 32 bytes. |
| `BOOTSTRAP_INVITE_CODE` | a random code, e.g. `openssl rand -hex 8`, with `BOOTSTRAP_INVITE_MAX_USES=1`: it is only for the admin's own registration |
| `ADMIN_EMAILS` | **leave empty for now** (see 2.6) |
| `RIOT_API_KEY` | the production key once approved (a development key works meanwhile; it expires every 24 h) |
| `RIOT_GLOBAL_BUDGET_PER_SEC` / `_PER_MIN` | defaults fit a development key; for the production key see the comment in the template (45 / 2400 for the usual 500 per 10 s, 30,000 per 10 min) |

Do not set `APP_ENV`, `CLOSED_BETA`, `COOKIE_SECURE`, `RATE_LIMIT_MULTIPLIER` or `TRUST_FORWARDED_FOR`. The defaults are the production values, and `compose.yaml` forces `APP_ENV` empty and `TRUST_FORWARDED_FOR=true`.

The server validates every variable at startup (`src/server/config.rs`): a malformed value (`TRUST_FORWARDED_FOR=ture`, `RATE_LIMIT_MULTIPLIER=1O`, a placeholder `SESSION_SECRET`, a non-https `SUPPORT_URL`) stops it with every error listed in `docker compose logs app`; risky values (the e2e settings, an empty `ADMIN_EMAILS`, a stray `GITHUB_TOKEN`, a misspelt variable name) are warnings at the top of the log. Read them after every `.env` change.

Fill in `deploy/backup.env`: block A with the Storage Box sub-account (section 5.1), and two crypt passphrases (run `rclone obscure '<passphrase>'` for each and paste the output). Store the plain passphrases and the sub-account password in the password manager: without the passphrases no backup can be decrypted.

### 2.5 Build and start

```bash
export GIT_SHA=$(git rev-parse --short HEAD)
sed -i "s/^COMPANION_TAG=.*/COMPANION_TAG=$GIT_SHA/" .env
docker compose build                 # see section 3 for time and memory
docker compose up -d
docker compose ps                    # app becomes "healthy" within ~90 s
curl -fsS https://$COMPANION_DOMAIN/healthz   # {"db":"ok","status":"ok"}
```

Do **not** run `scripts/smoke_signin.sh` with the bootstrap invite now: it registers a throwaway account, which uses up the single use of `BOOTSTRAP_INVITE_CODE` (`MAX_USES=1`) and leaves no way to register the admin. The sign-in check over HTTPS comes right after the admin registration (2.6 step 2).

Check the forwarded address once: sign in from your own connection with a wrong password, and look for your address (not Caddy's `172.x` address) being rate-limited after a few attempts. If every user shared one bucket, the login lockout would hit everyone at once.

### 2.6 Admin, invites, cleanup

1. Open `https://app.team-companion.eu/auth/register?invite=<BOOTSTRAP_INVITE_CODE>` and register the admin account.
2. Sign-in smoke test over HTTPS with that account, without registering anything (start the line with a space so the shell history does not keep the password): ` SKIP_REGISTER=1 SMOKE_EMAIL=<admin email> SMOKE_PASSWORD='<password>' ./scripts/smoke_signin.sh https://$COMPANION_DOMAIN` ends with `PASSED`.
3. Put that email into `ADMIN_EMAILS` and remove `BOOTSTRAP_INVITE_CODE` from `.env`, then `docker compose up -d app` (recreates the container with the new environment).
4. Sign out and in: the nav shows the admin entry; `/admin/invites` works. Mint invite codes there for the first teams.
5. Register the restore-drill account (`scripts/restore-drill.md`).
6. Run a backup by hand and a drill (sections 5 and 6) before inviting anyone.

### 2.7 Monitoring (minimal)

Monitoring runs from the owner's **other Hetzner server**, so it notices when this VM is down, and needs no external service. Both scripts are plain bash (with `curl`, and `rclone` or `ssh` for the backup check) and mail through `mail` or `sendmail` if the server has one (for example `apt install bsd-mailx` with a relay, or any MTA); without a mailer they only exit non-zero, and cron mails their output when `MAILTO` is set.

```bash
# on the monitoring server
sudo mkdir -p /opt/companion-watch && cd /opt/companion-watch
curl -fsSLO https://raw.githubusercontent.com/JasperSeehofer/lol_team_companion/main/scripts/healthcheck-remote.sh
curl -fsSLO https://raw.githubusercontent.com/JasperSeehofer/lol_team_companion/main/scripts/backup-watch.sh
chmod +x healthcheck-remote.sh backup-watch.sh
./healthcheck-remote.sh                 # "ok: https://app.team-companion.eu/healthz"
```

(or copy the two files from a checkout with `scp`.)

- **Uptime:** `scripts/healthcheck-remote.sh` asks `https://app.team-companion.eu/healthz` (10 s timeout) and fails on anything but HTTP 200 with `"status":"ok"` (`/healthz` answers 503 when the database query fails). With `ALERT_EMAIL` set it mails once after `FAILS_BEFORE_ALERT` (2) failed checks in a row and once when the site answers again; the count lives in `STATE_FILE` (`~/.local/state/companion-healthcheck.state`).
- **Backups (dead-man's switch):** `scripts/backup-watch.sh` alerts when the newest backup is older than `MAX_AGE_HOURS` (36: one missed night plus slack), or when it cannot find one. Two ways to look:
  - `remote` (recommended): lists the raw Storage Box folder with `rclone` and takes the newest file time. Use a second, **read-only** sub-account on the same directory (section 5.1) and no crypt settings: file names are encrypted on the box, modification times are not, so the monitoring server holds no passphrase and cannot delete anything. `RCLONE_CONFIG_WATCH_TYPE=sftp`, `_HOST`, `_PORT=23`, `_USER`, `_PASS` (obscured), `_DISABLE_HASHCHECK=true`, and `WATCH_REMOTE=watch:companion-backups`.
  - `log`: reads `docker compose logs backup` on the VM over SSH (`WATCH_SSH=<user>@37.27.20.30`; the key must be allowed to run that command) and takes the last `backup <id> done` line, which `backup.sh` writes only after the upload was downloaded again and verified. This also catches an upload that failed its verification; `remote` would count that upload.
- `HEALTHCHECK_PING_URL` in `deploy/backup.env` stays available for an external dead-man's switch (owner's choice, MISSION "ask first").
- **Disk:** alert at 80 % (provider monitoring, or a cron line on the VM: `df --output=pcent / | tail -1`).
- **Certificates:** Caddy renews by itself; the uptime check catches a failure.

**cron** (`sudo crontab -e` on the monitoring server: the lines below log to `/var/log`, which only root can write; in a user's crontab log to a file in its home instead. cron's `PATH` lacks `/usr/sbin`, the scripts look for `/usr/sbin/sendmail` themselves):

```cron
MAILTO=""
ALERT_EMAIL=info@team-companion.eu
# every 5 minutes; one mail when it goes down, one when it is back
*/5 * * * * /opt/companion-watch/healthcheck-remote.sh >> /var/log/companion-healthcheck.log 2>&1
# twice a day, after the 02:30 UTC backup and again in the evening
15 7,19 * * * . /opt/companion-watch/watch.env && /opt/companion-watch/backup-watch.sh remote >> /var/log/companion-backup-watch.log 2>&1
```

with `/opt/companion-watch/watch.env` (chmod 600) holding `export RCLONE_CONFIG_WATCH_...` lines and `export WATCH_REMOTE=watch:companion-backups`. cron's times follow the server's time zone; check it with `timedatectl`.

**systemd timers** instead of cron:

```ini
# /etc/systemd/system/companion-healthcheck.service
[Unit]
Description=Team Companion uptime check
[Service]
Type=oneshot
Environment=ALERT_EMAIL=info@team-companion.eu
Environment=STATE_FILE=/var/lib/companion-watch/healthcheck.state
ExecStart=/opt/companion-watch/healthcheck-remote.sh

# /etc/systemd/system/companion-healthcheck.timer
[Unit]
Description=Team Companion uptime check every 5 minutes
[Timer]
OnBootSec=2min
OnUnitActiveSec=5min
[Install]
WantedBy=timers.target

# /etc/systemd/system/companion-backup-watch.service
[Unit]
Description=Team Companion backup age check
[Service]
Type=oneshot
Environment=ALERT_EMAIL=info@team-companion.eu
EnvironmentFile=/opt/companion-watch/watch.systemd.env
ExecStart=/opt/companion-watch/backup-watch.sh remote

# /etc/systemd/system/companion-backup-watch.timer
[Unit]
Description=Team Companion backup age check twice a day
[Timer]
OnCalendar=*-*-* 07,19:15:00 UTC
Persistent=true
[Install]
WantedBy=timers.target
```

`watch.systemd.env` holds the same settings as `watch.env` without `export`. Then `sudo mkdir -p /var/lib/companion-watch && sudo systemctl daemon-reload && sudo systemctl enable --now companion-healthcheck.timer companion-backup-watch.timer`; `systemctl list-timers 'companion-*'` shows the next runs and `journalctl -u companion-healthcheck` the results. Test the alert path once: `HEALTH_URL=https://app.team-companion.eu/nope FAILS_BEFORE_ALERT=1 STATE_FILE=/tmp/hc-test ALERT_EMAIL=<you> ./healthcheck-remote.sh` must mail "Team Companion is DOWN".

## 3. Building the image

`docker compose build` (or `docker build -t companion:<sha> --build-arg GIT_SHA=<sha> .`) runs the multi-stage `Dockerfile`:

1. `rust:1.99.0-slim-trixie` with build packages; the toolchain is pinned with `RUSTUP_TOOLCHAIN` so `rust-toolchain.toml` (`channel = "stable"`) cannot pull a different compiler.
2. `wasm32-unknown-unknown` for that toolchain.
3. cargo-leptos 0.3.11 from the release tarball, checksum-verified (no compile).
4. `cargo leptos build --release` with BuildKit cache mounts for the cargo registry, `target/` and cargo-leptos' tool downloads (wasm-bindgen, Tailwind v4.2.1, wasm-opt).

The runtime image is `debian:trixie-slim` with `ca-certificates`, `libssl3t64` (the binary links OpenSSL through native-tls), `tini` and `curl`, a non-root user (uid 10001), the stripped binary and `target/site` (WASM, JS, CSS, fonts). `schema.surql` is compiled into the binary.

**Time and memory:** measured in CI (GitHub's 4 vCPU / 16 GB runner): a cold build takes about 20 minutes (image 231 MB uncompressed: stripped server binary 127 MB, release WASM 8.2 MB before compression, CSS 70 KB). Expect longer on a 2 vCPU VM, and peak memory of several GB during linking and the WASM LTO (not measured; add swap when building on a 4 GB VM). A rebuild after a source change, with the cache mounts warm, recompiles only the crate (CI: about 9 minutes, most of it the release link and the WASM LTO). The cache lives in BuildKit on the build host; reclaim space with `docker builder prune --filter type=exec.cachemount` when the disk fills.

**Building elsewhere** (recommended for a 4 GB VM): build on a workstation or CI, then copy the image without a registry:

```bash
docker build -t companion:$GIT_SHA --build-arg GIT_SHA=$GIT_SHA .
docker save companion:$GIT_SHA | gzip | ssh vm 'gunzip | docker load'
```

**CI:** the `image` job in `.github/workflows/ci.yml` builds both images on every push and pull request (layer cache in the GitHub Actions cache; the BuildKit cache mounts carried between runs with `reproducible-containers/buildkit-cache-dance`), runs the app image read-only with no capabilities, signs in with `scripts/smoke_signin.sh`, waits for the Docker health check, runs the smoke, hydration, auth, legal and account specs against the release build, and checks that `docker stop` exits 0 after a clean database close and that a restart on the same volume works. It pushes nothing: a registry (for example GHCR) is a new external service and needs the owner's decision.

## 4. Updates and rollback

Each deploy runs one image tag (`COMPANION_TAG`, the git short sha). Keep the previous image until the new one has run for a few days.

```bash
cd /opt/companion
git fetch && git checkout <new-tag-or-commit>
export GIT_SHA=$(git rev-parse --short HEAD)
# The shell variable wins over .env, so the build gets the NEW tag and the
# running image (old tag) is left untouched for a rollback.
COMPANION_TAG=$GIT_SHA docker compose build app backup   # app keeps running meanwhile
grep '^COMPANION_TAG=' .env | cut -d= -f2 > .previous-tag

# Backup right before the switch: the rollback point if the update migrates.
docker compose run --rm -e BACKUP_REASON=pre-update backup now

sed -i "s/^COMPANION_TAG=.*/COMPANION_TAG=$GIT_SHA/" .env
docker compose up -d                            # recreates app (and backup) with the new tag
docker compose logs -f --since 5m app           # watch "migrations applied" / errors
curl -fsS https://$COMPANION_DOMAIN/healthz
```

Downtime is the restart: a few seconds to stop (the app drains requests for up to `SHUTDOWN_DRAIN_SECS`, 10, and closes the database within `SHUTDOWN_DB_SECS`, which compose sets to 15; `docker compose stop app` exits with code 0), then startup, during which Caddy shows the "restarting" page. Migrations run at startup, before the server listens; a failing migration stops startup and the container restarts in a loop (section 10).

Why 15 s for the database: the app releases SurrealKV only after the embedded engine has stopped its background tasks. Usually that takes well under a second, but after a long busy run it was seen to take more than the 5 s default (M8 and M9 testing), and the app then exits with code 1 and "database did not close within 5s". No data is lost in that case (every commit is synced and the next start replays the log), but the exit code looks like a failure. Compose therefore sets `SHUTDOWN_DB_SECS=15`; with the 10 s drain that is 25 s, inside the 30 s `stop_grace_period`. If a stop still logs "did not close", look at the line before it: "N database handle(s) still held at shutdown" means some of the app's own handles were never released, and no waiting time fixes that. That was the case after long busy runs until the polish pass (9 held after the full browser suite): requests whose page render never finished kept handles through the Leptos context, the router state and the auth backend. Since then no request holds a strong handle (server functions borrow one per call; the auth backend and the session store keep a weak one), and a stop right after the full suite closes the database in about 0.1 s with exit 0. Should handles be held again, the log lists which request paths hold them ("N held by a server function of /path"); keep those lines for a fix. The data is safe in every case.

Read `CHANGELOG.md` for the release before updating. If it mentions a new migration, the pre-update backup is the only way back.

**Rollback:**

- **No migration in between:** set `COMPANION_TAG` back to `.previous-tag` and `docker compose up -d`.
- **The new version applied a migration:** the old image refuses the database ("the database records migration N, which this build does not know"). Restore the pre-update backup with the old image: set `COMPANION_TAG` back to `.previous-tag`, run `docker compose up -d app` (the container is recreated from the old image, which refuses the database and restarts in a loop; `restore.sh apply` restarts the container it finds, so it must already be the old one), then `restore.sh apply <id>-pre-update` (section 6). Data written since the update is lost, so decide quickly and announce it. The old image refuses the newer database before it writes anything (the version check runs before the `schema.surql` baseline since M8), so a restart loop does no harm, but it is downtime.

Clean up old images now and then: `docker image ls companion` and `docker image rm companion:<old>` (keep the current and previous tag).

## 5. Backups

**What:** a cold copy of `/data/db` (SurrealKV) and `/data/inbox`, as `companion-<UTC timestamp>.tar.zst` with a `.sha256` and a `manifest.json` (image, revision, sizes, downtime), uploaded through an rclone crypt remote to `companion-backups/companion/<id>/` on the Storage Box (names and contents encrypted; section 5.1).

**When:** daily at `BACKUP_AT` (UTC, default 02:30) by the `backup` sidecar, plus before every update.

**Why a cold copy:**

| Option | Consistent? | Downtime | Verdict |
|---|---|---|---|
| Copy the live directory (`cp`, `tar`, `rsync` while running) | No: compaction writes and deletes SSTables during the copy; the copy can mix generations | none | never |
| Stop, copy, start (chosen) | Yes: the process is gone and the files are at rest | seconds nightly (a graceful stop, the copy, startup) | simple, needs no code, restore is an untar |
| Filesystem or VM snapshot of the live volume | Crash-consistent: restoring relies on SurrealKV's WAL recovery, as after a power cut. SurrealKV syncs every commit by default (surrealdb-core 3.0.2: `SurrealKvConfig::default` is `sync_mode: Every`), so committed data should survive | none | good second line (provider snapshots), not the primary |
| Logical export from inside the app (`db.export(...)` in the SurrealDB SDK) | Yes: surrealdb-core 3.0.2 exports in one read-only transaction (`Datastore::export_with_config`) | none | best long-term: portable `.surql`, survives storage-format changes. Needs code (a scheduled export to `/data/export`, and a small `restore` binary using `db.import`). Planned for Release 2 (`docs/ARCHITECTURE.md`, "Deployment") |
| SurrealKV checkpoints (`Tree::create_checkpoint` in surrealkv 0.20.2) | Yes | none | not reachable: surrealdb 3.0.2 does not expose it |

**Trade-offs, honestly:** a nightly stop is visible to anyone using the app at 02:30 UTC. A cold copy is tied to the SurrealKV on-disk format: restore it with the same image or a newer one (the manifest records which image made it). Up to a day of data can be lost.

**Docker socket:** the sidecar mounts `/var/run/docker.sock` to stop and start the app, which is root-equivalent on the host. Alternative without it: remove the `backup` service and run the same script from the host with a systemd timer:

```ini
# /etc/systemd/system/companion-backup.service
[Service]
Type=oneshot
EnvironmentFile=/opt/companion/deploy/backup.env
Environment=DATA_SRC=/var/lib/docker/volumes/companion_data/_data STAGING_DIR=/var/backups/companion
ExecStart=/opt/companion/scripts/backup.sh
# /etc/systemd/system/companion-backup.timer
[Timer]
OnCalendar=*-*-* 02:30:00 UTC
Persistent=true
[Install]
WantedBy=timers.target
```

(the host then needs `rclone`, `zstd`, `curl`).

**By hand:** `docker compose run --rm backup now` (add `-e BACKUP_REASON=manual` to tag it).

**Check:** `docker compose logs --since 48h backup` shows `backup <id> done: ... downtime Ns`. `docker compose run --rm backup rclone lsf "$RCLONE_REMOTE/companion"` (with the env from `deploy/backup.env`) lists the remote copies.

**Retention:** after each successful upload the script deletes remote backups older than `RETENTION_DAYS` (30) but always keeps the newest `KEEP_MIN` (7), so a month of failed runs cannot erase the last good ones. The staging volume keeps only the latest local copy. On S3-compatible storage, optionally add a lifecycle rule as a backstop (longer than the script's window, for example 45 days) and object lock or versioning; a Storage Box has neither (section 5.1, trade-offs).

**Personal data:** backups contain emails, password hashes and team data, encrypted client-side by rclone crypt. Deleted accounts disappear from backups after at most 30 days; the privacy page must say so.

### 5.1 Hetzner Storage Box (this deployment)

The backups go to the owner's Storage Box over SFTP (port 23), with rclone's crypt remote on top: the box only ever sees encrypted file names and contents (backups hold emails and password hashes). **This deployment:** main account `u663454` (`u663454.your-storagebox.de`), Falkenstein (Germany), confirmed by the owner on 13 Oct 2026; the sub-accounts below are `u663454-subN`. Nothing in `backup.sh` or `restore.sh` is specific to S3 or SFTP: they use `rclone mkdir`, `lsf`, `copy`, `cat` and `purge`, which behave the same on both, and the harness runs the full backup, retention and fetch over an SFTP server (section 12, item 2).

**Set up** (Hetzner Console, the Storage Box's page; the wording of the console may differ slightly):

1. Settings: turn on **SSH support** and **External reachability** (the VM reaches the box over the internet).
2. Sub-accounts: create one for the backups with its own **base directory** (for example `companion`), SSH support and external reachability on, **not** read-only. Its login is `u123456-subN` and its host `u123456-subN.your-storagebox.de`. It sees only that directory, so the rest of the box stays out of reach of the VM.
3. Create a second sub-account on the same base directory, **read-only**, for the backup watcher on the monitoring server (section 2.7).
4. Write block A of `deploy/backup.env`: host, `PORT=23`, user, and the password obscured: `docker compose run --rm --no-deps backup rclone obscure '<password>'` (or `rclone obscure` anywhere). Keep `RCLONE_CONFIG_OFFSITECRYPT_REMOTE=offsite:companion-backups`: a folder inside the sub-account's directory.
5. Optional, instead of the password: an SSH key. `mkdir -p deploy/backup-ssh && ssh-keygen -t ed25519 -N '' -f deploy/backup-ssh/id_ed25519`, add the public key to the sub-account as Hetzner's Storage Box documentation describes (for example `ssh-copy-id -p 23 -s -i deploy/backup-ssh/id_ed25519 u123456-subN@u123456-subN.your-storagebox.de`; Hetzner's docs: port 23 needs "SSH support" on, the key is in OpenSSH format, and a sub-account reads its own `.ssh/authorized_keys` inside its base directory), set `RCLONE_CONFIG_OFFSITE_KEY_FILE=/config/ssh/id_ed25519`, leave `PASS` empty, and uncomment the `./deploy/backup-ssh:/config/ssh:ro` volume of the `backup` service in `compose.yaml`.
6. Pin the host key (recommended; rclone otherwise accepts any host key): `ssh-keyscan -p 23 u123456-subN.your-storagebox.de > deploy/backup-ssh/known_hosts`, compare the fingerprints (`ssh-keygen -lf deploy/backup-ssh/known_hosts`) with the ones Hetzner publishes (docs.hetzner.com, Storage Box, "SSH host keys"; at the time of writing the SHA256 fingerprints were `XqONwb1S0zuj5A1CDxpOSuD2hnAArV1A3wKY7Z3sdgM` for ED25519 and `oDHZqKXnoMtgvPBjjC57pcuFez28roaEuFcfwyg8O5c` for ECDSA, and a box matches exactly one of the listed keys; check the live page), set `RCLONE_CONFIG_OFFSITE_KNOWN_HOSTS_FILE=/config/ssh/known_hosts` and mount the directory as in step 5.
7. Test: `docker compose run --rm backup rclone lsd offsite:` (lists the sub-account's directory), then `docker compose run --rm backup now` must end with "remote copy verified".

`DISABLE_HASHCHECK=true` stops rclone from probing the box's restricted shell for hash commands (crypt never uses remote hashes; `backup.sh` downloads the archive again and compares its own sha256). `RCLONE_TRANSFERS=2` and `RCLONE_CHECKERS=4` keep the number of simultaneous connections low: Storage Boxes limit connections per account (check the current number in Hetzner's documentation).

**Location:** the privacy page says the backups are kept at "Hetzner Online GmbH, Falkenstein (Germany)"; the owner confirmed on 13 Oct 2026 that this box is in Falkenstein. If it is ever moved, change the "Backups" sentence in `src/pages/legal/datenschutz.rs` first.

### 5.2 Trade-offs: Storage Box versus S3 object storage

| | Storage Box (SFTP) | S3-compatible object storage |
|---|---|---|
| Encryption | rclone crypt on the VM (same) | rclone crypt on the VM (same) |
| Retention | `backup.sh` prunes after 30 days, keeps the newest 7 (same) | the same, plus an optional lifecycle rule as a backstop |
| Protection against a compromised VM | none on the box itself: the sub-account can delete what is in its directory. Limits: the sub-account sees only that directory; the read-only watcher alerts within 36 h when the newest backup disappears or goes stale | object lock or versioning (if the provider offers it) can keep deleted objects |
| Cost | already paid (the owner's box) | an extra bucket |

So the Storage Box relies on crypt (confidentiality), the script's retention (no silent loss of the last good copies) and the sub-account's narrow rights, and the watcher notices deletion. Storage Box **snapshots** (made from the main account, out of reach of the sub-account) would add protection against deletion, but they keep data longer: the privacy page promises that deleted data leaves the backups within 30 days, so a snapshot schedule needs `RETENTION_DAYS` lowered by as many days as snapshots are kept (for example 21 + 7), and the page checked. Not set up; owner's call.

## 6. Restore

Drill (non-destructive, monthly): `scripts/restore-drill.md`.

Into production (incident):

```bash
cd /opt/companion
set -a; . deploy/backup.env; set +a
./scripts/restore.sh list
sudo -E ./scripts/restore.sh drill <id>   # optional but recommended: proves the copy first
sudo -E ./scripts/restore.sh apply <id>   # asks you to type the id
```

`apply` downloads and verifies the backup, stops the app, copies the backup into the volume next to the live data (`db.restoring`, owned by uid 10001), moves the current `db/` and `inbox/` to `/data/pre-restore-<timestamp>/`, renames the copy into place, starts the app and waits for "healthy". If anything fails after the stop, the script starts the app again on whatever is in place (a failed copy leaves the old data untouched). The old data stays until you delete it (the script prints the command); delete it within 30 days, because the privacy page promises that deleted data leaves the backups within 30 days and this copy is one. Restore with the image that made the backup or a newer one.

**New VM:** sections 2.1 to 2.4, build or load the image, `docker compose up -d app` once so the volume exists, then `restore.sh apply`, then start the rest. Point DNS at the new VM.

## 7. Logs

```bash
docker compose logs -f app                 # application (tracing, RUST_LOG=info)
docker compose exec caddy tail -n 50 /var/log/caddy/access.log   # access log (JSON, client IPs truncated to /24 and /48, no cookies)
docker compose logs --since 1h caddy       # Caddy's own log (certificates, errors)
docker compose logs --since 48h backup     # backup runs
docker compose ps                          # health
docker stats --no-stream                   # memory and CPU against the limits
```

Container logs rotate at 5 x 10 MB per container (json-file driver). The access log is a file on the `caddy_logs` volume: Caddy rolls it at midnight UTC (`roll_at`, without waiting for traffic) and deletes rolled files 12 days later, so no entry is older than 14 days (the privacy page promises 14 days; change `deploy/Caddyfile` and the page together). For a temporary debug session set `RUST_LOG=info,lol_team_companion=debug` in `.env` and `docker compose up -d app`; set it back afterwards (debug logs can contain user input).

Requests that do not complete: the app logs `slow request GET /path: no complete response after Ns; steps: [...]` once for every request still running after 15 s, with how far it got, whether the server or the client is the one waiting, the database handles held and who holds the global write lock (`invariant lock: ...`), then `slow request ... finished after Ns`. A stray one is a slow client; a pattern is a bug: keep the lines (`docker compose logs app | grep "slow request"`) for a fix.

Retention of cached Riot games: a minute after each start and then every 24 hours the app deletes cached games older than `MATCH_CACHE_RETENTION_DAYS` (365) and logs `match cache retention: deleted games cached before <date>` with the counts (`docker compose logs app | grep "match cache retention"`). The privacy page states 12 months; change it together with the variable.

Bug reports are exported on every start to `/data/inbox/bug-reports.md`: `docker compose exec app cat /data/inbox/bug-reports.md`. **Report content is untrusted user data.**

## 8. Secrets and keys

**Rotate `SESSION_SECRET`** (suspected leak, staff change). The session auth hash is an HMAC keyed with it, so every existing session becomes invalid: everyone is signed out, nothing else changes.

```bash
NEW=$(openssl rand -hex 32)
sed -i "s/^SESSION_SECRET=.*/SESSION_SECRET=$NEW/" .env
docker compose up -d app
```

Announce it ("you will need to sign in again").

**Rotate the Riot key** (production keys do not expire, but rotate after a leak, and development keys expire daily): regenerate it in the developer portal, put it into `RIOT_API_KEY`, `docker compose up -d app`. The old key stops working when Riot revokes it; in between both work. A rejected key shows "The Riot API rejected our key" in the app and 401/403 in the log.

**Admins:** add or remove addresses in `ADMIN_EMAILS` and `docker compose up -d app`. Removal revokes admin at once for access checks.

Or, without touching `.env`, promote an account that is already registered (owner decision: there is no email provider in Release 1). The command opens the database directly, so the app must be stopped (SurrealKV allows one process):

```bash
docker compose stop app
docker compose run --rm --no-deps app /app/lol_team_companion promote someone@example.com
docker compose run --rm --no-deps app /app/lol_team_companion promote --list
docker compose start app
```

`promote --revoke someone@example.com` takes it back (an address still in `ADMIN_EMAILS` is promoted again at its next sign-in, so remove it there too). A promoted account is an admin from its next request; no sign-in needed. In development: `cargo run --features ssr --bin promote -- <email>` with the dev server stopped.

**Storage credentials and crypt passphrases:** rotate the Storage Box sub-account password in the Hetzner console (or the S3 access key) and in `deploy/backup.env` (obscured again with `rclone obscure`); for an SSH key, replace the key on the sub-account and in `deploy/backup-ssh/`. Changing the crypt passphrases makes old backups unreadable with the new remote: keep the old passphrases until the last old backup has aged out (30 days), or start a new folder (`RCLONE_CONFIG_OFFSITECRYPT_REMOTE`).

## 9. Failure recovery

**Disk full.** Symptoms: write errors in the app log, `/healthz` 503, backups failing with "not enough space".

1. `df -h /` and `docker system df` to see where the space went.
2. Usually build cache or old images: `docker builder prune` (or `--filter type=exec.cachemount`), `docker image prune`, remove old `companion:<tag>` images.
3. The staging volume: `docker volume inspect companion_backup_staging`; it holds one local backup plus failed runs (`failed-*`, cleaned after 7 days). Safe to empty.
4. Logs are capped (5 x 10 MB per container); journald: `journalctl --vacuum-size=200M`.
5. Restart the app if it does not recover by itself (`docker compose restart app`). If SurrealKV reports corruption after a full disk, restore the last backup (section 6).

**"Database is locked" / lock errors at startup.** SurrealKV uses an OS file lock (`flock`) on `/data/db/LOCK`; the file's content (a PID) is informational and a leftover file after a crash does not block anything (surrealkv 0.20.2 `lockfile.rs`). A lock error therefore means another process really has the directory open:

1. `docker ps -a --filter volume=companion_data` shows every container using the volume. Common causes: a second app container from a manual `docker run`, a `restore.sh` helper still running, or a drill pointed at the production volume by mistake.
2. Stop the other process. Do not delete `LOCK` by hand; it is not the cause.
3. Never put `/data` on NFS or another network filesystem: `flock` is not reliable there.

**Crash loop** (`docker compose ps` shows `restarting`): `docker compose logs --tail 100 app`.

- "Configuration error: ..." (one line per problem, then "N configuration error(s); the server does not start") → fix `.env`.
- "Database initialisation failed: migration N (...) failed" → section 10.
- "... which this build does not know" → an older image on a newer database; deploy the newer image or restore (section 4, rollback).
- Out of memory (exit 137, `docker inspect` shows `OOMKilled`) → raise the app memory limit in `compose.yaml`; check `SURREAL_SURREALKV_BLOCK_CACHE_CAPACITY`.

**Caddy cannot get a certificate:** `docker compose logs caddy`. DNS not pointing at the VM, ports 80/443 blocked, or the Let's Encrypt rate limit (wait an hour; do not restart in a loop).

**Riot outage or rate limiting:** the app shows "Riot is busy" or the feature errors; nothing to do on the server. Check https://developer.riotgames.com status and the portal's rate-limit page.

## 10. Migrations and database versions

- Every start checks that the build knows every recorded migration, applies the `schema.surql` baseline (idempotent) and then every migration not yet recorded in `schema_migration`, in order. A SurrealQL migration runs in one transaction with its record (all or nothing); a Rust migration is idempotent and recorded after it succeeds. Startup stops on the first failure, so the server never serves a half-migrated database.
- **A failed migration** leaves the database at the last good version (the failed SurrealQL migration rolled back; a Rust migration may be partly done but is safe to rerun). The container restarts in a loop and retries. Fix: deploy the previous image (no migration of the failed version was recorded, so it starts) and report the error, or restore the pre-update backup.
- **A newer-build database** (records a migration this build does not know) is refused at startup with "the database records migration N, which this build does not know; it was migrated by a newer version of the app". The check runs before the `schema.surql` baseline (M8, test `a_newer_database_is_refused_before_the_baseline_is_applied`), so the older image writes nothing into the newer database. Upgrade, or restore a backup made by the older image.
- Migrations are forward-only; there are no down migrations. The rollback for a migrating release is the pre-update backup.

## 11. Incident checklist

1. **Stabilise:** is the site up (`/healthz`)? If the app is broken and the cause is unclear, restart it once (`docker compose restart app`); do not restart in a loop.
2. **Preserve:** before changing anything, take a manual backup (`docker compose run --rm -e BACKUP_REASON=incident backup now`) and save logs: `docker compose logs --since 24h > incident-$(date -u +%F).log`.
3. **Communicate:** a short note to the teams (Discord): what is affected, when the next update comes.
4. **Diagnose:** app log, Caddy log, `docker compose ps`, `docker stats`, `df -h`. Check recent changes (last deploy, `.env` edits).
5. **Fix or roll back:** section 4 (rollback), section 6 (restore), section 9 (recovery).
6. **Security incidents** (suspected account compromise, leaked secret, unexpected admin): rotate `SESSION_SECRET` (signs everyone out), rotate the Riot key and the Storage Box password or key if they could be exposed, check `ADMIN_EMAILS` and `/admin/invites` for unknown codes and disable them. A breach of personal data has a 72-hour notification duty to the supervisory authority (GDPR Art. 33); the owner decides and reports.
7. **Verify:** `/healthz`, sign in, open a draft and a game plan, check the next backup runs.
8. **Write it down:** what happened, impact, timeline, cause, what changes. Add an entry to `CHANGELOG.md` if users noticed.

## 12. Verification checklist

The thirteen checks from the M8 preparation, with where each one is proven. "CI" means the `image` or `scripts` job in `.github/workflows/ci.yml` on every push; "dev" means checked by hand in the development container (no Docker daemon there); "VM" means the owner checks it on the first deploy (it needs Docker, a domain or real storage).

| # | Check | Where | State (M8) |
|---|---|---|---|
| 1 | Files in place, scripts executable, `.gitignore` covers `deploy/backup.env`, `.previous-tag`, `restore-drills.log`, `restore-*/` | repo | done |
| 2 | ShellCheck clean; ops harness passes (43 checks: stop/start order, restart on failure, no half backups, upload through an rclone crypt remote, download-and-compare, retention keeps the newest 7, fetch + sha256 + extract, drill pass and fail, cleanup; the same backup, retention and fetch over SFTP (`rclone serve sftp` as the Storage Box) with only encrypted names on the server; `healthcheck-remote.sh` ok/fail/timeout and one mail per state change, and with an unusable state file a healthy site still passes and a failure still mails; `backup-watch.sh` fresh/stale/empty storage and fresh/old/unreadable log) | CI `scripts`, dev | passing |
| 3 | Image builds with BuildKit; time and size recorded; cache mounts reused | CI `image` | passing: cold build about 20 min, warm about 9 min, image 231 MB uncompressed: stripped server binary 127 MB, release WASM 8.2 MB before compression, CSS 70 KB |
| 4 | Image contents: uid 10001, `pkg/` with `.wasm`/`.js`/`.css`, fonts, no missing shared libraries | CI `image` ("Inspect the image") | passing |
| 5 | Runs bare in production mode, read-only root, no capabilities; `/healthz`; Docker health check healthy; Playwright smoke, hydration, auth, legal and account specs against the release build; sign-in over HTTP | CI `image` | passing |
| 6 | Compose stack: HTTPS, security headers, zstd for `/pkg/*.wasm`, the "restarting" page while the app is stopped | CI validates `compose.yaml` and the Caddyfile (`caddy validate`); the live checks need a domain | VM, on first deploy (section 2.5) |
| 7 | Rate limiting sees client addresses through Caddy, not Caddy's | needs the stack | VM (section 2.5) |
| 8 | `docker compose run --rm backup now` ends with "remote copy verified"; the nightly loop fires | needs the Storage Box | VM (sections 5, 5.1 and 2.6) |
| 9 | Restore drill against a real backup (`DRILL PASSED`, "authenticated read ok"); the drill's sign-in check matches the real responses | the sign-in check is the same request sequence as `scripts/smoke_signin.sh`, which CI runs against the image | VM (`scripts/restore-drill.md`) |
| 10 | `restore.sh apply` rehearsal | needs a stack | VM or staging, before launch |
| 11 | Typed config: a typo stops the server with every error listed; the CI environment starts with warnings only | unit tests in `config.rs`; dev (the debug binary in production mode with `TRUST_FORWARDED_FOR=ture`, `RATE_LIMIT_MULTIPLIER=1O`, a bad `ADMIN_EMAILS`: four errors, no start) | done |
| 12 | Graceful shutdown: clean stop exit 0; drain bound; immediate restart without a lock error | dev (SIGTERM/SIGINT: about 0.1 to 1.3 s, exit 0, "database closed"; a slow download held open: warning after 10 s, exit 1 after 15 s; restart at once without a lock error); CI (`docker stop` exit 0 with "database closed", restart on the same volume and sign in again) | done |
| 13 | Docs: CLAUDE.md, ARCHITECTURE.md, CHANGELOG.md | repo | done |
