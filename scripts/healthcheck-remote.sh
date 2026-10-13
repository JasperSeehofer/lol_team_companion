#!/usr/bin/env bash
# Team Companion uptime check, run from ANOTHER machine (for example the
# owner's second Hetzner server) every few minutes by cron or a systemd timer
# (docs/RUNBOOK.md section 2.7). No external service involved.
#
#   healthcheck-remote.sh [url]    default: https://app.team-companion.eu/healthz
#
# Exit 0 when the URL answers HTTP 200 with "status":"ok" within the timeout,
# exit 1 otherwise (the reason goes to stderr), exit 2 for a bad setting.
# /healthz answers 503 when the database query fails, and nothing at all when
# the VM, Caddy or the app is down.
#
# Alerts (optional): with ALERT_EMAIL set, one mail goes out when the site
# has failed FAILS_BEFORE_ALERT checks in a row, and one when it answers
# again; nothing in between, so a check every few minutes does not flood the
# inbox. The mail is sent with `mail` (mailutils, bsd-mailx) or `sendmail`
# (any MTA), whichever is installed. Without either, the script only exits
# non-zero (cron mails the output itself when MAILTO is set).
#
# Configuration (environment):
#   HEALTH_URL          the URL to check (an argument wins)
#   HEALTH_TIMEOUT      seconds for one request, connect included (default: 10)
#   ALERT_EMAIL         where alerts go (unset: no mail)
#   ALERT_MAILER        auto (default), mail or sendmail
#   FAILS_BEFORE_ALERT  failed checks in a row before the alert (default: 2)
#   STATE_FILE          remembers the failures in a row
#                       (default: ${XDG_STATE_HOME:-~/.local/state}/companion-healthcheck.state);
#                       if it cannot be written, every failed check mails (warned)

set -Eeuo pipefail

usage() { sed -n '2,28p' "$0" | sed 's/^# \{0,1\}//'; }
if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then usage; exit 0; fi

URL="${1:-${HEALTH_URL:-https://app.team-companion.eu/healthz}}"
TIMEOUT="${HEALTH_TIMEOUT:-10}"
FAILS_BEFORE_ALERT="${FAILS_BEFORE_ALERT:-2}"
STATE_FILE="${STATE_FILE:-${XDG_STATE_HOME:-$HOME/.local/state}/companion-healthcheck.state}"

log() { printf '%s healthcheck: %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" >&2; }
[[ "$TIMEOUT" =~ ^[1-9][0-9]*$ ]] || { log "HEALTH_TIMEOUT must be a whole number of seconds"; exit 2; }
[[ "$FAILS_BEFORE_ALERT" =~ ^[1-9][0-9]*$ ]] || { log "FAILS_BEFORE_ALERT must be 1 or more"; exit 2; }
command -v curl >/dev/null 2>&1 || { log "missing tool: curl"; exit 2; }

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

# --- the check ------------------------------------------------------------------
REASON=""
if OUT="$(curl -sS -m "$TIMEOUT" -H 'Accept: application/json' -w '\n%{http_code}' "$URL" 2>&1)"; then
  CODE="${OUT##*$'\n'}"
  BODY="${OUT%$'\n'*}"
  if [[ "$CODE" != 200 ]]; then
    REASON="HTTP $CODE: ${BODY:0:200}"
  elif ! grep -Eq '"status"[[:space:]]*:[[:space:]]*"ok"' <<<"$BODY"; then
    REASON="HTTP 200 without \"status\":\"ok\": ${BODY:0:200}"
  fi
else
  # curl's error message, without the "000" status line -w appended.
  REASON="no answer: ${OUT%$'\n'000}"
  REASON="${REASON:0:200}"
fi
# One line for the log and the mail subject's companion text.
REASON="${REASON//$'\n'/ }"
REASON="${REASON%"${REASON##*[![:space:]]}"}"

# --- state and alerts -------------------------------------------------------------
# A state file that cannot be read or written never turns a healthy site into
# a failed check, and never swallows an alert: without it every failure mails.
STATE_OK=1
mkdir -p "$(dirname "$STATE_FILE")" 2>/dev/null || true
FAILS=0
if [[ -f "$STATE_FILE" && -r "$STATE_FILE" ]]; then
  read -r FAILS < "$STATE_FILE" 2>/dev/null || true
  [[ "$FAILS" =~ ^[0-9]+$ ]] || FAILS=0
fi
save_state() { # save_state <count>
  if ! { printf '%s\n' "$1" > "$STATE_FILE"; } 2>/dev/null; then
    STATE_OK=0
    log "warning: cannot write the state file $STATE_FILE; alerts are not de-duplicated"
  fi
}
HOST="$(hostname 2>/dev/null || echo unknown)"

if [[ -z "$REASON" ]]; then
  if (( FAILS >= FAILS_BEFORE_ALERT )); then
    log "ok again after $FAILS failed checks"
    send_mail "Team Companion is back: $URL" \
      "$URL answers \"status\":\"ok\" again after $FAILS failed checks in a row (checked from $HOST at $(date -u +%Y-%m-%dT%H:%M:%SZ))."
  fi
  save_state 0
  log "ok: $URL"
  exit 0
fi

FAILS=$((FAILS + 1))
save_state "$FAILS"
log "FAILED ($FAILS in a row): $URL: $REASON"
if (( FAILS == FAILS_BEFORE_ALERT || STATE_OK == 0 )); then
  send_mail "Team Companion is DOWN: $URL" \
    "$URL failed $FAILS checks in a row (from $HOST, latest at $(date -u +%Y-%m-%dT%H:%M:%SZ)).

Latest reason: $REASON

What to do: docs/RUNBOOK.md section 11 (incident checklist). Another mail follows when it answers again."
fi
exit 1
