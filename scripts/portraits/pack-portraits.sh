#!/usr/bin/env bash
# Pack portrait directories into per-theme tar.gz archives with SHA256 checksums.
#
# Input:  portrait directory with {theme}/{size}/*.png structure
# Output: dist directory with {theme}.tar.gz + {theme}.sha256 per theme
#
# Usage:
#   ./scripts/portraits/pack-portraits.sh <portraits-dir> <dist-dir>
#   ./scripts/portraits/pack-portraits.sh ~/work/penny-orc/pennyfarthing/pennyfarthing-dist/personas/portraits dist/portraits

set -euo pipefail

PORTRAITS_DIR="${1:?Usage: pack-portraits.sh <portraits-dir> <dist-dir>}"
DIST_DIR="${2:?Usage: pack-portraits.sh <portraits-dir> <dist-dir>}"

if [[ ! -d "$PORTRAITS_DIR" ]]; then
    echo "Error: portraits directory not found: $PORTRAITS_DIR"
    exit 1
fi

mkdir -p "$DIST_DIR"

count=0
for theme_dir in "$PORTRAITS_DIR"/*/; do
    theme=$(basename "$theme_dir")
    # Skip legacy flat-layout size dirs at top level
    [[ "$theme" == "small" || "$theme" == "medium" || "$theme" == "large" || "$theme" == "original" ]] && continue
    # Must have at least original/ with images
    [[ ! -d "$theme_dir/original" ]] && continue

    echo "Packing $theme..."
    COPYFILE_DISABLE=1 tar czf "$DIST_DIR/${theme}.tar.gz" -C "$theme_dir" .
    openssl dgst -sha256 -r "$DIST_DIR/${theme}.tar.gz" | cut -d' ' -f1 > "$DIST_DIR/${theme}.sha256"
    count=$((count + 1))
done

echo "Packed $count themes to $DIST_DIR"
