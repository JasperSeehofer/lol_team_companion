#!/usr/bin/env bash
# Polls the dev server until it responds or timeout is reached.
# Usage: ./scripts/wait_for_server.sh [timeout_seconds] [base_url]
set -euo pipefail

TIMEOUT="${1:-60}"
BASE_URL="${2:-http://127.0.0.1:3020}"

echo "Waiting for $BASE_URL/healthz (timeout: ${TIMEOUT}s) ..."

elapsed=0
while [[ $elapsed -lt $TIMEOUT ]]; do
    if curl -sf "$BASE_URL/healthz" > /dev/null 2>&1; then
        echo "Server ready after ${elapsed}s."
        exit 0
    fi
    sleep 1
    elapsed=$((elapsed + 1))
done

echo "ERROR: server did not respond within ${TIMEOUT}s"
exit 1
