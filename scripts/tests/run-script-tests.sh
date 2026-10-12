#!/usr/bin/env bash
# Offline tests for scripts/backup.sh, scripts/restore.sh,
# scripts/healthcheck-remote.sh and scripts/backup-watch.sh: a fake docker CLI,
# a local rclone crypt remote, the same crypt remote on an SFTP server
# (`rclone serve sftp`, standing in for a Hetzner Storage Box), a fake sendmail
# and a mock HTTP app (scripts/tests/mock_app.py, which imitates the endpoints
# the drill touches; it is NOT the real app).
#   scripts/tests/run-script-tests.sh scripts
# Needs: bash, rclone, python3, curl, tar, gzip, sha256sum.
set -Eeuo pipefail
SCRIPTS="$(cd "${1:?scripts dir}" && pwd)"
HERE="$(cd "$(dirname "$0")" && pwd)"
T="$(mktemp -d)"
cleanup() {
  local f
  for f in mock.pid sftp.pid health.pid; do
    kill "$(cat "$T/$f" 2>/dev/null)" 2>/dev/null || true
  done
  rm -rf "$T"
}
trap cleanup EXIT
mkdir -p "$T/bin" "$T/data/db/wal" "$T/data/inbox" "$T/staging" "$T/crypt" "$T/scratch" "$T/sftproot" "$T/hstate"
command -v rclone >/dev/null || { echo "rclone not on PATH"; exit 1; }

cat > "$T/bin/docker" <<FAKE
#!/usr/bin/env bash
echo "docker \$*" >> "$T/docker.log"
case "\$1" in
  ps) echo abc123 ;;
  image) exit 0 ;;
  inspect)
    case "\$3" in
      *State.Health*) echo healthy ;;
      *State.Running*) cat "$T/state" ;;
      *Config.Image*) echo companion:test ;;
      *revision*) echo f20a59d ;;
      *) echo sha256:deadbeef ;;
    esac ;;
  stop) echo false > "$T/state" ;;
  start) echo true > "$T/state" ;;
  run)
    port=\$(printf '%s\n' "\$@" | grep -oE '^127\.0\.0\.1:[0-9]+' | cut -d: -f2)
    vol=\$(printf '%s\n' "\$@" | grep -E ':/data\$' | sed 's#:/data##')
    [ -d "\$vol/db" ] || { echo "no db in \$vol" >&2; exit 1; }
    nohup python3 -I "$HERE/mock_app.py" "\$port" >/dev/null 2>&1 & echo \$! > "$T/mock.pid"; sleep 0.7 ;;
  exec) echo /api/get_current_user987 ;;
  rm) kill "\$(cat "$T/mock.pid")" 2>/dev/null; exit 0 ;;
  logs) echo "(fake logs)" ;;
  *) echo "fake docker: unsupported \$*" >&2; exit 1 ;;
esac
FAKE
chmod +x "$T/bin/docker"; echo true > "$T/state"
head -c 200000 /dev/urandom > "$T/data/db/wal/00001.wal"; echo 1 > "$T/data/db/LOCK"; echo "# inbox" > "$T/data/inbox/bug-reports.md"

# Fake sendmail: appends each mail to $T/mail.log.
printf '#!/bin/sh\n{ cat; echo ---; } >> "%s/mail.log"\n' "$T" > "$T/bin/sendmail"; chmod +x "$T/bin/sendmail"

export PATH="$T/bin:$PATH" RCLONE_CONFIG=/dev/null RCLONE_CACHE_DIR="$T/rclone-cache"
export RCLONE_CONFIG_OFFSITE_TYPE=local RCLONE_CONFIG_OFFSITECRYPT_TYPE=crypt RCLONE_CONFIG_OFFSITECRYPT_REMOTE="offsite:$T/crypt"
RCLONE_CONFIG_OFFSITECRYPT_PASSWORD="$(rclone obscure testpass)"; export RCLONE_CONFIG_OFFSITECRYPT_PASSWORD
RCLONE_CONFIG_OFFSITECRYPT_PASSWORD2="$(rclone obscure salt2)"; export RCLONE_CONFIG_OFFSITECRYPT_PASSWORD2
export RCLONE_REMOTE=offsitecrypt: DATA_SRC="$T/data" STAGING_DIR="$T/staging" BACKUP_COMPRESS=gzip
pass=0; fail=0
check() { if "$@"; then echo "PASS $*"; pass=$((pass+1)); else echo "FAIL $*"; fail=$((fail+1)); fi; }

