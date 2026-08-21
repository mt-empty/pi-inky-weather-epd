#!/bin/bash
# Generate a GIF cycling through the per-language dashboard screenshots in
# misc/languages/ (produced by generate-showcase.sh's language loop).
# Wraps ffmpeg's concat demuxer (fixed order + per-frame duration) + palette
# generation, following the same palette/paletteuse approach as
# generate-timelapse-gif.sh. See misc/gif-generation-commands.md.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

LANG_DIR="misc/languages"
OUTPUT_GIF="${1:-$LANG_DIR/languages.gif}"
FRAME_SECONDS="${2:-0.5}"

# Fixed display order (matches the readme's Language section), not
# alphabetical filename order.
LANGUAGES=(en fr de es ja)

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

if ! command -v ffmpeg >/dev/null 2>&1; then
    echo -e "${RED}[FAIL] ffmpeg is not installed.${NC}"
    echo -e "${YELLOW}Install it, e.g.:${NC}"
    echo "  Debian/Ubuntu: sudo apt-get install ffmpeg"
    echo "  macOS:         brew install ffmpeg"
    exit 1
fi

for lang in "${LANGUAGES[@]}"; do
    if [ ! -f "$LANG_DIR/dashboard-${lang}.png" ]; then
        echo -e "${RED}[FAIL] Missing $LANG_DIR/dashboard-${lang}.png${NC}"
        echo -e "${YELLOW}Run ./scripts/generate-showcase.sh first.${NC}"
        exit 1
    fi
done

echo -e "${BLUE}=====================================================${NC}"
echo -e "${BLUE}   Language Showcase GIF Generator${NC}"
echo -e "${BLUE}=====================================================${NC}"
echo ""

CONCAT_LIST="$(mktemp)"
trap 'rm -f "$CONCAT_LIST"' EXIT

for lang in "${LANGUAGES[@]}"; do
    echo "file '$PWD/$LANG_DIR/dashboard-${lang}.png'" >> "$CONCAT_LIST"
    echo "duration $FRAME_SECONDS" >> "$CONCAT_LIST"
done
# ffmpeg's concat demuxer ignores the last "duration" line unless the final
# file is repeated, so re-list the last frame without a duration.
echo "file '$PWD/$LANG_DIR/dashboard-${LANGUAGES[-1]}.png'" >> "$CONCAT_LIST"

echo -e "${BLUE}Generating optimized palette...${NC}"
ffmpeg -f concat -safe 0 -i "$CONCAT_LIST" \
    -vf "palettegen=max_colors=256:stats_mode=full" \
    -y "$LANG_DIR/palette.png" -loglevel error
echo -e "${GREEN}[OK] Palette generated${NC}"

echo -e "${BLUE}Encoding GIF...${NC}"
mkdir -p "$(dirname "$OUTPUT_GIF")"
ffmpeg -f concat -safe 0 -i "$CONCAT_LIST" \
    -i "$LANG_DIR/palette.png" \
    -lavfi "paletteuse=dither=bayer:bayer_scale=3:diff_mode=rectangle" \
    -y "$OUTPUT_GIF" -loglevel error
rm -f "$LANG_DIR/palette.png"
echo -e "${GREEN}[OK] GIF saved: $OUTPUT_GIF${NC}"

echo ""
echo -e "${GREEN}=====================================================${NC}"
echo -e "${GREEN}   Done!${NC}"
echo -e "${GREEN}=====================================================${NC}"
echo ""
echo -e "${YELLOW}Tips:${NC}"
echo -e "  - Preview: open $OUTPUT_GIF"
echo ""
