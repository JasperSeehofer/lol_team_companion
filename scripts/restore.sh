#!/usr/bin/env bash
# Team Companion restore: fetch a backup made by scripts/backup.sh, verify it, and
# either test it in a throwaway container (drill) or put it back into the
# production volume (apply). Runs on the VM host or on any machine with docker,
# rclone, curl and zstd (a drill on a laptop also proves the backups are usable
# away from the VM). See scripts/restore-drill.md and docs/RUNBOOK.md.
#
#   restore.sh list                     backups on the remote, oldest first
#   restore.sh fetch <id|latest> [dir]  download, check sha256, extract to dir
#                                       (default ./restore-<id>)
#   restore.sh drill <id|latest>        fetch into a scratch dir, start a
#                                       throwaway container on 127.0.0.1, check
#                                       /healthz and a real login, clean up
#   restore.sh apply <id|latest>        replace the production data (asks
#                                       first; keeps the old data in the volume)
#
# Configuration (environment; the same RCLONE_* settings as backup.sh, e.g.
# `set -a; . deploy/backup.env; set +a`):
#   RCLONE_REMOTE, BACKUP_PREFIX   where the backups are (as in backup.sh)
#   DRILL_IMAGE        image for the drill (default: the image recorded in the
#                      manifest if present locally, else companion:latest)
#   DRILL_EMAIL        account used for the login check (a dedicated, team-less
#   DRILL_PASSWORD     account that exists in production; never an admin)
#   DRILL_PORT         host port on 127.0.0.1 (default: 3999)
#   DRILL_KEEP         1 keeps the scratch dir and container for inspection
#   SCRATCH_ROOT       where scratch dirs go (default: ${TMPDIR:-/tmp})
#   COMPOSE_PROJECT    compose project (default: companion), for apply
#   APP_SERVICE        compose service (default: app), for apply
#   APP_CONTAINER      container name or id; overrides the two above
#   DATA_VOLUME        production volume (default: ${COMPOSE_PROJECT}_data)
#   DOCKER             docker binary (default: docker)

set -Eeuo pipefail
umask 077

usage() { sed -n '2,33p' "$0" | sed 's/^# \{0,1\}//'; }
case "${1:-}" in -h|--help|"") usage; exit 0 ;; esac

: "${RCLONE_REMOTE:?RCLONE_REMOTE is required (source deploy/backup.env)}"
BACKUP_PREFIX="${BACKUP_PREFIX:-companion}"
DRILL_PORT="${DRILL_PORT:-3999}"
SCRATCH_ROOT="${SCRATCH_ROOT:-${TMPDIR:-/tmp}}"
COMPOSE_PROJECT="${COMPOSE_PROJECT:-companion}"
APP_SERVICE="${APP_SERVICE:-app}"
DATA_VOLUME="${DATA_VOLUME:-${COMPOSE_PROJECT}_data}"
DOCKER="${DOCKER:-docker}"

REMOTE_BASE="${RCLONE_REMOTE%/}"
if [[ "$REMOTE_BASE" == *: ]]; then
  REMOTE_BASE="${REMOTE_BASE}${BACKUP_PREFIX}"
else
  REMOTE_BASE="${REMOTE_BASE}/${BACKUP_PREFIX}"
fi