# Old backups for the retention test: 9 older than 30 days + 1 recent.
for d in 20260101T023000Z 20260102T023000Z 20260103T023000Z 20260104T023000Z 20260105T023000Z \
         20260106T023000Z 20260107T023000Z 20260108T023000Z 20260109T023000Z "$(date -u -d '-1 day' +%Y%m%dT023000Z)"; do
  echo x | rclone rcat "offsitecrypt:companion/$d/x" 2>/dev/null
done

check "$SCRIPTS/backup.sh" 2>"$T/b1.log"
check grep -q "stop -t 30 abc123" "$T/docker.log"
check grep -q "start abc123" "$T/docker.log"
check test "$(rclone lsf --dirs-only offsitecrypt:companion 2>/dev/null | wc -l)" -eq 7   # KEEP_MIN
check test "$(cat "$T/state")" = true

# A failing compressor must still restart the app and leave no half backup.
mkdir -p "$T/badbin"; printf '#!/bin/sh\nexit 1\n' > "$T/badbin/gzip"; chmod +x "$T/badbin/gzip"
check bash -c "! PATH='$T/badbin:$PATH' '$SCRIPTS/backup.sh' 2>/dev/null"
check test "$(cat "$T/state")" = true
check test -z "$(find "$T/staging" -mindepth 1 -maxdepth 1 -type d -name 'failed-*')"

export SCRATCH_ROOT="$T/scratch" DRILL_LOG="$T/drills.log" DRILL_PORT="${DRILL_PORT:-3998}"
check "$SCRIPTS/restore.sh" fetch latest "$T/fetched" 2>/dev/null
check cmp "$T/data/db/wal/00001.wal" "$T/fetched/data/db/wal/00001.wal"
check env DRILL_EMAIL=drill@example.org DRILL_PASSWORD=drill-password-123 "$SCRIPTS/restore.sh" drill latest 2>"$T/d1.log"
check grep -q PASSED "$T/drills.log"
check bash -c "! DRILL_EMAIL=drill@example.org DRILL_PASSWORD=wrong '$SCRIPTS/restore.sh' drill latest 2>/dev/null"
check test -z "$(ls "$T/scratch")"

# --- The same backup and restore over SFTP (Hetzner Storage Box layout) ---------
# rclone serves $T/sftproot over SFTP with a password; "offsite" becomes an sftp
# remote with the settings of deploy/backup.env.example block A, crypt on top.
SFTP_PORT="${SFTP_PORT:-3997}"
rclone serve sftp "$T/sftproot" --addr "127.0.0.1:$SFTP_PORT" --user backup --pass sftp-secret \
  >"$T/sftp.log" 2>&1 & echo $! > "$T/sftp.pid"
for _ in $(seq 50); do (exec 3<>"/dev/tcp/127.0.0.1/$SFTP_PORT") 2>/dev/null && break; sleep 0.2; done
sftp_env() {
  env RCLONE_CONFIG_OFFSITE_TYPE=sftp RCLONE_CONFIG_OFFSITE_HOST=127.0.0.1 \
    RCLONE_CONFIG_OFFSITE_PORT="$SFTP_PORT" RCLONE_CONFIG_OFFSITE_USER=backup \
    RCLONE_CONFIG_OFFSITE_PASS="$(rclone obscure sftp-secret)" \
    RCLONE_CONFIG_OFFSITE_DISABLE_HASHCHECK=true RCLONE_TRANSFERS=2 RCLONE_CHECKERS=4 \
    RCLONE_CONFIG_OFFSITECRYPT_REMOTE=offsite:companion-backups "$@"
}
# Old backups for retention over SFTP: 8 older than 30 days.
for d in 20260201T023000Z 20260202T023000Z 20260203T023000Z 20260204T023000Z \
         20260205T023000Z 20260206T023000Z 20260207T023000Z 20260208T023000Z; do
  echo x | sftp_env rclone rcat "offsitecrypt:companion/$d/x" 2>/dev/null
