#!/usr/bin/env bash
# lint-gallery.sh: the design lint gates of scripts/lint_design.sh, applied to
# the gallery's HTML/CSS/SVG (lint_design.sh itself only scans src/**/*.rs and
# takes no paths). The rule definitions (PALETTE, PREFIX and every `rule ...`
# line) are sourced from scripts/lint_design.sh, so the patterns stay one copy.
# Gallery-only extras: CSS font sizes under 12px, and raw colour functions
# outside the generated tokens.css (rubric V1).
#
#   .planning/missions/M151/gallery/tools/lint-gallery.sh [files...]
# Default: every *.html *.css *.svg under gallery/ (out/ excluded). Exit 1 on a hit.
# Escape hatch as in the app: `lint:allow(<rule>)` on the same line.
set -uo pipefail
export LC_ALL=C.UTF-8
HERE=$(cd "$(dirname "$0")" && pwd)
GALLERY=$(dirname "$HERE")
REPO=$(cd "$GALLERY/../../../.." && pwd)
SRC="$REPO/scripts/lint_design.sh"

# shellcheck disable=SC1090
source <(sed -n '/^PALETTE=/,/^declare -A COUNT/p' "$SRC" | sed '$d')
[ "${#NAMES[@]}" -gt 10 ] || { echo "lint-gallery: could not read rules from $SRC"; exit 2; }
rule css-px-text 'font-size\s*:\s*((\d|1[01])(\.\d+)?px|0?\.[0-6]\d*rem)\b' '' \
  'font size below 12px; use the type scale (--text-caption is the floor)'
rule raw-colour '\b(rgba?|hsla?|oklch|oklab|lab|lch|hwb)\(' '(^|/)tokens\.css$' \
  'raw colour function outside tokens.css; use var(--color-...)'
rule filter-blur 'filter\s*:[^;]*blur\(' '' 'blur is not part of the system'
rule g12-css 'outline\s*:\s*(none|0)\b' '' 'outline removed; keep the 2px focus outline'
# The app exempts its primitives from rounded-full; the gallery has no such
# folder, so a round avatar/toggle needs lint:allow(rounded-full) with a reason.
# Mirror rounded-full for CSS: border-radius 50% / 9999px.
rule css-round 'border-radius\s*:\s*(50%|9999px|999px|100%)' '' \
  'round shape outside avatars/toggles/map tokens; mark a legitimate one lint:allow(css-round)'

if [ "$#" -gt 0 ]; then FILES=("$@"); else
  mapfile -t FILES < <(find "$GALLERY" \( -path "$GALLERY/out" -o -path "$GALLERY/tools" \) -prune -o \
    -type f \( -name '*.html' -o -name '*.css' -o -name '*.svg' \) -print | sort)
fi
[ "${#FILES[@]}" -gt 0 ] || { echo "lint-gallery: no files"; exit 0; }

fail=0 broken=0
for i in "${!NAMES[@]}"; do
  name=${NAMES[$i]} pat=${PATTERNS[$i]} ex=${EXEMPT[$i]}
  [ "$name" = region ] && pat='InitialTheme|data-theme'   # region= is Rust-prop syntax only
  out=$(grep -nHP -- "$pat" "${FILES[@]}" /dev/null 2>/tmp/lint-gallery.err); rc=$?
  [ "$rc" -gt 1 ] && { echo "::error::grep failed for [$name]: $(head -c 200 /tmp/lint-gallery.err)"; broken=1; continue; }
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    file=${line%%:*}
    if [ -n "$ex" ] && [ "$ex" != '^src/components/ui/' ] && [[ "$file" =~ $ex ]]; then continue; fi
    case "$line" in *"lint:allow($name)"*) continue ;; esac
    echo "[$name] ${line#"$GALLERY"/}"
    echo "    -> ${MESSAGES[$i]}"
    fail=1
  done <<< "$out"
done
[ "$broken" = 1 ] && fail=1
[ "$fail" = 0 ] && echo "Gallery lint: OK (${#FILES[@]} files, ${#NAMES[@]} rules)."
exit $fail
