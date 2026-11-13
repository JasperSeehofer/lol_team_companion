#!/usr/bin/env bash
# capture-all.sh: capture the OG card (and its share-link variant) for every
# treatment whose art exists: G/out/og-<t>.png and og-<t>-share.png, 1200x630.
# Needs the gallery server (README: python3 -m http.server 8151 from the repo
# root, or GALLERY_BASE). A treatment without surfaces/art/<t>.json is reported
# and skipped. Exit 1 if any capture fails or a PNG is over 1 MB.
set -uo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
G=$(dirname "$HERE")
REPO=$(cd "$G/../../../.." && pwd)
cd "$REPO" || exit 2
rel=${G#"$REPO"/}
fail=0
[ "$#" -gt 0 ] || set -- t1 t2 t3
for t1 in "$@"; do
  {
    if [ ! -f "$G/surfaces/art/$t1.json" ]; then echo "skip $t1: no surfaces/art/$t1.json yet"; continue; fi
    for v in card share; do
      q="t=$t1"; out="$rel/out/og-$t1.png"
      [ "$v" = share ] && { q="t=$t1&v=share"; out="$rel/out/og-$t1-share.png"; }
      node "$rel/tools/capture.mjs" "$rel/og/og.html?$q" "$out" --og --settle 800 || { fail=1; continue; }
      size=$(stat -c %s "$out")
      [ "$size" -gt 1048576 ] && { echo "FAIL $out is $size bytes (over 1 MB)"; fail=1; }
    done
  }
done
exit $fail