done
check sftp_env "$SCRIPTS/backup.sh" 2>"$T/b-sftp.log"
check grep -q "remote copy verified" "$T/b-sftp.log"
check test "$(sftp_env rclone lsf --dirs-only offsitecrypt:companion 2>/dev/null | wc -l)" -eq 7   # KEEP_MIN over SFTP
# On the box only encrypted names: no "companion" folder, no archive name in clear.
check test -d "$T/sftproot/companion-backups"
check bash -c "! find '$T/sftproot' | grep -Eq 'companion/|manifest|\.tar'"
check sftp_env "$SCRIPTS/restore.sh" fetch latest "$T/fetched-sftp" 2>/dev/null
check cmp "$T/data/db/wal/00001.wal" "$T/fetched-sftp/data/db/wal/00001.wal"

# --- healthcheck-remote.sh ------------------------------------------------------------
HP="${HEALTH_PORT:-3996}"
nohup python3 -I "$HERE/mock_app.py" "$HP" >/dev/null 2>&1 & echo $! > "$T/health.pid"; sleep 0.7
export STATE_FILE="$T/hstate/health" ALERT_EMAIL=owner@example.org ALERT_MAILER=sendmail FAILS_BEFORE_ALERT=2
check "$SCRIPTS/healthcheck-remote.sh" "http://127.0.0.1:$HP/healthz" 2>/dev/null
check bash -c "! '$SCRIPTS/healthcheck-remote.sh' 'http://127.0.0.1:$HP/auth/login' 2>/dev/null"   # 200 without status ok
check test ! -e "$T/mail.log"                                                                      # one failure: no mail yet
check bash -c "! HEALTH_TIMEOUT=2 '$SCRIPTS/healthcheck-remote.sh' 'http://127.0.0.1:1/healthz' 2>/dev/null"  # nothing listening
check grep -q "Subject: Team Companion is DOWN" "$T/mail.log"
check test "$(grep -c '^Subject:' "$T/mail.log")" -eq 1
check bash -c "! '$SCRIPTS/healthcheck-remote.sh' 'http://127.0.0.1:$HP/nothing' 2>/dev/null"      # 404, still down: no new mail
check test "$(grep -c '^Subject:' "$T/mail.log")" -eq 1
check "$SCRIPTS/healthcheck-remote.sh" "http://127.0.0.1:$HP/healthz" 2>/dev/null
check grep -q "Subject: Team Companion is back" "$T/mail.log"
check test "$(cat "$T/hstate/health")" = 0
rm -f "$T/mail.log"

# --- backup-watch.sh --------------------------------------------------------------------
# remote: the raw storage (no crypt passphrase), newest modification time.
check env WATCH_REMOTE="offsite:$T/crypt" "$SCRIPTS/backup-watch.sh" remote 2>/dev/null
check sftp_env env WATCH_REMOTE=offsite:companion-backups "$SCRIPTS/backup-watch.sh" remote 2>/dev/null
cp -r "$T/crypt" "$T/stale"; find "$T/stale" -type f -exec touch -d '-3 days' {} +
check bash -c "! WATCH_REMOTE='offsite:$T/stale' '$SCRIPTS/backup-watch.sh' remote 2>/dev/null"
check grep -q "Subject: Team Companion backups: the newest backup is 7[0-9] h old" "$T/mail.log"
mkdir -p "$T/empty"
check bash -c "! WATCH_REMOTE='offsite:$T/empty' '$SCRIPTS/backup-watch.sh' remote 2>/dev/null"
# log: the "done" line backup.sh wrote (here from the first run's log).
check env WATCH_LOG_SOURCE="cat '$T/b1.log'" "$SCRIPTS/backup-watch.sh" log 2>/dev/null
echo "2026-01-01T02:30:41Z backup: backup 20260101T023000Z done: 1 bytes" > "$T/old.log"
check bash -c "! WATCH_LOG_SOURCE=\"cat '$T/old.log'\" '$SCRIPTS/backup-watch.sh' log 2>/dev/null"
check bash -c "! WATCH_LOG_SOURCE=\"cat '$T/b-failed.log'\" '$SCRIPTS/backup-watch.sh' log 2>/dev/null"  # unreadable log alerts
echo "passed $pass, failed $fail"; [[ $fail -eq 0 ]]
