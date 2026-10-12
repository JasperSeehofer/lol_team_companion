#!/usr/bin/env bash
# Offline tests for scripts/backup.sh and scripts/restore.sh: a fake docker CLI,
# a local rclone crypt remote and a mock HTTP app (scripts/tests/mock_app.py, which
# imitates the endpoints the drill touches; it is NOT the real app).
#   scripts/tests/run-script-tests.sh scripts
# Needs: bash, rclone, python3, curl, tar, gzip, sha256sum.
set -Eeuo pipefail
SCRIPTS="$(cd "${1:?scripts dir}" && pwd)"
HERE="$(cd "$(dirname "$0")" && pwd)"
T="$(mktemp -d)"; trap 'kill "$(cat "$T/mock.pid" 2>/dev/null)" 2>/dev/null || true; rm -rf "$T"' EXIT
mkdir -p "$T/bin" "$T/data/db/wal" "$T/data/inbox" "$T/staging" "$T/crypt" "$T/scratch"
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

export PATH="$T/bin:$PATH" RCLONE_CONFIG=/dev/null
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
echo "passed $pass, failed $fail"; [[ $fail -eq 0 ]]
