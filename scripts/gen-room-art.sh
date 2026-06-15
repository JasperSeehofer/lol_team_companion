#!/usr/bin/env bash
# Generate room art via BFL EU FLUX.2 [pro] endpoint.
#
# Prompt passing: pass the prompt via the --prompt flag OR the PROMPT env var.
# The prompt should follow Path A vocabulary (Aurelian Reach / Sundered Hollow).
# Do NOT use Riot proper nouns, champion names, or "in the style of [proper noun]".
#
# Usage:
#   PROMPT="painterly scene..." ./scripts/gen-room-art.sh <room> <region> <plane-name> <seed>
#   ./scripts/gen-room-art.sh --prompt "painterly scene..." <room> <region> <plane-name> <seed>
#
# Arguments:
#   <room>         Room ID: solo-journey | war-room | library | throne
#   <region>       Region ID: demacia | pandemonium
#   <plane-name>   Output name: plane-sky | plane-mid | plane-near | overlay-{axis}-{1,2,3}
#   <seed>         Integer seed for reproducibility — record in AI-IMAGES.md
#
# Requires: curl, jq, BFL_KEY in environment
# Output: /tmp/<room>/<region>/<plane-name>.png (downloaded PNG from BFL EU)
#
# Records: echo seed + output path on success for AI-IMAGES.md logging.
# NEVER echoes BFL_KEY into stdout/logs.
#
# Offline only — the server binary never calls this script.
set -euo pipefail

# --- Parse --prompt flag ---
PROMPT_ARG=""
POSITIONAL=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --prompt)
            PROMPT_ARG="$2"
            shift 2
            ;;
        *)
            POSITIONAL+=("$1")
            shift
            ;;
    esac
done
set -- "${POSITIONAL[@]}"

ROOM="${1:?Usage: gen-room-art.sh [--prompt TEXT] <room> <region> <plane-name> <seed>}"
REGION="${2:?Usage: gen-room-art.sh [--prompt TEXT] <room> <region> <plane-name> <seed>}"
PLANE="${3:?Usage: gen-room-art.sh [--prompt TEXT] <room> <region> <plane-name> <seed>}"
SEED="${4:?Usage: gen-room-art.sh [--prompt TEXT] <room> <region> <plane-name> <seed>}"

# Resolve prompt: --prompt flag takes precedence over PROMPT env var
FINAL_PROMPT="${PROMPT_ARG:-${PROMPT:-}}"
if [[ -z "$FINAL_PROMPT" ]]; then
    echo "Error: provide the prompt via --prompt flag or PROMPT env var" >&2
    exit 1
fi

# Guard API key — never echoed
: "${BFL_KEY:?BFL_KEY must be set (export BFL_KEY=...); never hardcode}"

# --- Setup ---
OUT_DIR="/tmp/${ROOM}/${REGION}"
mkdir -p "$OUT_DIR"
OUT_PATH="${OUT_DIR}/${PLANE}.png"
RESP_TMP="$(mktemp /tmp/bfl_resp_XXXXXX.json)"

trap 'rm -f "$RESP_TMP"' EXIT

echo "Submitting to BFL EU: ${ROOM}/${REGION}/${PLANE} (seed=${SEED})"

# --- 1. Submit generation ---
RESPONSE=$(curl -s -f -X POST "https://api.eu.bfl.ai/v1/flux-2-pro" \
    -H "x-key: ${BFL_KEY}" \
    -H "Content-Type: application/json" \
    -d "{
        \"prompt\": $(echo "$FINAL_PROMPT" | jq -Rs .),
        \"width\": 1920,
        \"height\": 1080,
        \"steps\": 28,
        \"seed\": ${SEED}
    }")

echo "$RESPONSE" > "$RESP_TMP"

POLLING_URL=$(jq -r '.polling_url' "$RESP_TMP")
if [[ -z "$POLLING_URL" || "$POLLING_URL" = "null" ]]; then
    echo "Error: BFL EU did not return polling_url. Response:" >&2
    jq . "$RESP_TMP" >&2
    exit 1
fi

echo "Polling: ${POLLING_URL}"

# --- 2. Poll until Ready ---
STATUS="Pending"
until [[ "$STATUS" = "Ready" ]]; do
    sleep 2
    POLL_RESP=$(curl -s -f -H "x-key: ${BFL_KEY}" "$POLLING_URL")
    STATUS=$(echo "$POLL_RESP" | jq -r '.status')
    echo "  status: ${STATUS}"
done

# --- 3. Download result.sample (PNG) ---
SAMPLE_URL=$(curl -s -f -H "x-key: ${BFL_KEY}" "$POLLING_URL" | jq -r '.result.sample')
if [[ -z "$SAMPLE_URL" || "$SAMPLE_URL" = "null" ]]; then
    echo "Error: could not extract result.sample URL" >&2
    exit 1
fi

curl -L -s -f -o "$OUT_PATH" "$SAMPLE_URL"

echo ""
echo "Done."
echo "  Seed: ${SEED}"
echo "  Output: ${OUT_PATH}"
echo ""
echo "Record in AI-IMAGES.md:"
echo "  - Path: public/img/rooms/${ROOM}/${REGION}/${PLANE}.webp"
echo "  - Seed: ${SEED}"
echo "  - Compute-path: BFL EU FLUX.2 [pro] api.eu.bfl.ai/v1/flux-2-pro steps=28"
