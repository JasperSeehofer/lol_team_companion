#!/usr/bin/env bash
# Backup sidecar entrypoint.
#   schedule  (default) run backup.sh every day at BACKUP_AT (HH:MM, UTC)
#   now       run one backup and exit (docker compose run --rm backup now)
#   <cmd>     anything else is executed as is (e.g. rclone lsf "$RCLONE_REMOTE")
#
# A sleep loop instead of crond: cron jobs do not inherit the container
# environment (RCLONE_CONFIG_* credentials), a plain loop does.
set -Eeuo pipefail

BACKUP_AT="${BACKUP_AT:-02:30}"
[[ "$BACKUP_AT" =~ ^([01][0-9]|2[0-3]):[0-5][0-9]$ ]] || { echo "BACKUP_AT must be HH:MM (UTC)" >&2; exit 2; }

case "${1:-schedule}" in
  now)
    exec /usr/local/bin/backup.sh
    ;;
  schedule)
    echo "backup sidecar: daily at $BACKUP_AT UTC"
    while true; do
      now="$(date -u +%s)"
      next="$(date -u -d "today $BACKUP_AT" +%s)"
      (( next > now )) || next="$(date -u -d "tomorrow $BACKUP_AT" +%s)"
      echo "backup sidecar: next run $(date -u -d "@$next" +%Y-%m-%dT%H:%MZ)"
      sleep $(( next - now ))
      /usr/local/bin/backup.sh || echo "backup sidecar: backup failed (see above); next attempt tomorrow"
    done
    ;;
  *)
    exec "$@"
    ;;
esac
