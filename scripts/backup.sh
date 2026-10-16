#!/usr/bin/env bash
# Team Companion backup: consistent cold copy of the SurrealKV data directory,
# uploaded with rclone through a crypt remote (Hetzner Storage Box over SFTP,
# or S3-compatible object storage).
#
#   backup.sh            one backup now (the backup container runs this nightly)
#   backup.sh --help
#
# Why a cold copy: the app is the only process that may open the database
# (SurrealKV holds an exclusive lock), and copying the files of a live
# SurrealKV directory is not safe (compaction rewrites and deletes SSTables
# mid-copy, so the copy can mix generations). So the app container is stopped
# for the copy (seconds for a small database) and started again at once.
# The trap restarts it on any failure. See docs/RUNBOOK.md "Backups".
#
# Steps: preflight -> stop app -> tar + compress db/, inbox/ and attachments/
# (feedback screenshots, M141) -> start app ->
# test the archive -> sha256 -> manifest -> upload -> download again and compare
# the sha256 -> prune remote backups older than RETENTION_DAYS (always keeping
# the newest KEEP_MIN) -> prune local staging -> optional ping.
#
# Configuration (environment):
#   RCLONE_REMOTE        required. rclone remote and path, e.g. "offsitecrypt:" or
#                        "offsite:companion-backups". Credentials come from
#                        RCLONE_CONFIG_* variables or an rclone.conf.
#   BACKUP_PREFIX        folder under the remote (default: companion)
#   DATA_SRC             directory that holds db/, inbox/ and attachments/ (default: /data)
#   STAGING_DIR          local working directory (default: /staging)
#   COMPOSE_PROJECT      compose project of the app (default: companion)
#   APP_SERVICE          compose service name of the app (default: app)
#   APP_CONTAINER        container name or id; overrides the two above
#   BACKUP_REASON        optional tag appended to the backup id (e.g. pre-update)
#   BACKUP_COMPRESS      zstd (default) or gzip
#   RETENTION_DAYS       delete remote backups older than this (default: 30)
#   KEEP_MIN             never delete the newest N remote backups (default: 7)
#   KEEP_LOCAL           newest N backups kept in STAGING_DIR (default: 1)
#   STOP_TIMEOUT         seconds docker waits for the app to stop (default: 30)
#   HEALTH_TIMEOUT       seconds to wait for the app to be healthy again (default: 180)
#   VERIFY_DOWNLOAD      1 (default) downloads the archive again and checks sha256
#   HEALTHCHECK_PING_URL optional; GET on success, GET <url>/fail on failure
#                        (healthchecks.io style). External service: owner decides.
#   DOCKER               docker binary (default: docker)

set -Eeuo pipefail
umask 077

usage() { sed -n '2,42p' "$0" | sed 's/^# \{0,1\}//'; }
if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then usage; exit 0; fi

: "${RCLONE_REMOTE:?RCLONE_REMOTE is required (e.g. offsitecrypt:)}"
BACKUP_PREFIX="${BACKUP_PREFIX:-companion}"
DATA_SRC="${DATA_SRC:-/data}"
STAGING_DIR="${STAGING_DIR:-/staging}"
COMPOSE_PROJECT="${COMPOSE_PROJECT:-companion}"
APP_SERVICE="${APP_SERVICE:-app}"
BACKUP_COMPRESS="${BACKUP_COMPRESS:-zstd}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
KEEP_MIN="${KEEP_MIN:-7}"
KEEP_LOCAL="${KEEP_LOCAL:-1}"
STOP_TIMEOUT="${STOP_TIMEOUT:-30}"
HEALTH_TIMEOUT="${HEALTH_TIMEOUT:-180}"
VERIFY_DOWNLOAD="${VERIFY_DOWNLOAD:-1}"
DOCKER="${DOCKER:-docker}"

REMOTE_BASE="${RCLONE_REMOTE%/}"
if [[ "$REMOTE_BASE" == *: ]]; then
  REMOTE_BASE="${REMOTE_BASE}${BACKUP_PREFIX}"
