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
# simulate-24h.sh exports this itself too, but only inside its own child
# process — the static-screenshot renders below call the binary directly
# from this script, so it needs its own export of the same value.
export APP_MISC__TIMEZONE="$SHOWCASE_TZ"
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
echo -e "${BLUE}=====================================================${NC}"
echo -e "${BLUE}   Config-Example Screenshots${NC}"
echo -e "${BLUE}=====================================================${NC}"
echo ""

CACHE_DIR="${APP_MISC__WEATHER_DATA_CACHE_PATH:-./cached_data/}"

# Renders one simulate frame from a given fixture and copies it to
# misc/<output_name>, with any additional APP_* env overrides applied only
# for that render (e.g. a render_options/colours toggle being demonstrated).
render_example() {
    local fixture_dir="$1"
    local timestamp="$2"
    local output_name="$3"
    shift 3

    mkdir -p "$CACHE_DIR"
    cp "${fixture_dir}"/*.json "$CACHE_DIR"

    if env "$@" APP_DEV__DISABLE_WEATHER_API_REQUESTS=true APP_DEV__DISABLE_PNG_OUTPUT=false \
        ./target/debug/pi-inky-weather-epd simulate "$timestamp" > /dev/null 2>&1; then
        mv dashboard.png "misc/${output_name}"
        echo -e "${GREEN}[OK] Generated: misc/${output_name}${NC}"
    else
        echo -e "${RED}[FAIL] Failed to generate misc/${output_name}${NC}"
        exit 1
    fi
}

# Default configuration — the gif fixture's opening hour (9am, clear),
# whose forward-looking 24h graph shows the full day/rain/snow narrative.
render_example "misc/showcase_fixtures/gif" "2025-10-24T22:00:00Z" "dashboard-default.png"

# Moon-phase toggle demo — a clear night hour, explicitly disabling
# use_moon_phase_instead_of_clear_night so the plain clear-night icon
# renders instead of a moon-phase icon (matching the filename).
render_example "misc/showcase_fixtures/moon-phase" "2025-10-25T11:00:00Z" "dashboard-without-moon-phase.png" \
    APP_RENDER_OPTIONS__USE_MOON_PHASE_INSTEAD_OF_CLEAR_NIGHT=false

# X-axis-at-zero toggle demo — the blizzard hour (-5C), so the fixed
# zero-line visibly separates from the chart's bottom edge.
render_example "misc/showcase_fixtures/gif" "2025-10-25T16:00:00Z" "dashboard-x-axis-at-zero.png" \
    APP_RENDER_OPTIONS__X_AXIS_ALWAYS_AT_MIN=false

# Dark theme demo — same hour as the default shot, colours overridden.
render_example "misc/showcase_fixtures/gif" "2025-10-24T22:00:00Z" "dashboard-dark.png" \
    APP_COLOURS__BACKGROUND_COLOUR=black \
    APP_COLOURS__TEXT_COLOUR=white \
    APP_COLOURS__X_AXIS_COLOUR=white \
    APP_COLOURS__Y_LEFT_AXIS_COLOUR=red \
    APP_COLOURS__Y_RIGHT_AXIS_COLOUR=blue \
    APP_COLOURS__ACTUAL_TEMP_COLOUR=red \
    APP_COLOURS__FEELS_LIKE_COLOUR=green \
    APP_COLOURS__RAIN_COLOUR=blue

echo ""
echo -e "${GREEN}=====================================================${NC}"
echo -e "${GREEN}   Done!${NC}"
echo -e "${GREEN}=====================================================${NC}"
echo ""
echo -e "${YELLOW}Tips:${NC}"
echo -e "  - Preview: open misc/timelapse.gif"
echo -e "  - Clean up frames: rm -rf simulation_output"
echo -e "  - Diff against the previous version before committing: git diff --stat misc/timelapse.gif misc/dashboard-*.png"
echo ""
