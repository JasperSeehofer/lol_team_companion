#!/usr/bin/env bash
# Design-system lint gates (docs/DESIGN.md section 9). Bash + GNU grep, no Node.
#
#   scripts/lint_design.sh    fail on any hit
#
# Each rule matches lines in src/**/*.rs (server, models and bin excluded).
# Every rule is hard zero: the ratcheting baseline used during M2 is gone.
#
# Escape hatch for a legitimate exception: put `lint:allow(<rule>)` in a
# comment on the same line.
set -uo pipefail
cd "$(dirname "$0")/.."
# grep -P needs a UTF-8 locale for the emoji code points.
export LC_ALL=C.UTF-8

mapfile -t FILES < <(find src -name '*.rs' \
  ! -path 'src/server/*' ! -path 'src/models/*' ! -path 'src/bin/*' | sort)

PALETTE='(red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose|slate|gray|zinc|neutral|stone)'
PREFIX='(text|bg|border|ring|fill|stroke|from|via|to|divide|outline|decoration|placeholder|caret|accent|shadow)'

# Rules: NAMES[i] uses PATTERNS[i] (PCRE), does not apply to files matching
# EXEMPT[i] ('' = applies everywhere) and reports MESSAGES[i].
NAMES=() PATTERNS=() EXEMPT=() MESSAGES=()
rule() { NAMES+=("$1"); PATTERNS+=("$2"); EXEMPT+=("$3"); MESSAGES+=("$4"); }

rule palette "\\b${PREFIX}-${PALETTE}-\\d{2,3}\\b|\\b(text|bg|border)-(white|black)\\b" '' \
  'raw Tailwind palette colour; use a semantic or domain token'
rule hex '^(?!\s*//).*(?<![\w&/])#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3})(?![\w-])' '' \
  'hex colour; use a token'
rule px-text 'text-\[(\d|1[01])(\.\d+)?px\]|text-\[0?\.[0-6]\d*rem\]' '' \
  'text below 12px; use the type scale (text-caption is the floor)'
rule rounded-full '\brounded-full\b' '^src/components/ui/' \
  'rounded-full outside avatars and toggles (primitives only)'
rule blur 'backdrop-blur|backdrop-filter' '' \
  'glass and blur are not part of the system'
rule gradient 'bg-gradient|linear-gradient|radial-gradient|conic-gradient' '' \
  'gradients only where they encode data (none yet)'
rule drop-shadow 'drop-shadow|text-shadow' '' \
  'no drop or text shadows; one overlay shadow token exists'
rule pulse 'animate-pulse' '^src/components/ui/skeleton\.rs$' \
  'animate-pulse outside the skeleton components'
rule ornament 'Cinzel|font-imperial|font-glitch|VT323|canvas-grain|FleurDeLis|GiltCorner|HeraldicDivider|RiotTape|CompanionSigil|[Pp]andemonium' '' \
  'retired ornament or theme'
# Typographic marks (stars, check, cross) are allowed; pictographs are not.
rule emoji '[\x{1F300}-\x{1FAFF}\x{2B50}\x{2B55}\x{FE0F}]|(?![\x{2605}\x{2606}\x{2713}\x{2715}\x{2717}])[\x{2600}-\x{27BF}]|\\u\{(?!(?:2605|2606|2713|2715|2717)\})(1[Ff][0-9A-Fa-f]{3}|2[67][0-9A-Fa-f]{2}|[Ff][Ee]0[Ff])\}' '' \
  'emoji in UI strings; use words or an SVG'
rule region 'region\s*==|(?<![_\w])region=|InitialTheme|data-theme' '' \
  'theme branching is gone; there is one theme'

declare -A COUNT
TOTAL=0

count_rule() { # name pattern allow
  local name=$1 pattern=$2 allow=$3 line file
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    file=${line%%:*}
    if [ -n "$allow" ] && [[ "$file" =~ $allow ]]; then continue; fi
    case "$line" in *"lint:allow($name)"*) continue ;; esac
    COUNT["$name"$'\t'"$file"]=$(( ${COUNT["$name"$'\t'"$file"]:-0} + 1 ))
    TOTAL=$((TOTAL + 1))
  done < <(grep -nP -- "$pattern" "${FILES[@]}" /dev/null 2>/dev/null)
}

for i in "${!NAMES[@]}"; do
  count_rule "${NAMES[$i]}" "${PATTERNS[$i]}" "${EXEMPT[$i]}"
done

# G-12: every outline-none needs a focus-visible replacement on the same line.
while IFS= read -r line; do
  [ -z "$line" ] && continue
  file=${line%%:*}
  case "$line" in *focus-visible:*|*"lint:allow(g12)"*) continue ;; esac
  COUNT["g12"$'\t'"$file"]=$(( ${COUNT["g12"$'\t'"$file"]:-0} + 1 ))
  TOTAL=$((TOTAL + 1))
done < <(grep -nE -- 'outline-none' "${FILES[@]}" /dev/null 2>/dev/null)

rule g12 'outline-none' '' 'outline-none without a focus-visible replacement on the same line'
fail=0
for i in "${!NAMES[@]}"; do
  name=${NAMES[$i]}
  msg=${MESSAGES[$i]}
  for key in "${!COUNT[@]}"; do
    [ "${key%%$'\t'*}" = "$name" ] || continue
    file=${key#*$'\t'}
    echo "::error file=$file::[$name] ${COUNT[$key]} hit(s): $msg"
    grep -nP -- "${PATTERNS[$i]}" "$file" 2>/dev/null | grep -v "lint:allow($name)" | head -5 | sed 's/^/    /'
    fail=1
  done
done

if [ "$fail" = 0 ]; then
  echo "Design lint: OK (every rule at zero)."
fi
exit $fail
