#!/usr/bin/env bash
# Quick health check against the running dev server.
# Usage: ./scripts/health.sh [base_url]
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:3002}"

echo "Checking $BASE_URL/healthz ..."
response=$(curl -sf "$BASE_URL/healthz") || {
    echo "ERROR: server not reachable at $BASE_URL"
    exit 1
}

echo "$response" | python3 -m json.tool 2>/dev/null || echo "$response"

db_status=$(echo "$response" | python3 -c "import sys,json; print(json.load(sys.stdin)['db'])" 2>/dev/null || echo "unknown")
if [[ "$db_status" != "ok" ]]; then
    echo "WARNING: DB status is '$db_status'"
    exit 1
fi

echo "Health check passed."
