#!/bin/bash
set -e

# Compare dashboard SVG output between two git tags to detect regressions
#
# Usage:
#   ./scripts/compare-versions.sh              # Compare last 2 tags
#   ./scripts/compare-versions.sh v0.8.11 v0.8.12  # Compare specific tags

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPARISON_DIR="$PROJECT_DIR/comparison_output"

# Test configurations to run
declare -a TEST_CONFIGS=(
    "Australia/Melbourne|bom|-37.7410|144.7012|melbourne_bom"
    "Australia/Melbourne|open_meteo|-37.7410|144.7012|melbourne_openmeteo"
    "America/New_York|open_meteo|40.7128|-74.0060|ny_openmeteo"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Get last N tags
get_last_tags() {
    local count=$1
    git tag --sort=-version:refname | head -n "$count"
}

# Run dashboard generation for a specific configuration
run_dashboard() {
    local tz=$1
    local provider=$2
    local lat=$3
    local lon=$4
    local output_name=$5

    log_info "Running: TZ=$tz PROVIDER=$provider LAT=$lat LON=$lon"

    # Run with specific config
    TZ="$tz" \
    APP_API__PROVIDER="$provider" \
    APP_API__LATITUDE="$lat" \
    APP_API__LONGITUDE="$lon" \
    APP_DEBUGGING__DISABLE_PNG_OUTPUT=true \
    cargo run --quiet 2>&1 | grep -v "Compiling\|Finished\|Running"

    # Copy the generated SVG to comparison directory with versioned name
    if [ -f "dashboard.svg" ]; then
        cp dashboard.svg "$output_name"
        log_success "Generated: $output_name"
        return 0
    else
        log_error "Failed to generate dashboard.svg"
        return 1
    fi
}

# Compare two SVG files
compare_svgs() {
    local file1=$1
    local file2=$2
    local label=$3

    if [ ! -f "$file1" ] || [ ! -f "$file2" ]; then
        log_warning "Missing files for comparison: $label"
        return 1
    fi

    # Use diff to compare (ignoring whitespace changes)
    if diff -q "$file1" "$file2" > /dev/null 2>&1; then
        log_success "✓ No changes detected: $label"
        return 0
    else
        log_warning "✗ CHANGES DETECTED: $label"
        echo "----------------------------------------"
        echo "Diff for: $label"
        echo "----------------------------------------"
        diff -u "$file1" "$file2" || true
        echo "----------------------------------------"
        return 1
    fi
}

# Main script
main() {
    cd "$PROJECT_DIR"

    # Determine which tags to compare
    if [ $# -eq 2 ]; then
        TAG1=$1
        TAG2=$2
        log_info "Comparing specified tags: $TAG1 vs $TAG2"
    else
        # Get last 2 tags automatically
        TAGS=($(get_last_tags 2))
        if [ ${#TAGS[@]} -lt 2 ]; then
            log_error "Need at least 2 git tags to compare. Found: ${#TAGS[@]}"
            exit 1
        fi
        TAG2=${TAGS[0]}  # Most recent
        TAG1=${TAGS[1]}  # Previous
        log_info "Auto-detected tags: $TAG1 (old) vs $TAG2 (new)"
    fi

    # Verify tags exist
    if ! git rev-parse "$TAG1" > /dev/null 2>&1; then
        log_error "Tag not found: $TAG1"
        exit 1
    fi
    if ! git rev-parse "$TAG2" > /dev/null 2>&1; then
        log_error "Tag not found: $TAG2"
        exit 1
    fi

    # Save current branch/commit
    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
    CURRENT_COMMIT=$(git rev-parse HEAD)

    # Create comparison directory
    rm -rf "$COMPARISON_DIR"
    mkdir -p "$COMPARISON_DIR"

    log_info "Starting regression comparison between $TAG1 and $TAG2"
    echo ""

    # Process each tag
    for TAG in "$TAG1" "$TAG2"; do
        log_info "========================================"
        log_info "Processing version: $TAG"
        log_info "========================================"

        # Checkout tag
        git checkout "$TAG" 2>&1 | grep -v "^Note:"

        # Build the project
        log_info "Building $TAG..."
        cargo build --release --quiet

        # Run each test configuration
        for config in "${TEST_CONFIGS[@]}"; do
            IFS='|' read -r tz provider lat lon name <<< "$config"

            output_file="$COMPARISON_DIR/${TAG}_${name}.svg"

            if run_dashboard "$tz" "$provider" "$lat" "$lon" "$output_file"; then
                echo ""
            else
                log_error "Failed to generate dashboard for $name in $TAG"
            fi
        done

        echo ""
    done

    # Restore original branch/commit
    log_info "Restoring original state: $CURRENT_BRANCH"
    git checkout "$CURRENT_BRANCH" 2>&1 | grep -v "^Note:"

    # Compare outputs
    log_info "========================================"
    log_info "COMPARISON RESULTS"
    log_info "========================================"
    echo ""

    CHANGES_DETECTED=0
    for config in "${TEST_CONFIGS[@]}"; do
        IFS='|' read -r tz provider lat lon name <<< "$config"

        file1="$COMPARISON_DIR/${TAG1}_${name}.svg"
        file2="$COMPARISON_DIR/${TAG2}_${name}.svg"

        if ! compare_svgs "$file1" "$file2" "$name"; then
            CHANGES_DETECTED=1
        fi
        echo ""
    done

    # Summary
    echo ""
    log_info "========================================"
    log_info "SUMMARY"
    log_info "========================================"
    echo "Compared: $TAG1 → $TAG2"
    echo "Output directory: $COMPARISON_DIR"
    echo ""

    if [ $CHANGES_DETECTED -eq 0 ]; then
        log_success "No regressions detected! All outputs match."
        exit 0
    else
        log_warning "REGRESSIONS DETECTED! Review changes above."
        log_info "SVG files saved in: $COMPARISON_DIR"
        exit 1
    fi
}

# Run main function
main "$@"
