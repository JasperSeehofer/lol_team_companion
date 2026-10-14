#!/usr/bin/env bash
# deploy/land.sh — Team Companion's repo-owned half of the garden's `71-mission-session.sh land <slug>`
# (M137 R1b, 2026-10-14). The wrapper owns the sequence (verify gate → trial merge → check → merge →
# push → deploy hook → card); this hook says what "check" and "deploy" mean HERE.
#
#   deploy/land.sh check              — on the merged tree (index state, before the merge commit):
#                                       fmt + design lint locally, and CI must be green for the
#                                       branch tip being merged (MERGE_HEAD). The Rust test suites
#                                       are NOT compiled here: a debug build of this crate needs
#                                       ~3 GB and stalls the brain host (M137 R1, twice); CI is the
#                                       test gate for this repo.
#   deploy/land.sh deploy <old> <new> — NO-OP by design. This is a live product with real users:
#                                       the image is built and shipped by the garden's
#                                       82-release.sh (build | ship), and the switch to the new tag
#                                       is RUNBOOK §4 behind its own two-step card. Printing the
#                                       pointer here is the whole job; nothing on the VM changes.
set -euo pipefail
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_DIR"
export PATH="$HOME/.cargo/bin:$PATH"

ci_green_for() { # <sha> — every job of the newest completed run on this sha concluded success
  local sha="$1" api="https://api.github.com/repos/JasperSeehofer/lol_team_companion/actions"
  curl -sf --max-time 20 "$api/runs?head_sha=$sha&per_page=5" | python3 -c '
import sys, json
runs = [r for r in json.load(sys.stdin).get("workflow_runs", []) if r.get("name") == "CI"]
if not runs: print("no CI run for this sha yet"); sys.exit(2)
r = runs[0]
if r["status"] != "completed": print("CI run", r["id"], "still", r["status"]); sys.exit(3)
if r["conclusion"] != "success": print("CI run", r["id"], "concluded", r["conclusion"]); sys.exit(4)
print("CI run", r["id"], "success")'
}

case "${1:-}" in
  check)
    cargo fmt --check
    echo "  fmt ok"
    scripts/lint_design.sh
    tip="$(git rev-parse -q --verify MERGE_HEAD 2>/dev/null || git rev-parse HEAD)"
    echo "  CI gate for $tip:"
    ci_green_for "$tip"
    ;;
  deploy)
    old="${2:-?}" new="${3:-?}"
    cat <<MSG
  deploy: NO-OP for Team Companion (live product). main moved $old -> $new.
  Next, each behind its own card: 82-release.sh team-companion build && ... ship, then switch
  (RUNBOOK §4: pre-update backup -> COMPANION_TAG=<sha> -> docker compose up -d -> healthz).
  A changed deploy/Caddyfile needs: docker compose up -d --force-recreate caddy.
MSG
    ;;
  *) echo "usage: deploy/land.sh check | deploy <old> <new>" >&2; exit 2 ;;
esac