else
  REMOTE_BASE="${REMOTE_BASE}/${BACKUP_PREFIX}"
fi

log() { printf '%s backup: %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" >&2; }
die() { log "ERROR: $*"; exit 1; }

ping_url() {
  [[ -n "${HEALTHCHECK_PING_URL:-}" ]] || return 0
  curl -fsS -m 10 --retry 3 -o /dev/null "$1" || log "warning: ping to monitor failed"
}

case "$BACKUP_COMPRESS" in
  zstd) COMPRESS_CMD=(zstd -q -T0 -10); DECOMPRESS_CMD=(zstd -q -d -c); EXT="tar.zst" ;;
  gzip) COMPRESS_CMD=(gzip -6); DECOMPRESS_CMD=(gzip -d -c); EXT="tar.gz" ;;
  *) die "BACKUP_COMPRESS must be zstd or gzip" ;;
esac

for tool in "$DOCKER" rclone tar sha256sum "${COMPRESS_CMD[0]}" du df date; do
  command -v "$tool" >/dev/null 2>&1 || die "missing tool: $tool"
done
[[ -d "$DATA_SRC/db" ]] || die "no database directory at $DATA_SRC/db"

mkdir -p "$STAGING_DIR"
# One backup at a time.
exec 9>"$STAGING_DIR/.backup.lock"
if command -v flock >/dev/null 2>&1; then
  flock -n 9 || die "another backup is running"
fi

TS="$(date -u +%Y%m%dT%H%M%SZ)"
ID="$TS${BACKUP_REASON:+-$BACKUP_REASON}"
# Two runs in the same second (a manual run right after a failed one) must
# not share a work directory: the second would treat the first's finished
# archive as its own (CI caught it).
n=1
while [[ -e "$STAGING_DIR/$ID" || -e "$STAGING_DIR/failed-$ID" ]]; do
  n=$((n + 1))
  ID="$TS${BACKUP_REASON:+-$BACKUP_REASON}-$n"
done
WORK="$STAGING_DIR/$ID"
ARCHIVE="companion-$ID.$EXT"
mkdir "$WORK"

CONTAINER=""
STOPPED=0
FAILED=1

restart_app() {
  if [[ "$STOPPED" == 1 && -n "$CONTAINER" ]]; then
    log "starting the app container again"
    if "$DOCKER" start "$CONTAINER" >/dev/null; then
      STOPPED=0
    else
      log "ERROR: could not start $CONTAINER: start it by hand (docker compose up -d app)"
    fi
  fi
}

on_exit() {
  local code=$?
  restart_app
  if [[ "$FAILED" == 1 ]]; then
    log "backup $ID FAILED (exit $code)"
    # Keep a finished archive for inspection (failed-<id>, removed after 7
    # days); drop a half-written one so it never counts as a backup.
    if [[ -s "$WORK/$ARCHIVE.sha256" ]]; then
      mv "$WORK" "$STAGING_DIR/failed-$ID" 2>/dev/null || true
      log "archive kept at $STAGING_DIR/failed-$ID"
    else
      rm -rf "$WORK"
    fi
    [[ -n "${HEALTHCHECK_PING_URL:-}" ]] && ping_url "${HEALTHCHECK_PING_URL%/}/fail"
  fi
}
trap on_exit EXIT
trap 'exit 130' INT TERM

resolve_container() {
  if [[ -n "${APP_CONTAINER:-}" ]]; then
    printf '%s\n' "$APP_CONTAINER"
    return
  fi
  local ids
  ids="$("$DOCKER" ps -a -q \
    --filter "label=com.docker.compose.project=$COMPOSE_PROJECT" \
    --filter "label=com.docker.compose.service=$APP_SERVICE")"
  [[ -n "$ids" ]] || die "no container for compose service $COMPOSE_PROJECT/$APP_SERVICE"
  [[ "$(wc -l <<<"$ids")" -eq 1 ]] || die "more than one container for $APP_SERVICE: set APP_CONTAINER"
  printf '%s\n' "$ids"
}

