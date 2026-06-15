#!/usr/bin/env bash
# Verify room art asset budget against two caps (D-11):
#
#   1. Per-file cap:            no *.webp under public/img/rooms exceeds 120 KB
#   2. Per-room/region aggregate: sum of *.webp in each region dir <= 400 KB (409600 bytes)
#
# All violations are reported before exit (FAIL=0 accumulator pattern).
#
# Usage:
#   ./scripts/verify-budget.sh [<path>]
#
# Arguments:
#   <path>  Optional. Scope the check to one room or region directory.
#           Default: all of public/img/rooms
#
# Examples:
#   ./scripts/verify-budget.sh                                       # all rooms
#   ./scripts/verify-budget.sh public/img/rooms/solo-journey/        # one room
#   ./scripts/verify-budget.sh public/img/rooms/solo-journey/demacia/ # one region
#
# Exit codes:
#   0  All checks pass
#   1  One or more budget violations found (details printed to stdout)
#
# Mirrors the asset_budget CI job in .github/workflows/ci.yml.
# Offline only — the server binary never calls this script.
set -euo pipefail

ROOT="${1:-public/img/rooms}"

if [[ ! -d "$ROOT" ]]; then
    echo "Info: ${ROOT} does not exist — nothing to check (pass)"
    exit 0
fi

# Per-file cap: 120 KB
PER_FILE_CAP_BYTES=122880  # 120 * 1024

# Per-room/region aggregate cap: 400 KB
AGGREGATE_CAP_BYTES=409600  # 400 * 1024

FAIL=0

echo "Checking room art budget in: ${ROOT}"
echo ""

# --- Check 1: Per-file cap (120 KB) ---
echo "=== Per-file cap (120 KB) ==="
# Use -size +120k (find's unit 'k' = 1024 bytes)
OVER_FILES=$(find "$ROOT" -name "*.webp" -size +120k 2>/dev/null || true)
if [[ -n "$OVER_FILES" ]]; then
    echo "FAIL: files exceeding 120 KB cap:"
    echo "$OVER_FILES" | while IFS= read -r f; do
        SIZE=$(stat -c%s "$f" 2>/dev/null || stat -f%z "$f")
        echo "  ${f}: ${SIZE} bytes ($(( SIZE / 1024 )) KB)"
    done
    FAIL=1
else
    echo "PASS: no individual file exceeds 120 KB"
fi
echo ""

# --- Check 2: Per-room/region aggregate (400 KB = 409600 bytes) ---
echo "=== Per-room/region aggregate (400 KB) ==="

# Determine the scope: if ROOT is already a region dir (depth ≥ 3 from public/img/rooms),
# check just that region. If it's a room dir, iterate its regions. If it's the root,
# iterate all rooms and their regions.
FOUND_ANY_REGION=0

check_region() {
    local region_dir="$1"
    # Sum bytes of all *.webp in the region dir (non-recursive aggregate)
    local total
    total=$(find "$region_dir" -maxdepth 1 -name "*.webp" -exec du -b {} \; 2>/dev/null \
        | awk '{sum+=$1} END{print sum+0}')
    FOUND_ANY_REGION=1
    if [[ "$total" -gt "$AGGREGATE_CAP_BYTES" ]]; then
        echo "OVER BUDGET: ${region_dir} (${total} bytes, $(( total / 1024 )) KB — limit is 400 KB)"
        FAIL=1
    else
        echo "OK:   ${region_dir} (${total} bytes, $(( total / 1024 )) KB)"
    fi
}

# Detect the depth of ROOT relative to public/img/rooms
# Strategy: try to find region dirs under ROOT at various depths
for room in "${ROOT}"/*/; do
    [[ -d "$room" ]] || continue
    for region in "${room}"*/; do
        [[ -d "$region" ]] || continue
        check_region "$region"
    done
done

# If no room/region dirs found, try ROOT itself as a region dir
if [[ "$FOUND_ANY_REGION" -eq 0 ]]; then
    if find "$ROOT" -maxdepth 1 -name "*.webp" | grep -q .; then
        check_region "$ROOT"
    else
        echo "INFO: no *.webp files found under ${ROOT} — budget trivially satisfied"
    fi
fi

echo ""
if [[ "$FAIL" -eq 0 ]]; then
    echo "All budget checks passed."
else
    echo "Budget violations found — fix before committing art assets."
fi

exit $FAIL
