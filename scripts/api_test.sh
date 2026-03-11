#!/usr/bin/env bash
# Tests key server function endpoints against the running dev server.
#
# Leptos server functions are POST to /api/{fn_name}, with:
#   Content-Type: application/x-www-form-urlencoded  (input)
#   Response: JSON
#
# Requires the server to be running: cargo leptos watch
# Usage: ./scripts/api_test.sh [base_url] [email] [password]
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:3002}"
EMAIL="${2:-test@example.com}"
PASSWORD="${3:-testpassword123}"
COOKIE_JAR="$(mktemp /tmp/lol_cookies_XXXXXX)"
trap 'rm -f "$COOKIE_JAR"' EXIT

PASS=0
FAIL=0

check() {
    local name="$1"
    local status="$2"
    local body="$3"
    local expect_status="${4:-200}"
    if [[ "$status" -eq "$expect_status" ]]; then
        echo "  PASS  $name (HTTP $status)"
        PASS=$((PASS + 1))
    else
        echo "  FAIL  $name — expected HTTP $expect_status, got HTTP $status"
        echo "        body: $body"
        FAIL=$((FAIL + 1))
    fi
}

api_post() {
    local path="$1"
    local data="${2:-}"
    curl -s -o /tmp/lol_resp_body -w "%{http_code}" \
        -X POST "$BASE_URL$path" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -c "$COOKIE_JAR" -b "$COOKIE_JAR" \
        ${data:+--data-urlencode "dummy=1" -d "$data"}
}

echo "=== LoL Team Companion API Tests ==="
echo "Server: $BASE_URL"
echo ""

# --- Health ---
echo "[ Health ]"
status=$(curl -s -o /tmp/lol_resp_body -w "%{http_code}" "$BASE_URL/healthz")
body=$(cat /tmp/lol_resp_body)
check "GET /healthz" "$status" "$body"

# --- Public pages (SSR) ---
echo ""
echo "[ Public Pages ]"
for path in "/" "/auth/login" "/auth/register"; do
    status=$(curl -s -o /dev/null -w "%{http_code}" -c "$COOKIE_JAR" -b "$COOKIE_JAR" "$BASE_URL$path")
    check "GET $path" "$status" "" 200
done

# --- Register (may 200 or 303 redirect) ---
echo ""
echo "[ Auth ]"
status=$(curl -s -o /tmp/lol_resp_body -w "%{http_code}" \
    -X POST "$BASE_URL/api/register_action" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=testuser&email=${EMAIL}&password=${PASSWORD}" \
    -c "$COOKIE_JAR" -b "$COOKIE_JAR" \
    -L --max-redirs 0 2>/dev/null || true)
body=$(cat /tmp/lol_resp_body 2>/dev/null || true)
# 200 = success JSON, 4xx = already registered or validation error — both are "reachable"
if [[ "$status" -ge 200 && "$status" -lt 500 ]]; then
    echo "  PASS  POST /api/register_action (HTTP $status — reachable)"
    PASS=$((PASS + 1))
else
    echo "  FAIL  POST /api/register_action (HTTP $status)"
    FAIL=$((FAIL + 1))
fi

# --- Login ---
status=$(curl -s -o /tmp/lol_resp_body -w "%{http_code}" \
    -X POST "$BASE_URL/api/login_action" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "email=${EMAIL}&password=${PASSWORD}" \
    -c "$COOKIE_JAR" -b "$COOKIE_JAR")
body=$(cat /tmp/lol_resp_body)
if [[ "$status" -eq 200 && "$body" != *'"Err"'* ]]; then
    echo "  PASS  POST /api/login_action (HTTP $status)"
    PASS=$((PASS + 1))
    LOGGED_IN=1
else
    echo "  WARN  POST /api/login_action — HTTP $status (may need valid credentials)"
    echo "        body: $body"
    LOGGED_IN=0
fi

# --- Authenticated server functions (only if logged in) ---
if [[ "$LOGGED_IN" -eq 1 ]]; then
    echo ""
    echo "[ Authenticated Server Functions ]"

    for fn_name in "get_current_user" "list_drafts" "list_trees" "get_pool"; do
        status=$(curl -s -o /tmp/lol_resp_body -w "%{http_code}" \
            -X POST "$BASE_URL/api/$fn_name" \
            -H "Content-Type: application/x-www-form-urlencoded" \
            -c "$COOKIE_JAR" -b "$COOKIE_JAR")
        body=$(cat /tmp/lol_resp_body)
        check "POST /api/$fn_name" "$status" "$body"
    done
else
    echo ""
    echo "[ Authenticated tests skipped — login failed ]"
    echo "  Tip: ./scripts/api_test.sh $BASE_URL your@email.com yourpassword"
fi

# --- Summary ---
echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
[[ "$FAIL" -eq 0 ]] && exit 0 || exit 1