log() { printf '%s restore: %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" >&2; }
die() { log "ERROR: $*"; exit 1; }
need() { for t in "$@"; do command -v "$t" >/dev/null 2>&1 || die "missing tool: $t"; done; }

list_ids() {
  rclone lsf --dirs-only "$REMOTE_BASE" | sed 's#/$##' | grep -E '^[0-9]{8}T[0-9]{6}Z' | sort
}

resolve_id() {
  local want="$1" id
  if [[ "$want" == "latest" ]]; then
    id="$(list_ids | tail -n 1)"
    [[ -n "$id" ]] || die "no backups under $REMOTE_BASE"
  else
    id="$want"
    rclone lsf "$REMOTE_BASE/$id/manifest.json" >/dev/null 2>&1 || die "backup $id not found"
  fi
  printf '%s\n' "$id"
}

json_field() { # json_field <file> <key>: tiny reader for the flat manifest
  sed -n "s/^ *\"$2\": *\"\{0,1\}\([^\",]*\)\"\{0,1\},\{0,1\}$/\1/p" "$1" | head -n 1
}

# fetch <id> <dir>: download into <dir>/download, verify, extract to <dir>/data
fetch() {
  local id="$1" dir="$2" archive expected actual
  need rclone tar sha256sum
  mkdir -p "$dir/download" "$dir/data"
  log "downloading $REMOTE_BASE/$id"
  rclone copy "$REMOTE_BASE/$id" "$dir/download"
  [[ -f "$dir/download/manifest.json" ]] || die "manifest.json missing in $id"
  archive="$(json_field "$dir/download/manifest.json" archive)"
  [[ -n "$archive" && -f "$dir/download/$archive" ]] || die "archive missing in $id"
  expected="$(json_field "$dir/download/manifest.json" sha256)"
  actual="$(sha256sum "$dir/download/$archive" | cut -d' ' -f1)"
  [[ "$expected" == "$actual" ]] || die "sha256 mismatch for $archive (manifest $expected, file $actual)"
  (cd "$dir/download" && sha256sum -c --quiet "$archive.sha256") || die "sha256 file check failed"
  log "sha256 ok ($actual)"
  case "$archive" in
    *.tar.zst) need zstd; zstd -q -d -c "$dir/download/$archive" | tar -C "$dir/data" -xf - ;;
    *.tar.gz) tar -C "$dir/data" -xzf "$dir/download/$archive" ;;
    *) die "unknown archive type: $archive" ;;
  esac
  [[ -d "$dir/data/db" ]] || die "archive has no db/ directory"
  mkdir -p "$dir/data/inbox"
  # A LOCK file in the copy is harmless: SurrealKV uses an OS file lock
  # (flock); the file's content (an old PID) is informational only.
  log "extracted to $dir/data ($(du -sh "$dir/data/db" | cut -f1) in db/)"
  log "made by image $(json_field "$dir/download/manifest.json" image_ref) (revision $(json_field "$dir/download/manifest.json" image_revision))"
}

# --- drill ------------------------------------------------------------------

DRILL_CONTAINER=""
DRILL_DIR=""
drill_cleanup() {
  if [[ "${DRILL_KEEP:-0}" == 1 ]]; then
    log "DRILL_KEEP=1: container ${DRILL_CONTAINER:-none} and $DRILL_DIR left in place"
    return
  fi
  if [[ -n "$DRILL_CONTAINER" ]]; then
    "$DOCKER" rm -f "$DRILL_CONTAINER" >/dev/null 2>&1 || true
  fi
  if [[ -n "$DRILL_DIR" && -d "$DRILL_DIR" ]]; then
    rm -rf "$DRILL_DIR"
  fi
}

login_check() { # login_check <base_url>: sign in with DRILL_EMAIL / DRILL_PASSWORD
  local base="$1" html action headers body jar status location
  jar="$DRILL_DIR/cookies.txt"
  html="$(curl -fsS -m 15 "$base/auth/login")" || { log "GET /auth/login failed"; return 1; }
  # The sign-in form is an ActionForm whose action is the hashed server-fn URL.
  action="$(grep -oE 'action="/api/login_action[A-Za-z0-9_]*"' <<<"$html" | head -n 1 | cut -d'"' -f2)"
  [[ -n "$action" ]] || { log "sign-in form action not found in /auth/login"; return 1; }
  headers="$DRILL_DIR/login.headers"
  body="$DRILL_DIR/login.body"
  status="$(curl -sS -m 30 -o "$body" -D "$headers" -c "$jar" -w '%{http_code}' \
    -X POST "$base$action" \
    -H 'Content-Type: application/x-www-form-urlencoded' \
    -H 'Accept: text/html' \
    -H "Referer: $base/auth/login" \
    --data-urlencode "email=$DRILL_EMAIL" \
    --data-urlencode "password=$DRILL_PASSWORD")"
  location="$(tr -d '\r' < "$headers" | sed -n 's/^[Ll]ocation: *//p' | head -n 1)"
  if ! grep -qiE '^set-cookie: *id=' "$headers"; then
    log "login failed: no session cookie (HTTP $status, location '${location}', body: $(head -c 200 "$body"))"
    return 1
  fi
  # Sign-in redirects to Home (/); a failed sign-in sets no session cookie.
  if [[ "$location" == "/" ]]; then
    log "login ok (HTTP $status, -> $location)"
  else
    log "login unclear: session cookie set but no redirect to / (HTTP $status, location '${location}')"
    return 1
  fi
  # Optional deeper check: an authenticated server function reads the user.
  local fn
  fn="$("$DOCKER" exec "$DRILL_CONTAINER" sh -c \
    "grep -aoE '/api/get_current_user[A-Za-z0-9_]*' /app/site/pkg/lol_team_companion.wasm | head -n 1" 2>/dev/null || true)"
  if [[ -n "$fn" ]]; then
    if curl -fsS -m 15 -b "$jar" -X POST "$base$fn" -H 'Accept: application/json' \
      | grep -qi "$(printf '%s' "$DRILL_EMAIL" | tr '[:upper:]' '[:lower:]')"; then
      log "authenticated read ok ($fn returns the drill account)"
    else
      log "warning: $fn did not return the drill account (check by hand)"
    fi
  else
    log "note: get_current_user URL not found in the WASM; skipped the authenticated read"
  fi
}

