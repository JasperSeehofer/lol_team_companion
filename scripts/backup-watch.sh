#!/usr/bin/env bash
# Dead-man's switch for the nightly backups, without an external service: run
# it from ANOTHER machine (the owner's second server) once or twice a day by
# cron or a systemd timer (docs/RUNBOOK.md section 2.7). It finds the time of
# the newest backup and alerts when that is older than MAX_AGE_HOURS (36: one
# missed night plus slack).
#
#   backup-watch.sh [remote]   newest file on the backup storage (default)
#   backup-watch.sh log        last "backup <id> done" line of the backup log
#
# remote: WATCH_REMOTE is the rclone remote and folder of the RAW storage,
#   for example "watch:companion-backups" with RCLONE_CONFIG_WATCH_* for a
#   read-only Storage Box sub-account on the same directory. Not the crypt
#   remote: file names are encrypted on the storage, but modification times
#   are not, so the watching machine needs no crypt passphrase. The newest
#   modification time of any file below it is the last upload (backup.sh
#   uploads with the archive's own time, written minutes before).
# log: reads the backup sidecar's log on the VM over SSH and takes the last
#   "<time> backup: backup <id> done:" line, which backup.sh writes only after
#   the upload was downloaded again and verified.
#     WATCH_SSH        user@host of the VM (required for log)
#     WATCH_SSH_PORT   SSH port (default: 22)
#     WATCH_LOG_CMD    command run there (default: cd /opt/companion &&
#                      docker compose logs --no-log-prefix --since 72h backup)
#     WATCH_LOG_SOURCE replaces the whole SSH command, e.g. "cat backup.log"
#
# Exit 0 when the newest backup is young enough, 1 when it is too old or
# cannot be found (both alert), 2 for a bad setting.
#
# Configuration (environment, both modes):
#   MAX_AGE_HOURS   alert when the newest backup is older (default: 36)
#   ALERT_EMAIL     where alerts go (unset: no mail, only the exit code)
#   ALERT_MAILER    auto (default), mail or sendmail

set -Eeuo pipefail

usage() { sed -n '2,33p' "$0" | sed 's/^# \{0,1\}//'; }
case "${1:-}" in -h|--help) usage; exit 0 ;; esac

MODE="${1:-remote}"
MAX_AGE_HOURS="${MAX_AGE_HOURS:-36}"

log() { printf '%s backup-watch: %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" >&2; }
[[ "$MAX_AGE_HOURS" =~ ^[1-9][0-9]*$ ]] || { log "MAX_AGE_HOURS must be a whole number of hours"; exit 2; }

# send_mail <subject> <body>: through mail or sendmail; a missing mailer is
# reported, never fatal.
send_mail() {
  [[ -n "${ALERT_EMAIL:-}" ]] || return 0
  local mailer="${ALERT_MAILER:-auto}" sendmail=""
  sendmail="$(command -v sendmail 2>/dev/null || true)"
  [[ -z "$sendmail" && -x /usr/sbin/sendmail ]] && sendmail=/usr/sbin/sendmail
  if [[ "$mailer" == auto ]]; then
    if command -v mail >/dev/null 2>&1; then mailer=mail
    elif [[ -n "$sendmail" ]]; then mailer=sendmail
    else mailer=none
    fi
  fi
  case "$mailer" in
    mail) printf '%s\n' "$2" | mail -s "$1" "$ALERT_EMAIL" || log "warning: mail failed" ;;
    sendmail)
      [[ -n "$sendmail" ]] || { log "warning: sendmail not found; alert not sent"; return 0; }
      printf 'To: %s\nSubject: %s\n\n%s\n' "$ALERT_EMAIL" "$1" "$2" | "$sendmail" -t \
        || log "warning: sendmail failed"
      ;;
    *) log "warning: neither mail nor sendmail is installed; alert not sent" ;;
  esac
}

alert() { # alert <reason>
  log "ALERT: $1"
  send_mail "Team Companion backups: $1" \
    "$1

Checked from $(hostname 2>/dev/null || echo unknown) at $(date -u +%Y-%m-%dT%H:%M:%SZ) (mode: $MODE, limit: ${MAX_AGE_HOURS} h).
What to do: docs/RUNBOOK.md section 5 (Check) and section 7 (docker compose logs --since 48h backup)."
  exit 1
}

# newest_time prints the newest backup time (anything GNU date -d reads), or
# nothing when there is none.
newest_time() {
  case "$MODE" in
    remote)
      : "${WATCH_REMOTE:?WATCH_REMOTE is required for remote mode (e.g. watch:companion-backups)}"
      command -v rclone >/dev/null 2>&1 || { log "missing tool: rclone"; exit 2; }
      local listing
      # TZ=UTC: rclone prints times in the process's zone.
      listing="$(TZ=UTC rclone lsf -R --files-only --format t "$WATCH_REMOTE")" \
        || alert "cannot list $WATCH_REMOTE (storage unreachable or credentials wrong)"
      [[ -n "$listing" ]] || return 0
      printf '%s UTC\n' "$(sort <<<"$listing" | tail -n 1)"
      ;;
    log)
      local source lines
      if [[ -n "${WATCH_LOG_SOURCE:-}" ]]; then
        source="$WATCH_LOG_SOURCE"
      else
        : "${WATCH_SSH:?WATCH_SSH (user@host) is required for log mode}"
        local cmd="${WATCH_LOG_CMD:-cd /opt/companion && docker compose logs --no-log-prefix --since 72h backup}"
        source="ssh -o BatchMode=yes -o ConnectTimeout=20 -p ${WATCH_SSH_PORT:-22} $WATCH_SSH $(printf '%q' "$cmd")"
      fi
      lines="$(bash -c "$source" 2>&1)" || alert "cannot read the backup log ($source): ${lines:0:200}"
      grep -Eo '^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z backup: backup [^ ]+ done:' <<<"$lines" \
        | tail -n 1 | cut -d' ' -f1 || true
      ;;
    *) log "unknown mode $MODE (remote or log)"; exit 2 ;;
  esac
}

# An alert inside newest_time (a subshell) has already logged and mailed;
# its exit status ends this script through errexit.
NEWEST="$(newest_time)"
[[ -n "$NEWEST" ]] || alert "no backup found (mode $MODE)"
THEN="$(date -u -d "$NEWEST" +%s 2>/dev/null)" || alert "cannot read the backup time \"$NEWEST\""
AGE_H=$(( ($(date -u +%s) - THEN) / 3600 ))
NEWEST_UTC="$(date -u -d "@$THEN" +%Y-%m-%dT%H:%M:%SZ)"
if (( AGE_H >= MAX_AGE_HOURS )); then
  alert "the newest backup is ${AGE_H} h old ($NEWEST_UTC), more than ${MAX_AGE_HOURS} h"
fi
log "ok: newest backup $NEWEST_UTC (${AGE_H} h old)"
