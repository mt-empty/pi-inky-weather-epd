#!/bin/bash
# Simulate 24 hours of weather dashboard generation
# This script generates dashboards for each hour of a day using cached data

set -euo pipefail

# Time and date components needs to be in UTC
# https://www.worldtimebuddy.com/united-states-new-york-new-york-to-utc

# Get current UTC date and hour as defaults
DEFAULT_UTC_DATE=$(date -u +"%Y-%m-%d")
DEFAULT_UTC_HOUR=$(date -u +"%H")

# Configuration
OUTPUT_DIR="simulation_output"
START_DATE="${1:-$DEFAULT_UTC_DATE}"  # Default to current UTC date, can be overridden
START_HOUR="${2:-$DEFAULT_UTC_HOUR}"  # Default to current UTC hour (0-23)
TIMEZONE="${3:-$TZ}"              # Optional timezone (e.g., "America/New_York", "Australia/Melbourne")

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=====================================================${NC}"
echo -e "${BLUE}   24-Hour Weather Dashboard Simulator${NC}"
echo -e "${BLUE}=====================================================${NC}"
echo ""

# Delete old simulation files
if [ -d "$OUTPUT_DIR" ]; then
    echo -e "${BLUE}Cleaning up old simulation files...${NC}"
    rm -rf "$OUTPUT_DIR"
    echo -e "${GREEN}[OK] Removed old output directory${NC}"
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"
echo -e "${GREEN}[OK] Created output directory: $OUTPUT_DIR${NC}"
echo ""

# Build the project if needed
echo -e "${BLUE}Building project with CLI support...${NC}"
if cargo build --features cli 2>&1 | grep -q "Finished"; then
    echo -e "${GREEN}[OK] Build successful${NC}"
else
    echo -e "${YELLOW}Warning: Build had warnings (continuing)${NC}"
fi
echo ""

# Fetch fresh weather data before simulation
echo -e "${BLUE}Fetching fresh weather data...${NC}"
if TZ=$TIMEZONE APP_DEBUGGING__DISABLE_WEATHER_API_REQUESTS=false ./target/debug/pi-inky-weather-epd > /dev/null 2>&1; then
    echo -e "${GREEN}[OK] Weather data cached successfully${NC}"
else
    echo -e "${YELLOW}Warning: Failed to fetch data, will use existing cache if available${NC}"
fi
echo ""

# Set timezone if provided
if [ -n "$TIMEZONE" ]; then
    export TZ="$TIMEZONE"
    echo -e "${BLUE}Timezone set to: $TIMEZONE${NC}"
fi

# Generate dashboards for each hour
echo -e "${BLUE}Generating 24 hourly dashboards starting from ${START_DATE}T${START_HOUR}:00:00Z${NC}"
echo ""

for hour in $(seq "$START_HOUR" $((START_HOUR + 23))); do
    # Calculate actual hour (wrap around if needed)
    actual_hour=$((hour % 24))

    # Calculate day offset for hours >= 24
    day_offset=$((hour / 24))

    # Format hour with leading zero
    hour_formatted=$(printf "%02d" "$actual_hour")

    # Calculate the date (add day offset if needed)
    if [ "$day_offset" -gt 0 ]; then
        current_date=$(date -d "$START_DATE + $day_offset days" +%Y-%m-%d)
    else
        current_date="$START_DATE"
    fi

    # Create timestamp in RFC3339 format (UTC)
    timestamp="${current_date}T${hour_formatted}:00:00Z"

    # Create output filename
    output_file="${OUTPUT_DIR}/dashboard_${current_date}_${hour_formatted}00.svg"

    echo -e "${BLUE}  [${hour_formatted}] ${timestamp}${NC}"

    # Run the application with simulated time
    # Redirect stdout to capture only the generated SVG
    if TZ=$TIMEZONE APP_DEBUGGING__DISABLE_WEATHER_API_REQUESTS=true ./target/debug/pi-inky-weather-epd --simulate-time "$timestamp" > /dev/null 2>&1; then
        # Copy the generated dashboard.svg to the timestamped file
        if [ -f "dashboard.svg" ]; then
            cp "dashboard.svg" "$output_file"
            # cp "dashboard.png" "${output_file%.svg}.png"
            # Fix icon paths for correct relative path from simulation_output/ directory
            sed -i 's|"static/|"../static/|g' "$output_file"
            sed -i "s|'static/|'../static/|g" "$output_file"
            echo -e "${GREEN}       [OK] Generated: $output_file${NC}"
        else
            echo -e "${RED}       [FAIL] dashboard.svg not found${NC}"
        fi
    else
        echo -e "${RED}       [FAIL] Failed to generate dashboard${NC}"
    fi
done

echo ""
echo -e "${GREEN}=====================================================${NC}"
echo -e "${GREEN}   Simulation Complete!${NC}"
echo -e "${GREEN}=====================================================${NC}"
echo ""
echo -e "${BLUE}Generated files are in: ${OUTPUT_DIR}/${NC}"
echo -e "${BLUE}Total dashboards: 24${NC}"
echo ""
echo -e "${YELLOW}Tips:${NC}"
echo -e "  - View SVGs: open ${OUTPUT_DIR}/dashboard_*.svg"
echo -e "  - Create GIF: See misc/gif-generation-commands.md"
echo -e "  - Clean up: rm -rf ${OUTPUT_DIR}"
echo ""