drill() {
  local want="$1" id image base waited=0 t0 secret run_user=()
  need "$DOCKER" rclone curl tar sha256sum
  [[ -n "${DRILL_EMAIL:-}" && -n "${DRILL_PASSWORD:-}" ]] || die "set DRILL_EMAIL and DRILL_PASSWORD"
  id="$(resolve_id "$want")"
  t0="$(date +%s)"
  DRILL_DIR="$(mktemp -d "$SCRATCH_ROOT/companion-drill-XXXXXX")"
  trap drill_cleanup EXIT
  fetch "$id" "$DRILL_DIR"

  image="${DRILL_IMAGE:-}"
  if [[ -z "$image" ]]; then
    image="$(json_field "$DRILL_DIR/download/manifest.json" image_ref)"
    if [[ -z "$image" ]] || ! "$DOCKER" image inspect "$image" >/dev/null 2>&1; then
      image="companion:latest"
    fi
  fi
  "$DOCKER" image inspect "$image" >/dev/null 2>&1 || die "image $image not found locally (set DRILL_IMAGE)"

  # Extracted files belong to whoever ran this script. As root they keep the
  # archive's owner (uid 10001, the image's user); otherwise run the container
  # as the current user so SurrealKV can write its WAL and LOCK.
  if [[ "$(id -u)" != 0 ]]; then
    run_user=(--user "$(id -u):$(id -g)")
  fi
  secret="$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')"
  DRILL_CONTAINER="companion-drill-$id"
  log "starting $DRILL_CONTAINER from $image on 127.0.0.1:$DRILL_PORT"
  "$DOCKER" run -d --name "$DRILL_CONTAINER" "${run_user[@]}" \
    --read-only --tmpfs /tmp \
    -p "127.0.0.1:$DRILL_PORT:3020" \
    -v "$DRILL_DIR/data:/data" \
    -e SESSION_SECRET="$secret" \
    -e COOKIE_SECURE=false \
    -e APP_ENV= \
    -e RIOT_API_KEY= \
    -e ADMIN_EMAILS= \
    -e BOOTSTRAP_INVITE_CODE= \
    "$image" >/dev/null

  base="http://127.0.0.1:$DRILL_PORT"
  until curl -fsS -m 3 "$base/healthz" 2>/dev/null | grep -q '"db":"ok"'; do
    sleep 2
    waited=$((waited + 2))
    if (( waited >= 180 )); then
      "$DOCKER" logs --tail 50 "$DRILL_CONTAINER" >&2 || true
      die "drill container not healthy after 180 s"
    fi
  done
  log "/healthz ok after ${waited}s"

  if ! login_check "$base"; then
    "$DOCKER" logs --tail 30 "$DRILL_CONTAINER" >&2 || true
    die "DRILL FAILED for $id: login check"
  fi
  log "DRILL PASSED for $id in $(( $(date +%s) - t0 ))s (image $image)"
  printf '%s\t%s\t%s\tPASSED\t%ss\n' "$(date -u +%Y-%m-%dT%H:%MZ)" "$id" "$image" \
    "$(( $(date +%s) - t0 ))" >> "${DRILL_LOG:-./restore-drills.log}"
}

# --- apply -------------------------------------------------------------------

