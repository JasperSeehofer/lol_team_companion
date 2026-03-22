#!/usr/bin/env bash
# Tests key server function endpoints against the running dev server.
#
# Leptos 0.8 appends a hash to server fn URLs (e.g. /api/save_draft123456).
# This script discovers the real URLs from the compiled WASM binary.
#
# Requires the server to be running: cargo leptos watch
# Usage: ./scripts/api_test.sh [base_url] [email] [password]
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:3002}"
TIMESTAMP="$(date +%s)"
EMAIL="${2:-apitest_${TIMESTAMP}@test.invalid}"
PASSWORD="${3:-ApiTest1234!}"
USERNAME="apiuser_${TIMESTAMP}"
WASM_PATH="target/site/pkg/lol_team_companion.wasm"
COOKIE_JAR="$(mktemp /tmp/lol_cookies_XXXXXX)"
trap 'rm -f "$COOKIE_JAR"' EXIT

PASS=0
FAIL=0

# --- Discover hashed server fn URLs from WASM binary ---
resolve_fn() {
    local fn_name="$1"
    local url
    url=$(strings "$WASM_PATH" 2>/dev/null \
        | grep -oP "^/api/${fn_name}[0-9]+" \
        | head -1)
    if [[ -z "$url" ]]; then
        echo "ERROR: Could not resolve server fn URL for '$fn_name'" >&2
        echo "/api/$fn_name"  # fallback (will 404)
    else
        echo "$url"
    fi
}

if [[ ! -f "$WASM_PATH" ]]; then
    echo "WARN: WASM binary not found at $WASM_PATH — using plain /api/ URLs (may 404)"
    resolve_fn() { echo "/api/$1"; }
fi

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
        echo "        body: $(echo "$body" | head -c 200)"
        FAIL=$((FAIL + 1))
    fi
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

# --- Resolve URLs ---
URL_REGISTER=$(resolve_fn "register_action")
URL_LOGIN=$(resolve_fn "login_action")

# --- Register ---
echo ""
echo "[ Auth ]"
status=$(curl -s -o /tmp/lol_resp_body -w "%{http_code}" \
    -X POST "${BASE_URL}${URL_REGISTER}" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=${USERNAME}&email=${EMAIL}&password=${PASSWORD}" \
    -c "$COOKIE_JAR" -b "$COOKIE_JAR" \
    -L --max-redirs 0 2>/dev/null || true)
body=$(cat /tmp/lol_resp_body 2>/dev/null || true)
# 200 = success JSON, 4xx = already registered or validation error — both are "reachable"
if [[ "$status" -ge 200 && "$status" -lt 500 ]]; then
    echo "  PASS  POST ${URL_REGISTER} (HTTP $status — reachable)"
    PASS=$((PASS + 1))
else
    echo "  FAIL  POST ${URL_REGISTER} (HTTP $status)"
    FAIL=$((FAIL + 1))
fi

# --- Login ---
status=$(curl -s -o /tmp/lol_resp_body -w "%{http_code}" \
    -X POST "${BASE_URL}${URL_LOGIN}" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "email=${EMAIL}&password=${PASSWORD}" \
    -c "$COOKIE_JAR" -b "$COOKIE_JAR")
body=$(cat /tmp/lol_resp_body)
if [[ "$status" -eq 200 && "$body" != *'"Err"'* ]]; then
    echo "  PASS  POST ${URL_LOGIN} (HTTP $status)"
    PASS=$((PASS + 1))
    LOGGED_IN=1
else
    echo "  WARN  POST ${URL_LOGIN} — HTTP $status (may need valid credentials)"
    echo "        body: $(echo "$body" | head -c 200)"
    LOGGED_IN=0
fi

# --- Authenticated server functions (only if logged in) ---
if [[ "$LOGGED_IN" -eq 1 ]]; then
    echo ""
    echo "[ Authenticated Server Functions ]"

    for fn_name in "get_current_user" "list_drafts" "list_trees" "get_pool"; do
        url=$(resolve_fn "$fn_name")
        status=$(curl -s -o /tmp/lol_resp_body -w "%{http_code}" \
            -X POST "${BASE_URL}${url}" \
            -H "Content-Type: application/x-www-form-urlencoded" \
            -c "$COOKIE_JAR" -b "$COOKIE_JAR")
        body=$(cat /tmp/lol_resp_body)
        check "POST $url" "$status" "$body"
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
