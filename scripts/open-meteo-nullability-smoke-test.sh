#!/bin/bash
set -uo pipefail

# Ad hoc smoke test: hits the live Open-Meteo API with the exact same query
# parameters this app uses (see src/constants.rs) for one representative
# coordinate per national weather service Open-Meteo's "Best Match" draws
# from, and reports which response fields come back containing a `null`
# somewhere in their array.
#
# Why: Open-Meteo doesn't document field-level nullability, and model
# selection/coverage/horizon can change over time, so re-run this whenever
# you suspect the API's shape has drifted, or periodically as a manual
# check. Not wired into CI - it hits the network and depends on live,
# time-varying forecast data.
#
# Usage:
#   ./scripts/open-meteo-nullability-smoke-test.sh              # all locations
#   ./scripts/open-meteo-nullability-smoke-test.sh london berlin # just these
#
# Requires: curl, python3

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_URL="${OPEN_METEO_BASE_URL:-https://api.open-meteo.com}"

# name | lat | lon | provider whose coverage this is meant to exercise
LOCATIONS=(
    "reykjavik|64.1466|-21.9426|ECMWF (no regional model nearby)"
    "new_york|40.7128|-74.0060|NOAA/NCEP"
    "tokyo|35.6762|139.6503|JMA"
    "seoul|37.5665|126.9780|KMA"
    "berlin|52.5200|13.4050|DWD"
    "toronto|43.6532|-79.3832|GEM/ECCC"
    "paris|48.8566|2.3522|Meteo-France"
    "beijing|39.9042|116.4074|CMA"
    "melbourne|-37.8136|144.9631|BOM"
    "rome|41.9028|12.4964|ItaliaMeteo ARPAE"
    "oslo|59.9139|10.7522|MET Norway"
    "amsterdam|52.3676|4.9041|KNMI"
    "copenhagen|55.6761|12.5683|DMI"
    "london|51.5085|-0.1257|UK Met Office"
    "zurich|47.3769|8.5417|MeteoSwiss"
    "vienna|48.2082|16.3738|GeoSphere Austria"
    "prague|50.0755|14.4378|CHMI"
)

# Mirror src/constants.rs exactly - if those params drift, update here too.
HOURLY_VARS="temperature_2m,apparent_temperature,precipitation_probability,precipitation,uv_index,wind_speed_10m,wind_gusts_10m,relative_humidity_2m,snowfall,cloud_cover,weather_code,is_day"
DAILY_VARS="sunrise,sunset,temperature_2m_max,temperature_2m_min,precipitation_sum,precipitation_probability_max,snowfall_sum,cloud_cover_mean,weather_code"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'

check_response() {
    local label="$1" json_file="$2" array_key="$3"
    python3 - "$json_file" "$array_key" <<'PY'
import json, sys

path, array_key = sys.argv[1], sys.argv[2]
with open(path) as f:
    data = json.load(f)

if "error" in data and data.get("error") is True:
    print(f"    API_ERROR: {data.get('reason', 'unknown')}")
    sys.exit(2)

fields = data.get(array_key, {})
found_null = False
for name, values in fields.items():
    if not isinstance(values, list):
        continue
    null_count = sum(1 for v in values if v is None)
    if null_count:
        found_null = True
        print(f"    NULL: {name} has {null_count}/{len(values)} null entries")

sys.exit(1 if found_null else 0)
PY
}

run_one() {
    local name="$1" lat="$2" lon="$3" provider="$4"
    echo -e "${BLUE}== ${name} (${lat}, ${lon}) - ${provider} ==${NC}"

    local hourly_url="${BASE_URL}/v1/forecast?latitude=${lat}&longitude=${lon}&hourly=${HOURLY_VARS}&current=is_day&forecast_days=3&timezone=UTC"
    local daily_url="${BASE_URL}/v1/forecast?latitude=${lat}&longitude=${lon}&daily=${DAILY_VARS}&current=is_day&forecast_days=8&past_days=1&timezone=auto"

    local hourly_json="/tmp/open-meteo-smoke-${name}-hourly.json"
    local daily_json="/tmp/open-meteo-smoke-${name}-daily.json"

    local hourly_code daily_code
    hourly_code=$(curl -s -o "$hourly_json" -w '%{http_code}' "$hourly_url")
    daily_code=$(curl -s -o "$daily_json" -w '%{http_code}' "$daily_url")

    local status="${GREEN}OK${NC}"

    if [[ "$hourly_code" != "200" ]]; then
        echo -e "    ${RED}HOURLY HTTP $hourly_code${NC}"
        status="${RED}FAIL${NC}"
    else
        echo "  hourly:"
        if ! check_response "$name" "$hourly_json" "hourly"; then
            rc=$?
            [[ $rc -eq 2 ]] && status="${RED}FAIL${NC}" || status="${YELLOW}NULLS FOUND${NC}"
        fi
    fi

    if [[ "$daily_code" != "200" ]]; then
        echo -e "    ${RED}DAILY HTTP $daily_code${NC}"
        status="${RED}FAIL${NC}"
    else
        echo "  daily:"
        if ! check_response "$name" "$daily_json" "daily"; then
            rc=$?
            [[ $rc -eq 2 ]] && status="${RED}FAIL${NC}" || status="${YELLOW}NULLS FOUND${NC}"
        fi
    fi

    echo -e "  status: ${status}"
    echo
}

filters=("$@")

for entry in "${LOCATIONS[@]}"; do
    IFS='|' read -r name lat lon provider <<< "$entry"
    if [[ ${#filters[@]} -gt 0 ]]; then
        match=0
        for f in "${filters[@]}"; do
            [[ "$name" == "$f" ]] && match=1
        done
        [[ $match -eq 0 ]] && continue
    fi
    run_one "$name" "$lat" "$lon" "$provider"
done

echo "Done. Raw responses saved under /tmp/open-meteo-smoke-*.json for inspection."
