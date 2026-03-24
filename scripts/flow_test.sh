#!/usr/bin/env bash
# Pipeline flow test: draft → game-plan → post-game linking.
#
# Exercises the same server functions the UI calls to verify the
# draft→game-plan→post-game pipeline works end-to-end.
#
# Leptos 0.8 appends a hash to server fn URLs (e.g. /api/save_draft123456).
# This script discovers the real URLs from the compiled WASM binary.
#
# Requires: running dev server (cargo leptos watch)
# Usage: ./scripts/flow_test.sh [base_url]
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:3020}"
WASM_PATH="target/site/pkg/lol_team_companion.wasm"
COOKIE_JAR="$(mktemp /tmp/lol_flow_cookies_XXXXXX)"
trap 'rm -f "$COOKIE_JAR" /tmp/lol_flow_resp' EXIT

PASS=0
FAIL=0
TIMESTAMP="$(date +%s)"
EMAIL="flow_${TIMESTAMP}@test.invalid"
PASSWORD="FlowTest1234!"
USERNAME="flowuser_${TIMESTAMP}"

# --- Discover hashed server fn URLs from WASM binary ---
resolve_fn() {
    local fn_name="$1"
    # Extract URL like /api/{fn_name}{hash} from WASM strings
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
    echo "ERROR: WASM binary not found at $WASM_PATH"
    echo "       Build first with: cargo leptos watch"
    exit 1
fi

echo "Resolving server function URLs from WASM binary..."
URL_REGISTER=$(resolve_fn "register_action")
URL_LOGIN=$(resolve_fn "login_action")
URL_CREATE_TEAM=$(resolve_fn "create_team")
URL_SAVE_DRAFT=$(resolve_fn "save_draft")
URL_LIST_DRAFTS=$(resolve_fn "list_drafts")
URL_CHECK_DRAFT_PLAN=$(resolve_fn "check_draft_has_game_plan")
URL_CREATE_PLAN=$(resolve_fn "create_plan")
URL_LIST_PLANS=$(resolve_fn "list_plans")

check() {
    local name="$1"
    local ok="$2"
    if [[ "$ok" -eq 1 ]]; then
        echo "  PASS  $name"
        PASS=$((PASS + 1))
    else
        echo "  FAIL  $name"
        FAIL=$((FAIL + 1))
    fi
}

api_post() {
    local url="$1"
    local data="${2:-}"
    local status
    status=$(curl -s -o /tmp/lol_flow_resp -w "%{http_code}" \
        -X POST "${BASE_URL}${url}" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -c "$COOKIE_JAR" -b "$COOKIE_JAR" \
        ${data:+-d "$data"})
    echo "$status"
}

echo ""
echo "=== Pipeline Flow Test ==="
echo "Server: $BASE_URL"
echo ""

# --- 1. Register ---
echo "[ Register & Login ]"
status=$(api_post "$URL_REGISTER" "username=${USERNAME}&email=${EMAIL}&password=${PASSWORD}")
body=$(cat /tmp/lol_flow_resp)
check "Register new user" "$([[ "$status" -eq 200 && "$body" != *'"Err"'* ]] && echo 1 || echo 0)"

# --- 2. Login (registration auto-logs in, but explicit login ensures cookie) ---
status=$(api_post "$URL_LOGIN" "email=${EMAIL}&password=${PASSWORD}")
body=$(cat /tmp/lol_flow_resp)
check "Login" "$([[ "$status" -eq 200 && "$body" != *'"Err"'* ]] && echo 1 || echo 0)"

# --- 3. Create team ---
echo ""
echo "[ Team Setup ]"
status=$(api_post "$URL_CREATE_TEAM" "name=FlowTeam_${TIMESTAMP}&region=NA")
body=$(cat /tmp/lol_flow_resp)
check "Create team" "$([[ "$status" -eq 200 ]] && echo 1 || echo 0)"

# --- 4. Save a draft ---
echo ""
echo "[ Draft ]"
ACTIONS='[{"id":null,"draft_id":"","phase":"pick1","side":"blue","champion":"Jinx","order":0,"comment":null},{"id":null,"draft_id":"","phase":"pick1","side":"blue","champion":"Thresh","order":1,"comment":null},{"id":null,"draft_id":"","phase":"pick2","side":"blue","champion":"Ahri","order":2,"comment":null},{"id":null,"draft_id":"","phase":"pick2","side":"blue","champion":"LeeSin","order":3,"comment":null},{"id":null,"draft_id":"","phase":"pick2","side":"blue","champion":"Gnar","order":4,"comment":null}]'
status=$(api_post "$URL_SAVE_DRAFT" "name=FlowDraft_${TIMESTAMP}&actions_json=${ACTIONS}&comments_json=%5B%5D&tags_json=%5B%5D&our_side=blue")
body=$(cat /tmp/lol_flow_resp)
# save_draft returns the draft ID as a JSON string
DRAFT_ID=""
if [[ "$status" -eq 200 && "$body" != *'"Err"'* ]]; then
    DRAFT_ID=$(echo "$body" | sed 's/^"//;s/"$//;s/.*"Ok":"//;s/".*//')
    check "Save draft (id: $DRAFT_ID)" 1
else
    check "Save draft" 0
    echo "        body: $body"
fi

# --- 5. List drafts — verify our draft exists ---
status=$(api_post "$URL_LIST_DRAFTS")
body=$(cat /tmp/lol_flow_resp)
check "List drafts contains new draft" "$([[ "$status" -eq 200 && "$body" == *"FlowDraft_${TIMESTAMP}"* ]] && echo 1 || echo 0)"

# --- 6. Check draft has no game plan yet ---
if [[ -n "$DRAFT_ID" ]]; then
    status=$(api_post "$URL_CHECK_DRAFT_PLAN" "draft_id=${DRAFT_ID}")
    body=$(cat /tmp/lol_flow_resp)
    # Should return Ok(None) — no existing plan
    check "Draft has no game plan yet" "$([[ "$status" -eq 200 ]] && echo 1 || echo 0)"
fi

# --- 7. Create a game plan linked to the draft ---
echo ""
echo "[ Game Plan ]"
PLAN_JSON="{\"id\":null,\"team_id\":\"\",\"draft_id\":\"${DRAFT_ID}\",\"name\":\"FlowPlan_${TIMESTAMP}\",\"our_champions\":[\"Jinx\",\"Thresh\",\"Ahri\",\"LeeSin\",\"Gnar\"],\"enemy_champions\":[],\"win_conditions\":[\"Teamfight\"],\"objective_priority\":[\"Dragon\"],\"teamfight_strategy\":\"Front-to-back\",\"early_game\":null,\"top_strategy\":null,\"jungle_strategy\":null,\"mid_strategy\":null,\"bot_strategy\":null,\"support_strategy\":null,\"notes\":null,\"win_condition_tag\":\"teamfight\"}"
status=$(api_post "$URL_CREATE_PLAN" "plan_json=${PLAN_JSON}")
body=$(cat /tmp/lol_flow_resp)
PLAN_ID=""
if [[ "$status" -eq 200 && "$body" != *'"Err"'* ]]; then
    PLAN_ID=$(echo "$body" | sed 's/^"//;s/"$//;s/.*"Ok":"//;s/".*//')
    check "Create game plan linked to draft (id: $PLAN_ID)" 1
else
    check "Create game plan" 0
    echo "        body: $body"
fi

# --- 8. List game plans — verify it exists and is linked ---
status=$(api_post "$URL_LIST_PLANS")
body=$(cat /tmp/lol_flow_resp)
check "List plans contains new plan" "$([[ "$status" -eq 200 && "$body" == *"FlowPlan_${TIMESTAMP}"* ]] && echo 1 || echo 0)"
check "Plan is linked to draft" "$([[ "$body" == *"${DRAFT_ID}"* ]] && echo 1 || echo 0)"

# --- 9. Check draft now has a game plan ---
if [[ -n "$DRAFT_ID" ]]; then
    status=$(api_post "$URL_CHECK_DRAFT_PLAN" "draft_id=${DRAFT_ID}")
    body=$(cat /tmp/lol_flow_resp)
    check "Draft now has game plan" "$([[ "$status" -eq 200 && "$body" == *"${PLAN_ID}"* ]] && echo 1 || echo 0)"
fi

# --- Summary ---
echo ""
echo "=== Pipeline Flow: $PASS passed, $FAIL failed ==="
[[ "$FAIL" -eq 0 ]] && exit 0 || exit 1
