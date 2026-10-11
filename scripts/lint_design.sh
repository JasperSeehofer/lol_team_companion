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
# Shared models build URLs the browser loads, so the external-url rule
# also covers them.
mapfile -t MODEL_FILES < <(find src/models -name '*.rs' | sort)

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
# The browser loads nothing from a third party (fonts, images, scripts):
# no absolute URL to another host in client code (SVG namespaces aside).
rule external-url '^(?!\s*//).*https?://(?!www\.w3\.org)[a-z0-9-]+(\.[a-z0-9-]+)+' '' \
  'third-party URL in client code; serve it from the app (see /img/...)'
# A text tone with an opacity modifier (`text-muted/60`) is not in the
# contrast table and falls below 4.5:1; muted is the faintest text tone.
rule faded-text '(?<![\w-])text-(primary|secondary|muted|dimmed|accent|success|warning|danger|info|win|loss)/\d+' '' \
  'faded text tone (opacity modifier) falls below AA; use a tone from the contrast table'

# Control and box edges at an opacity (`border-outline/50`) fall to about
# 2:1, below the 3:1 WCAG 1.4.11 asks of control boundaries. Use
# `border-outline` for controls and `border-rule` for decorative hairlines.
rule faint-outline '(?<![\w-])(border|divide|ring)-outline/\d+' '' \
  'outline token at an opacity falls below 3:1; use border-outline (controls) or border-rule (hairlines)'

declare -A COUNT
TOTAL=0
GREP_OUT=$(mktemp)
GREP_ERR=$(mktemp)
trap 'rm -f "$GREP_OUT" "$GREP_ERR"' EXIT
broken=0

# grep exits 0 (hits), 1 (no hits) or 2 (error: bad pattern, no PCRE, bad
# locale). An error must fail the gate, not read as "zero hits". Runs in
# this shell (no subshell) so `broken` sticks; hits go to $GREP_OUT.
run_grep() { # flags pattern [extra files...]
  local flags=$1 pattern=$2
  shift 2
  grep "$flags" -- "$pattern" "${FILES[@]}" "$@" /dev/null >"$GREP_OUT" 2>"$GREP_ERR"
  if [ "$?" -gt 1 ]; then
    echo "::error::design lint: grep failed for pattern '$pattern': $(head -c 300 "$GREP_ERR")"
    broken=1
  fi
}

count_rule() { # name pattern allow
  local name=$1 pattern=$2 allow=$3 line file
  if [ "$name" = external-url ]; then
    run_grep -nP "$pattern" "${MODEL_FILES[@]}"
  else
    run_grep -nP "$pattern"
  fi
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    file=${line%%:*}
    if [ -n "$allow" ] && [[ "$file" =~ $allow ]]; then continue; fi
    case "$line" in *"lint:allow($name)"*) continue ;; esac
    COUNT["$name"$'\t'"$file"]=$(( ${COUNT["$name"$'\t'"$file"]:-0} + 1 ))
    TOTAL=$((TOTAL + 1))
  done < "$GREP_OUT"
}

for i in "${!NAMES[@]}"; do
  count_rule "${NAMES[$i]}" "${PATTERNS[$i]}" "${EXEMPT[$i]}"
done

# G-12: every outline-none needs a focus-visible replacement on the same line.
run_grep -nE 'outline-none'
while IFS= read -r line; do
  [ -z "$line" ] && continue
  file=${line%%:*}
  case "$line" in *focus-visible:*|*"lint:allow(g12)"*) continue ;; esac
  COUNT["g12"$'\t'"$file"]=$(( ${COUNT["g12"$'\t'"$file"]:-0} + 1 ))
  TOTAL=$((TOTAL + 1))
done < "$GREP_OUT"

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

if [ "$broken" = 1 ]; then
  fail=1
fi
if [ "$fail" = 0 ]; then
  echo "Design lint: OK (every rule at zero)."
fi
exit $fail