health_status() {
  "$DOCKER" inspect -f '{{if .State.Health}}{{.State.Health.Status}}{{else}}nohc-{{.State.Status}}{{end}}' "$CONTAINER"
}

wait_healthy() {
  local waited=0 status
  while (( waited < HEALTH_TIMEOUT )); do
    status="$(health_status 2>/dev/null || echo unknown)"
    if [[ "$status" == "healthy" || "$status" == "nohc-running" ]]; then
      return 0
    fi
    sleep 2
    waited=$((waited + 2))
  done
  return 1
}

# --- preflight ----------------------------------------------------------------
log "backup $ID -> $REMOTE_BASE/$ID"
rclone mkdir "$REMOTE_BASE" || die "cannot reach $REMOTE_BASE (check RCLONE_* settings)"
rclone lsf --max-depth 1 "$REMOTE_BASE" >/dev/null || die "cannot list $REMOTE_BASE"

DATA_BYTES="$(du -sb "$DATA_SRC/db" | cut -f1)"
ATTACH_BYTES=0
if [[ -d "$DATA_SRC/attachments" ]]; then ATTACH_BYTES="$(du -sb "$DATA_SRC/attachments" | cut -f1)"; fi
FREE_BYTES="$(( $(df -Pk "$STAGING_DIR" | awk 'NR==2 {print $4}') * 1024 ))"
NEED_BYTES="$(( DATA_BYTES + DATA_BYTES / 5 + ATTACH_BYTES + 50 * 1024 * 1024 ))"
(( FREE_BYTES > NEED_BYTES )) || die "not enough space in $STAGING_DIR: need $NEED_BYTES bytes, have $FREE_BYTES"

CONTAINER="$(resolve_container)"
IMAGE_ID="$("$DOCKER" inspect -f '{{.Image}}' "$CONTAINER")"
IMAGE_REF="$("$DOCKER" inspect -f '{{.Config.Image}}' "$CONTAINER")"
IMAGE_REV="$("$DOCKER" inspect -f '{{index .Config.Labels "org.opencontainers.image.revision"}}' "$CONTAINER" 2>/dev/null || true)"
WAS_RUNNING="$("$DOCKER" inspect -f '{{.State.Running}}' "$CONTAINER")"

# --- stop, copy, start --------------------------------------------------------
T0="$(date +%s)"
if [[ "$WAS_RUNNING" == "true" ]]; then
  log "stopping $CONTAINER (timeout ${STOP_TIMEOUT}s)"
  STOPPED=1
  "$DOCKER" stop -t "$STOP_TIMEOUT" "$CONTAINER" >/dev/null
  [[ "$("$DOCKER" inspect -f '{{.State.Running}}' "$CONTAINER")" == "false" ]] || die "container still running"
else
  log "app container is not running; copying without a stop"
fi

PARTS=(db)
[[ -d "$DATA_SRC/inbox" ]] && PARTS+=(inbox)
[[ -d "$DATA_SRC/attachments" ]] && PARTS+=(attachments)
log "archiving ${PARTS[*]} ($DATA_BYTES bytes in db/)"
tar -C "$DATA_SRC" --numeric-owner -cf - "${PARTS[@]}" | "${COMPRESS_CMD[@]}" > "$WORK/$ARCHIVE"

restart_app
T1="$(date +%s)"
DOWNTIME=$((T1 - T0))
log "app start requested after ${DOWNTIME}s of downtime"

# --- verify the archive locally -----------------------------------------------
"${DECOMPRESS_CMD[@]}" "$WORK/$ARCHIVE" | tar -tf - > "$WORK/files.txt"
FILE_COUNT="$(wc -l < "$WORK/files.txt" | tr -d ' ')"
grep -q '^db/' "$WORK/files.txt" || die "archive has no db/ entries"
(cd "$WORK" && sha256sum "$ARCHIVE" > "$ARCHIVE.sha256")
SHA256="$(cut -d' ' -f1 < "$WORK/$ARCHIVE.sha256")"
ARCHIVE_BYTES="$(wc -c < "$WORK/$ARCHIVE" | tr -d ' ')"

