#!/bin/bash
# Generate a Bayer-dithered timelapse GIF from a 24-hour simulated forecast.
# Wraps simulate-24h.sh (frame generation) + ffmpeg (palette + GIF encode).
# See misc/gif-generation-commands.md for the underlying commands and why
# they're structured this way.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

OUTPUT_DIR="simulation_output"
LOCATION="${1:-}"     # Optional "lat,lon" (e.g. "-41.244772,-71.015625"); empty = use configured location
START_DATE="${2:-}"   # Forwarded to simulate-24h.sh; empty = today (UTC)
START_HOUR="${3:-}"   # Forwarded to simulate-24h.sh; empty = current hour (UTC)
TIMEZONE="${4:-}"     # Forwarded to simulate-24h.sh; empty = system timezone
OUTPUT_GIF="${5:-$OUTPUT_DIR/timelapse.gif}"

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

if [ -n "$LOCATION" ]; then
    LATITUDE="${LOCATION%%,*}"
    LONGITUDE="${LOCATION#*,}"
    if [ "$LATITUDE" = "$LONGITUDE" ] || [ -z "$LATITUDE" ] || [ -z "$LONGITUDE" ]; then
        echo -e "${RED}[FAIL] Invalid location '$LOCATION', expected \"lat,lon\" (e.g. \"-41.244772,-71.015625\")${NC}"
        exit 1
    fi
    # Open-Meteo has global coverage, unlike bom (Australia only), so it's
    # always safe to switch providers when overriding the location.
    export APP_API__PROVIDER=open_meteo
    export APP_API__LATITUDE="$LATITUDE"
    export APP_API__LONGITUDE="$LONGITUDE"
    echo -e "${BLUE}Location override: lat=$LATITUDE lon=$LONGITUDE (provider=open_meteo)${NC}"
    echo ""
fi

echo -e "${BLUE}=====================================================${NC}"
echo -e "${BLUE}   Weather Dashboard Timelapse GIF Generator${NC}"
echo -e "${BLUE}=====================================================${NC}"
echo ""

./scripts/simulate-24h.sh "$START_DATE" "$START_HOUR" "$TIMEZONE"

PNG_DIR="$OUTPUT_DIR/png"
frame_count=$(find "$PNG_DIR" -name 'dashboard_*.png' | wc -l)
if [ "$frame_count" -eq 0 ]; then
    echo -e "${RED}[FAIL] No PNG frames found in $PNG_DIR${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}Generating optimized palette from $frame_count frames...${NC}"
ffmpeg -framerate 3 -pattern_type glob -i "$PNG_DIR/dashboard_*.png" \
    -vf "palettegen=max_colors=256:stats_mode=full" \
    -y "$PNG_DIR/palette.png" -loglevel error
echo -e "${GREEN}[OK] Palette generated${NC}"

echo -e "${BLUE}Encoding GIF with Bayer dithering...${NC}"
mkdir -p "$(dirname "$OUTPUT_GIF")"
ffmpeg -framerate 3 -pattern_type glob -i "$PNG_DIR/dashboard_*.png" \
    -i "$PNG_DIR/palette.png" \
    -lavfi "paletteuse=dither=bayer:bayer_scale=3:diff_mode=rectangle" \
    -y "$OUTPUT_GIF" -loglevel error
echo -e "${GREEN}[OK] GIF saved: $OUTPUT_GIF${NC}"

echo ""
echo -e "${GREEN}=====================================================${NC}"
echo -e "${GREEN}   Done!${NC}"
echo -e "${GREEN}=====================================================${NC}"
echo ""
echo -e "${YELLOW}Tips:${NC}"
echo -e "  - Preview: open $OUTPUT_GIF"
echo -e "  - Replace the tracked GIF: mv $OUTPUT_GIF misc/timelapse.gif"
echo -e "  - Clean up frames: rm -rf $OUTPUT_DIR"
echo ""