resolve_app_container() {
  if [[ -n "${APP_CONTAINER:-}" ]]; then printf '%s\n' "$APP_CONTAINER"; return; fi
  local ids
  ids="$("$DOCKER" ps -a -q --filter "label=com.docker.compose.project=$COMPOSE_PROJECT" \
    --filter "label=com.docker.compose.service=$APP_SERVICE")"
  [[ -n "$ids" && "$(wc -l <<<"$ids")" -eq 1 ]] || die "cannot find exactly one $APP_SERVICE container: set APP_CONTAINER"
  printf '%s\n' "$ids"
}

apply() {
  local want="$1" id dir container image ts answer waited=0
  need "$DOCKER" rclone tar sha256sum
  id="$(resolve_id "$want")"
  container="$(resolve_app_container)"
  image="$("$DOCKER" inspect -f '{{.Config.Image}}' "$container")"
  dir="$(mktemp -d "$SCRATCH_ROOT/companion-restore-XXXXXX")"
  fetch "$id" "$dir"
  log "backup $id was made by $(json_field "$dir/download/manifest.json" image_ref); production runs $image"
  log "a backup from a NEWER build than $image will be refused at startup (unknown migration)"
  printf 'This stops %s and replaces the data in volume %s with backup %s.\nType the backup id to continue: ' \
    "$container" "$DATA_VOLUME" "$id" >&2
  read -r answer
  [[ "$answer" == "$id" ]] || die "aborted"

  ts="$(date -u +%Y%m%dT%H%M%SZ)"
  log "stopping $container"
  # From here on, any failure starts the app again (on whatever data is in
  # place: the swap below either completed or left the old data untouched).
  # shellcheck disable=SC2064 # expand $container now
  trap "log 'restore failed; starting $container again'; \"$DOCKER\" start '$container' >/dev/null 2>&1 || true" ERR INT TERM
  "$DOCKER" stop -t 30 "$container" >/dev/null
  # Swap the data inside the volume with a one-off container from the app
  # image (it has sh and cp). The backup is copied next to the live data
  # first and swapped in with two renames, so a failed copy (disk full)
  # leaves the old data in place. The old data stays in
  # /data/pre-restore-<ts>; backup.sh only archives db/ and inbox/, so it is
  # not backed up again (delete it within 30 days: privacy page retention).
  "$DOCKER" run --rm --user 0 --entrypoint sh \
    -v "$DATA_VOLUME:/data" -v "$dir/data:/restore:ro" "$image" -c "
      set -e
      rm -rf /data/db.restoring
      cp -a /restore/db /data/db.restoring
      chown -R 10001:10001 /data/db.restoring
      mkdir -p /data/pre-restore-$ts
      if [ -d /data/db ]; then mv /data/db /data/pre-restore-$ts/db; fi
      if [ -d /data/inbox ]; then mv /data/inbox /data/pre-restore-$ts/inbox; fi
      mv /data/db.restoring /data/db
      mkdir -p /data/inbox
      chown 10001:10001 /data/inbox
    "
  log "starting $container"
  "$DOCKER" start "$container" >/dev/null
  trap - ERR INT TERM
  until [[ "$("$DOCKER" inspect -f '{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "$container")" == "healthy" ]]; do
    sleep 2
    waited=$((waited + 2))
    (( waited < 240 )) || die "app not healthy after restore: docker compose logs app; old data is in /data/pre-restore-$ts"
  done
  rm -rf "$dir"
  log "restored $id; app healthy. Old data kept in volume $DATA_VOLUME at /data/pre-restore-$ts"
  log "remove it once you are sure: docker run --rm --user 0 --entrypoint rm -v $DATA_VOLUME:/data $image -rf /data/pre-restore-$ts"
}

case "${1:-}" in
  list) need rclone; list_ids ;;
  fetch)
    [[ -n "${2:-}" ]] || { usage; exit 2; }
    id="$(resolve_id "$2")"
    fetch "$id" "${3:-./restore-$id}"
    ;;
  drill)
    [[ -n "${2:-}" ]] || { usage; exit 2; }
    drill "$2"
    ;;
  apply)
    [[ -n "${2:-}" ]] || { usage; exit 2; }
    apply "$2"
    ;;
  *) usage; exit 2 ;;
esac