cat > "$WORK/manifest.json" <<JSON
{
  "id": "$ID",
  "created_utc": "$TS",
  "reason": "${BACKUP_REASON:-scheduled}",
  "archive": "$ARCHIVE",
  "archive_bytes": $ARCHIVE_BYTES,
  "sha256": "$SHA256",
  "db_bytes": $DATA_BYTES,
  "file_count": $FILE_COUNT,
  "compression": "$BACKUP_COMPRESS",
  "downtime_seconds": $DOWNTIME,
  "image_ref": "$IMAGE_REF",
  "image_id": "$IMAGE_ID",
  "image_revision": "${IMAGE_REV:-unknown}",
  "format": "cold copy of SURREAL_DATA_DIR (SurrealKV); restore with the same or a newer image"
}
JSON
rm -f "$WORK/files.txt"

# --- wait for the app before the slow part ------------------------------------
HEALTH_OK=1
if [[ "$WAS_RUNNING" == "true" ]]; then
  if wait_healthy; then
    log "app healthy again after $(( $(date +%s) - T0 ))s in total"
  else
    HEALTH_OK=0
    log "ERROR: app not healthy ${HEALTH_TIMEOUT}s after the restart; continuing with the upload"
  fi
fi

# --- upload and verify ----------------------------------------------------------
log "uploading $ARCHIVE ($ARCHIVE_BYTES bytes)"
rclone copy --no-traverse "$WORK" "$REMOTE_BASE/$ID"

if [[ "$VERIFY_DOWNLOAD" == 1 ]]; then
  REMOTE_SHA="$(rclone cat "$REMOTE_BASE/$ID/$ARCHIVE" | sha256sum | cut -d' ' -f1)"
  [[ "$REMOTE_SHA" == "$SHA256" ]] || die "uploaded archive does not match (local $SHA256, remote $REMOTE_SHA)"
  log "remote copy verified (sha256 $SHA256)"
else
  rclone lsf "$REMOTE_BASE/$ID/$ARCHIVE" >/dev/null || die "uploaded archive not found"
fi

# --- retention ------------------------------------------------------------------
# Only after today's backup is safely uploaded, and never below KEEP_MIN, so a
# month of failed backups cannot delete the last good ones.
CUTOFF="$(date -u -d "-${RETENTION_DAYS} days" +%Y%m%dT%H%M%SZ)"
mapfile -t REMOTE_IDS < <(rclone lsf --dirs-only "$REMOTE_BASE" | sed 's#/$##' \
  | grep -E '^[0-9]{8}T[0-9]{6}Z' | sort)
PRUNABLE=$(( ${#REMOTE_IDS[@]} - KEEP_MIN ))
PRUNED=0
for (( i = 0; i < PRUNABLE; i++ )); do
  old="${REMOTE_IDS[$i]}"
  if [[ "${old:0:16}" < "$CUTOFF" ]]; then
    log "pruning $old (older than $RETENTION_DAYS days)"
    rclone purge "$REMOTE_BASE/$old"
    PRUNED=$((PRUNED + 1))
  fi
done

mapfile -t LOCAL_IDS < <(find "$STAGING_DIR" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' \
  | grep -E '^[0-9]{8}T[0-9]{6}Z' | sort)
for (( i = 0; i < ${#LOCAL_IDS[@]} - KEEP_LOCAL; i++ )); do
  rm -rf "${STAGING_DIR:?}/${LOCAL_IDS[$i]}"
done
find "$STAGING_DIR" -mindepth 1 -maxdepth 1 -type d -name 'failed-*' -mtime +7 -exec rm -rf {} +

[[ "$HEALTH_OK" == 1 ]] || die "backup uploaded, but the app did not become healthy: check docker compose ps / logs"

FAILED=0
log "backup $ID done: $ARCHIVE_BYTES bytes, downtime ${DOWNTIME}s, pruned $PRUNED, $(( ${#REMOTE_IDS[@]} - PRUNED )) remote backups kept"
ping_url "${HEALTHCHECK_PING_URL:-}"
