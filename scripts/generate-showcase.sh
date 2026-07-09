#!/bin/bash
# Regenerate the README showcase assets (misc/timelapse.gif) from frozen,
# checked-in fixture data — fully offline and deterministic. Re-run this
# any time rendering code changes so the showcase stays in sync, without
# depending on live API data that can't be reproduced later.
#
# Fixture data lives in misc/showcase_fixtures/<scenario>/ and was built by
# splicing real API captures from tests/fixtures/ (see git history for the
# one-off splicing script). To add a new scenario, drop a fixture pair
# there (open_meteo_hourly_forecast.json + open_meteo_daily_forecast.json)
# with a fixed, contiguous time window.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# The showcase location/timezone must stay in sync with whatever lat/lon
# the fixture data was written to represent — sunrise/sunset and day/night
# icon selection are computed astronomically from these, not from the
# fixture JSON.
SHOWCASE_LAT="-37.8136"
SHOWCASE_LON="144.9631"
SHOWCASE_TZ="Australia/Melbourne"

# Fixed window baked into misc/showcase_fixtures/gif/ — must match the
# fixture's own hourly.time values (see find_forecast_window in
# src/dashboard/context.rs: the sim timestamp's UTC date must have a
# matching entry in the fixture, and be near the start of its window).
# The narrative starts at local 9am (Australia/Melbourne, AEDT = UTC+11),
# which is 2025-10-24T22:00 UTC — hence the previous-day date/hour here.
GIF_DATE="2025-10-24"
GIF_START_HOUR="22"

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

echo -e "${BLUE}=====================================================${NC}"
echo -e "${BLUE}   README Showcase Generator (offline, deterministic)${NC}"
echo -e "${BLUE}=====================================================${NC}"
echo ""

export APP_API__PROVIDER=open_meteo
export APP_API__LATITUDE="$SHOWCASE_LAT"
export APP_API__LONGITUDE="$SHOWCASE_LON"
export FIXTURE_DIR="misc/showcase_fixtures/gif"

./scripts/simulate-24h.sh "$GIF_DATE" "$GIF_START_HOUR" "$SHOWCASE_TZ"

PNG_DIR="simulation_output/png"
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
ffmpeg -framerate 3 -pattern_type glob -i "$PNG_DIR/dashboard_*.png" \
    -i "$PNG_DIR/palette.png" \
    -lavfi "paletteuse=dither=bayer:bayer_scale=3:diff_mode=rectangle" \
    -y "misc/timelapse.gif" -loglevel error
echo -e "${GREEN}[OK] GIF saved: misc/timelapse.gif${NC}"

echo ""
echo -e "${GREEN}=====================================================${NC}"
echo -e "${GREEN}   Done!${NC}"
echo -e "${GREEN}=====================================================${NC}"
echo ""
echo -e "${YELLOW}Tips:${NC}"
echo -e "  - Preview: open misc/timelapse.gif"
echo -e "  - Clean up frames: rm -rf simulation_output"
echo -e "  - Diff against the previous version before committing: git diff --stat misc/timelapse.gif"
echo ""
