#!/usr/bin/env bash
# Encode a PNG to alpha-preserving WebP using ImageMagick.
#
# Produces a WebP file with:
#   - RGB quality: 80 (good fidelity for painterly art; recompress to 68 if over budget)
#   - Alpha quality: 100 (lossless alpha for transparent mid/near planes and overlays)
#
# Sky planes are opaque (no alpha channel), but the alpha-quality flag is harmless
# for them — they can also use plain -quality 80 without the define.
#
# Usage:
#   ./scripts/encode-webp.sh <input.png> <output.webp>
#
# Arguments:
#   <input.png>   Source PNG (opaque or with alpha channel)
#   <output.webp> Destination WebP path (parent directory is created if needed)
#
# Requires: ImageMagick (magick command), already installed for placeholder pipeline.
#
# Budget guidance (D-11):
#   Per-file cap: 120 KB (enforced by verify-budget.sh and CI asset_budget job)
#   If output exceeds cap, this script offers a second-pass at -quality 68.
#   Fallback option (Pitfall 3): resize to 1600x900 with `magick ... -resize 1600x900`
#   before encoding; CSS object-cover handles scale. Document in AI-IMAGES.md.
#
# Offline only — the server binary never calls this script.
set -euo pipefail

INPUT="${1:?Usage: encode-webp.sh <input.png> <output.webp>}"
OUTPUT="${2:?Usage: encode-webp.sh <input.png> <output.webp>}"

# Per-file budget cap in bytes: 120 KB
PER_FILE_CAP_BYTES=122880

if [[ ! -f "$INPUT" ]]; then
    echo "Error: input file not found: ${INPUT}" >&2
    exit 1
fi

mkdir -p "$(dirname "$OUTPUT")"

echo "Encoding: ${INPUT} -> ${OUTPUT}"

# --- Primary encode: RGB q80, alpha lossless ---
magick "$INPUT" \
    -quality 80 \
    -define webp:alpha-quality=100 \
    "$OUTPUT"

FILE_SIZE=$(stat -c%s "$OUTPUT" 2>/dev/null || stat -f%z "$OUTPUT")
echo "  Output size: ${FILE_SIZE} bytes ($(( FILE_SIZE / 1024 )) KB)"

# --- Optional second pass: recompress to q68 if over per-file cap ---
if [[ "$FILE_SIZE" -gt "$PER_FILE_CAP_BYTES" ]]; then
    echo "  WARNING: ${OUTPUT} exceeds ${PER_FILE_CAP_BYTES} bytes (120 KB cap)."
    echo "  Attempting second pass at -quality 68 ..."
    # Note: if still over budget after q68, consider resize to 1600x900 (Pitfall 3):
    #   magick "$INPUT" -resize 1600x900 -quality 68 -define webp:alpha-quality=100 "$OUTPUT"
    # CSS object-cover on <img> handles the scale change. Document in AI-IMAGES.md.
    magick "$INPUT" \
        -quality 68 \
        -define webp:alpha-quality=100 \
        "$OUTPUT"
    FILE_SIZE=$(stat -c%s "$OUTPUT" 2>/dev/null || stat -f%z "$OUTPUT")
    echo "  Second-pass size: ${FILE_SIZE} bytes ($(( FILE_SIZE / 1024 )) KB)"
    if [[ "$FILE_SIZE" -gt "$PER_FILE_CAP_BYTES" ]]; then
        echo "  WARNING: still over 120 KB cap after q68 recompress."
        echo "  Next step: resize to 1600x900 or reduce FLUX generation complexity."
        echo "  Run verify-budget.sh after placing file to confirm budget status."
    fi
fi

echo ""
echo "Done: ${OUTPUT}"
echo "Next steps:"
echo "  1. mv ${OUTPUT} public/img/rooms/<room>/<region>/<plane-name>.webp"
echo "  2. ./scripts/verify-budget.sh public/img/rooms/<room>/<region>/"
