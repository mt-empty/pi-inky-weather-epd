#!/usr/bin/env bash
# generate-icon-preview.sh
# Generates SVG preview pages for all icons in static/fill-svg-static/
# Layout: 9 columns × 4 rows = 36 icons per page, 800×480 canvas
# Usage: ./scripts/generate-icon-preview.sh [--output-dir DIR]
#
# Icons prefixed with "not-used" are excluded.
# Output: static/icon-preview-1.svg, static/icon-preview-2.svg, ...

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ICON_DIR="$REPO_ROOT/static/fill-svg-static"
OUTPUT_DIR="$REPO_ROOT"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --output-dir) OUTPUT_DIR="$2"; shift 2 ;;
        *) echo "Unknown arg: $1"; exit 1 ;;
    esac
done

# ── Layout constants ───────────────────────────────────────────────────────
COLS=9
ROWS=4
ICONS_PER_PAGE=$(( COLS * ROWS ))          # 36
CANVAS_W=800
CANVAS_H=480
TITLE_H=5
COL_W=$(( CANVAS_W / COLS ))               # 88px
ROW_H=$(( (CANVAS_H - TITLE_H) / ROWS ))   # 115px
ICON_W=75
ICON_H=100
ICON_X=$(( (COL_W - ICON_W) / 2 ))        # 6 — centres icon horizontally in column
ICON_Y=5
LABEL_Y1=$(( ICON_Y + ICON_H ))       # 113
LABEL_Y2=$(( LABEL_Y1 + 10 ))             # 123
COL_MID=$(( COL_W / 2 ))                  # 44

# ── Collect icons ──────────────────────────────────────────────────────────
mapfile -t ICONS < <(
    find "$ICON_DIR" -maxdepth 1 -name "*.svg" \
        ! -name "not-used*" \
        ! -name "icon-preview*.svg" \
        -printf "%f\n" | sort
)

TOTAL=${#ICONS[@]}
TOTAL_PAGES=$(( (TOTAL + ICONS_PER_PAGE - 1) / ICONS_PER_PAGE ))
echo "Found $TOTAL icons → $TOTAL_PAGES page(s) of up to $ICONS_PER_PAGE"

# ── Split filename into ≤2 label lines at the hyphen nearest the midpoint ──
label_lines() {
    local name="${1%.svg}"
    local len=${#name}
    if (( len <= 16 )); then
        printf '%s\n\n' "$name"
        return
    fi
    local mid=$(( len / 2 )) best=-1 dist=9999
    for (( i=0; i<len; i++ )); do
        if [[ "${name:$i:1}" == "-" ]]; then
            local d=$(( i > mid ? i - mid : mid - i ))
            if (( d < dist )); then dist=$d; best=$i; fi
        fi
    done
    if (( best < 0 )); then
        printf '%s\n\n' "$name"
    else
        printf '%s\n%s\n' "${name:0:$best}" "${name:$(( best + 1 ))}"
    fi
}

# ── Generate pages ─────────────────────────────────────────────────────────
for (( page=0; page<TOTAL_PAGES; page++ )); do
    PAGE_NUM=$(( page + 1 ))
    OUT="$OUTPUT_DIR/icon-preview-${PAGE_NUM}.svg"
    START=$(( page * ICONS_PER_PAGE ))
    END=$(( START + ICONS_PER_PAGE ))
    (( END > TOTAL )) && END=$TOTAL

    {
        cat <<EOF
<svg width="$CANVAS_W" height="$CANVAS_H" font-family="Roboto, sans-serif" xmlns="http://www.w3.org/2000/svg">
    <rect width="100%" height="100%" fill="white" />
    <text x="$(( CANVAS_W / 2 ))" y="15" text-anchor="middle" font-size="12" fill="#555">E-ink Icon Preview — Page ${PAGE_NUM}/${TOTAL_PAGES} · icons $(( START + 1 ))–${END} of ${TOTAL}</text>
EOF

        for (( i=START; i<END; i++ )); do
            li=$(( i - START ))
            col=$(( li % COLS ))
            row=$(( li / COLS ))
            sx=$(( col * COL_W ))
            sy=$(( TITLE_H + row * ROW_H ))
            name="${ICONS[$i]}"

            mapfile -t llines < <(label_lines "$name")
            l1="${llines[0]}"
            l2="${llines[1]:-}"

            printf '    <!-- %s -->\n' "$name"
            printf '    <svg x="%d" y="%d" overflow="visible">\n' "$sx" "$sy"
            printf '        <image x="%d" y="%d" width="%d" height="%d" href="static/fill-svg-static/%s" />\n' \
                "$ICON_X" "$ICON_Y" "$ICON_W" "$ICON_H" "$name"
            if [[ -n "$l2" ]]; then
                printf '        <text x="%d" y="%d" text-anchor="middle" font-size="9" fill="black">%s</text>\n' \
                    "$COL_MID" "$LABEL_Y1" "$l1"
                printf '        <text x="%d" y="%d" text-anchor="middle" font-size="9" fill="#555">%s</text>\n' \
                    "$COL_MID" "$LABEL_Y2" "$l2"
            else
                printf '        <text x="%d" y="%d" text-anchor="middle" font-size="9" fill="black">%s</text>\n' \
                    "$COL_MID" "$LABEL_Y1" "$l1"
            fi
            printf '    </svg>\n'
        done

        # Grid lines
        for (( r=1; r<ROWS; r++ )); do
            gy=$(( TITLE_H + r * ROW_H + 10 ))
            printf '    <line x1="0" y1="%d" x2="%d" y2="%d" stroke="#e0e0e0" stroke-width="1" />\n' \
                "$gy" "$CANVAS_W" "$gy"
        done
        for (( c=1; c<COLS; c++ )); do
            gy=$(( TITLE_H + 20 ))
            gx=$(( c * COL_W ))
            printf '    <line x1="%d" y1="%d" x2="%d" y2="%d" stroke="#e0e0e0" stroke-width="1" />\n' \
                "$gx" "$gy" "$gx" "$CANVAS_H"
        done

        cat <<'EOF'
    <style>
        @font-face { font-family: 'Roboto'; src: url('static/fonts/Roboto-Regular.ttf') format('truetype'); }
    </style>
</svg>
EOF
    } > "$OUT"

    echo "  ✓ $OUT  ($(( END - START )) icons)"
done

echo "Done. Open static/icon-preview-*.svg in the SVG preview extension."
