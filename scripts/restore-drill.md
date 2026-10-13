# Restore drill

A backup counts only once it has been restored. This drill restores a backup into a scratch directory, starts a throwaway container on it, and checks that the app is healthy and that a real account can sign in. It never touches the production volume.

**When:** before inviting the first teams (launch gate), then monthly, after every SurrealDB or image base upgrade, and after any change to `backup.sh`, `restore.sh` or the compose file.

**Where:** on the VM (fast, same image), and at least once a quarter on another machine (proves the backup and the rclone crypt passphrases work without the VM).

## One-time setup

1. Create the drill account in production: register `drill@<your-domain>` with an invite code, a long random password, no team, never an admin. Store the password in the password manager next to the rclone crypt passphrases.
2. On the machine that runs the drill: Docker, `rclone` (1.6x or newer), `curl`, `zstd`, `tar`.
3. The image: on the VM it is already there. Elsewhere, build it from the git revision in the backup's `manifest.json` (`image_revision`), or copy it: `docker save companion:<tag> | ssh other-host docker load`.
4. A `deploy/backup.env` with the same `RCLONE_CONFIG_*` values (read-only credentials are enough for a drill: on the Storage Box, the read-only watcher sub-account of RUNBOOK 5.1 with the crypt passphrases).

## Run it

```bash
cd /opt/companion                         # the deploy checkout
set -a; . deploy/backup.env; set +a        # RCLONE_* settings
# Storage Box with an SSH key (RUNBOOK 5.1 steps 5 and 6): backup.env names
# the key and known_hosts as the sidecar sees them (/config/ssh/...); on the
# host they are under deploy/backup-ssh/, so override the two paths:
export RCLONE_CONFIG_OFFSITE_KEY_FILE=$PWD/deploy/backup-ssh/id_ed25519
export RCLONE_CONFIG_OFFSITE_KNOWN_HOSTS_FILE=$PWD/deploy/backup-ssh/known_hosts
export DRILL_EMAIL=drill@your-domain DRILL_PASSWORD='...'
sudo -E ./scripts/restore.sh list          # backups, oldest first
sudo -E ./scripts/restore.sh drill latest  # or a specific id
```

`sudo` keeps file ownership from the archive (uid 10001, the image's user). Without root the script runs the container as your own user instead.

What it does:

1. Downloads `<id>/` (archive, `.sha256`, `manifest.json`) through the crypt remote.
2. Checks the sha256 twice (manifest and `.sha256` file) and extracts `db/` and `inbox/` into a fresh `mktemp` directory.
3. Starts `companion-drill-<id>` from the image recorded in the manifest (or `companion:latest`, or `DRILL_IMAGE`), bound to `127.0.0.1:3999`, read-only root, a random `SESSION_SECRET`, `COOKIE_SECURE=false`, no Riot key, no admin list, no bootstrap invite.
4. Waits for `/healthz` to answer `{"db":"ok"}` (the database opened, the baseline and migrations ran).
5. Reads the sign-in form's server-function URL from `/auth/login`, posts the drill credentials, and expects a session cookie plus a redirect to Home (`/`). If it finds the `get_current_user` URL in the WASM it also reads the account back with the new session.
6. Removes the container and the scratch directory (`DRILL_KEEP=1` keeps both for inspection) and appends a line to `restore-drills.log`.

## Record

Copy the line from `restore-drills.log` into the table below (commit it with the next docs change) and note anything odd: slow start, warnings in `docker logs`, a manual step.

| Date (UTC) | Backup id | Image | Where | Result | Duration | Notes |
|---|---|---|---|---|---|---|
| 2026-10-13T10:53Z | 20261013T103138Z-pre-drill | companion:2f004de | owner's laptop | PASSED | 3s | Off-VM: read with the read-only watcher sub-account and the passphrases from the password manager (`~/.config/companion-drill/backup.env`, kept for the quarterly drill), image copied from the laptop build. |
| 2026-10-13T10:54Z | 20261013T103138Z-pre-drill | companion:2f004de | owner's laptop, rehearsal stack | `apply` PASSED | 7s | `restore.sh apply` against a throwaway compose project (`companion-rehearsal`, app only): stop, swap, restart, healthy; old data kept as `pre-restore-<ts>`; the owner signed in on the restored stack. Stack removed afterwards. |
| 2026-10-13T10:39Z | 20261013T103138Z-pre-drill | companion:2f004de | VM | PASSED | 4s | First drill. The first attempt failed with "Permission denied" on `/data`: as root the script left the scratch data directory root-only (fixed in `restore.sh` the same day). Without the two path overrides above, `rclone` could not find the key file. |

## If it fails

- **sha256 mismatch:** the object was damaged in storage or in transit. Try the previous backup; open an incident if two in a row fail.
- **Cannot decrypt / garbage file names:** wrong crypt passphrases. They must be the plain passphrases from the password manager, passed through `rclone obscure`.
- **Not healthy within 180 s:** read the container log the script prints. "the database records migration N, which this build does not know" means the drill image is older than the backup; use the image from `manifest.json`. A lock error means a second process used the same scratch directory.
- **Login fails:** check the drill account still exists and the password is current. A rate-limit message means more than a few attempts in a row; wait a minute.
- **Healthy and login ok, but data looks wrong:** keep the scratch copy (`DRILL_KEEP=1`), sign in through a browser on `http://127.0.0.1:3999` (through an SSH tunnel on the VM) and compare with production.

## What the drill does not prove

- That the newest data is in the backup: backups are daily, so up to 24 hours of changes can be lost (the recovery point).
- That a restore into production works end to end: `restore.sh apply` is exercised only in a real incident or on a staging VM. Rehearse it there once before launch.
- Riot-backed features: the drill runs without a key.
