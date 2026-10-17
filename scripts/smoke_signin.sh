#!/usr/bin/env bash
# Sign-in smoke test over plain HTTP (no browser): registers an account with
# an invite code through the real registration form, signs in with it in a
# fresh cookie jar, and reads the account back through an authenticated
# server function. CI runs it against the container image; it also works
# against `cargo leptos watch` started with the e2e environment.
#
#   scripts/smoke_signin.sh [base_url]
#
# Environment:
#   INVITE_CODE    invite code for the registration (default: E2E-INVITE-CODE)
#   SMOKE_EMAIL    account email (default: smoke-<random>@test.invalid)
#   SMOKE_PASSWORD password (default: a random 24-character string)
#   SKIP_REGISTER  1 = only sign in (the account must exist)
#
# The forms are Leptos ActionForms whose action is the server function's
# hashed URL (/api/register_action<hash>, /api/login_action<hash>); a form
# post with Accept: text/html is answered with a redirect and the session
# cookie (`id`).
set -Eeuo pipefail

BASE="${1:-http://127.0.0.1:3020}"
INVITE_CODE="${INVITE_CODE:-E2E-INVITE-CODE}"
RAND="$(head -c 6 /dev/urandom | od -An -tx1 | tr -d ' \n')"
EMAIL="${SMOKE_EMAIL:-smoke-$RAND@test.invalid}"
PASSWORD="${SMOKE_PASSWORD:-pw-$(head -c 12 /dev/urandom | od -An -tx1 | tr -d ' \n')}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

log() { printf 'smoke: %s\n' "$*" >&2; }
die() { log "FAILED: $*"; exit 1; }

form_action() { # form_action <page html> <server fn name>
  grep -oE "action=\"/api/$2[A-Za-z0-9_]*\"" <<<"$1" | head -n 1 | cut -d'"' -f2
}

# post_form <action> <jar> <data...>: prints the HTTP status; headers in $WORK/h
post_form() {
  local action="$1" jar="$2"
  shift 2
  local args=()
  for kv in "$@"; do args+=(--data-urlencode "$kv"); done
  curl -sS -m 30 -o "$WORK/body" -D "$WORK/h" -c "$jar" -b "$jar" -w '%{http_code}' \
    -X POST "$BASE$action" \
    -H 'Content-Type: application/x-www-form-urlencoded' \
    -H 'Accept: text/html' \
    -H "Referer: $BASE/" \
    "${args[@]}"
}

has_session_cookie() { grep -qiE '^set-cookie: *id=' "$WORK/h"; }
location() { tr -d '\r' < "$WORK/h" | sed -n 's/^[Ll]ocation: *//p' | head -n 1; }

curl -fsS -m 10 "$BASE/healthz" | grep -q '"db":"ok"' || die "/healthz is not ok"
log "/healthz ok"

if [[ "${SKIP_REGISTER:-0}" != 1 ]]; then
  html="$(curl -fsS -m 15 "$BASE/auth/register?invite=$INVITE_CODE")" || die "GET /auth/register"
  action="$(form_action "$html" register_action)"
  [[ -n "$action" ]] || die "registration form action not found"
  status="$(post_form "$action" "$WORK/jar-register" \
    "username=smoke-$RAND" "email=$EMAIL" "password=$PASSWORD" "invite_code=$INVITE_CODE" "accept_terms=on")"
  has_session_cookie || die "registration: no session cookie (HTTP $status, $(head -c 200 "$WORK/body"))"
  log "registered $EMAIL (HTTP $status -> $(location))"
fi

html="$(curl -fsS -m 15 "$BASE/auth/login")" || die "GET /auth/login"
action="$(form_action "$html" login_action)"
[[ -n "$action" ]] || die "sign-in form action not found"
status="$(post_form "$action" "$WORK/jar-login" "email=$EMAIL" "password=$PASSWORD")"
has_session_cookie || die "sign-in: no session cookie (HTTP $status, $(head -c 200 "$WORK/body"))"
loc="$(location)"
[[ "$loc" == "/" ]] || die "sign-in: expected a redirect to /, got '$loc' (HTTP $status)"
log "signed in (HTTP $status -> $loc)"

# The authenticated read: get_current_user's hashed URL is in the WASM.
fn="$(curl -fsS -m 30 "$BASE/pkg/lol_team_companion.wasm" | grep -aoE '/api/get_current_user[0-9]+' | head -n 1 || true)"
[[ -n "$fn" ]] || die "get_current_user URL not found in the WASM"
curl -fsS -m 15 -b "$WORK/jar-login" -X POST "$BASE$fn" -H 'Accept: application/json' \
  | grep -qi "$EMAIL" || die "$fn did not return the signed-in account"
log "authenticated read ok ($fn)"
log "PASSED"
