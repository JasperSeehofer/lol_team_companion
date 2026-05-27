#!/usr/bin/env bash
# Reclaim disk from orphaned Rust target directories left by GSD phase execution.
#
# Two classes of orphans:
#   1. .claude/worktrees/agent-* whose lock file points to a dead PID
#      (the post-merge `git worktree remove --force` never ran because the
#      previous orchestrator crashed mid-wave).
#   2. /tmp/phase*-target, /tmp/lol*-target — executor agents that redirected
#      CARGO_TARGET_DIR to /tmp and never cleaned up.
#
# Idempotent. Safe to run with `cargo leptos watch` active — only touches
# orphans, never the live target/. Never deletes the main checkout's target/.

set -uo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

freed_kb=0
report=()

# --- 1. Stale worktrees in .claude/worktrees/ ----------------------------------
if [ -d .claude/worktrees ]; then
    for wt in .claude/worktrees/agent-*; do
        [ -d "$wt" ] || continue
        wt_name="$(basename "$wt")"
        lock_file=".git/worktrees/${wt_name}/locked"

        alive=false
        if [ -f "$lock_file" ]; then
            # Extract PID from "claude agent agent-<id> (pid NNNNN)" format
            pid="$(grep -oE 'pid [0-9]+' "$lock_file" 2>/dev/null | awk '{print $2}')"
            if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
                alive=true
            fi
        else
            # No lock file — also a candidate; check for any cargo/claude proc
            # holding files inside it
            if lsof +D "$wt" 2>/dev/null | grep -q .; then
                alive=true
            fi
        fi

        if [ "$alive" = "true" ]; then
            report+=("  KEEP   $wt (active, pid=$pid)")
            continue
        fi

        size_kb=$(du -sk "$wt" 2>/dev/null | awk '{print $1}')
        # Force-unlock + remove via git so .git/worktrees metadata is consistent
        git worktree unlock "$wt" 2>/dev/null || true
        if git worktree remove --force "$wt" 2>/dev/null; then
            git branch -D "worktree-${wt_name}" 2>/dev/null || true
            freed_kb=$((freed_kb + size_kb))
            report+=("  REMOVE $wt (${size_kb} KB, stale)")
        else
            # Filesystem-level fallback if git refuses
            rm -rf "$wt" 2>/dev/null && {
                git worktree prune 2>/dev/null || true
                git branch -D "worktree-${wt_name}" 2>/dev/null || true
                freed_kb=$((freed_kb + size_kb))
                report+=("  REMOVE $wt (${size_kb} KB, fs fallback)")
            } || report+=("  FAIL   $wt (manual cleanup needed)")
        fi
    done
fi

git worktree prune 2>/dev/null || true

# --- 2. Orphaned /tmp targets --------------------------------------------------
shopt -s nullglob
for d in /tmp/phase*-target /tmp/lol*-target /tmp/*cargo-target* /tmp/CARGO_TARGET_*; do
    [ -d "$d" ] || continue
    # Skip if anything currently has it open
    if lsof +D "$d" 2>/dev/null | grep -q .; then
        report+=("  KEEP   $d (in use)")
        continue
    fi
    size_kb=$(du -sk "$d" 2>/dev/null | awk '{print $1}')
    rm -rf "$d" && {
        freed_kb=$((freed_kb + size_kb))
        report+=("  REMOVE $d (${size_kb} KB)")
    } || report+=("  FAIL   $d")
done
shopt -u nullglob

# --- 3. Old GSD tmp manifests (cosmetic, but they accumulate) ------------------
manifests=(/tmp/gsd-worktree-wave-*.json)
if [ ${#manifests[@]} -gt 0 ] && [ -f "${manifests[0]}" ]; then
    # Remove ones older than 1 day
    find /tmp -maxdepth 1 -name 'gsd-worktree-wave-*.json' -mtime +0 -delete 2>/dev/null || true
fi

# --- Report --------------------------------------------------------------------
if [ ${#report[@]} -eq 0 ]; then
    echo "clean-orphan-targets: nothing to reclaim"
    exit 0
fi

freed_mb=$((freed_kb / 1024))
echo "clean-orphan-targets: reclaimed ${freed_mb} MB"
for line in "${report[@]}"; do
    echo "$line"
done
