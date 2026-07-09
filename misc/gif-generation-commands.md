# High-Quality GIF Generation Commands

This document records the exact commands used to generate the high-quality timelapse GIF with optimal Bayer dithering.

## Quick Start

`scripts/generate-timelapse-gif.sh` wraps everything below (frame generation
+ palette + Bayer-dithered encode) into one command. It checks for `ffmpeg`
up front and exits with install instructions if it's missing.

```bash
cargo build --features cli

# Uses your configured location:
./scripts/generate-timelapse-gif.sh

# Or target any location for this run only ("lat,lon"):
./scripts/generate-timelapse-gif.sh "-41.244772,-71.015625"

# Full signature: [location] [date] [start_hour] [timezone] [output_gif]
./scripts/generate-timelapse-gif.sh "-41.244772,-71.015625" 2025-12-26 6 "America/Argentina/Buenos_Aires"
```

Output: `simulation_output/timelapse.gif`. Promote it to the tracked GIF with
`mv simulation_output/timelapse.gif misc/timelapse.gif` once you're happy
with it.

The rest of this document explains what the script does and why, and covers
the manual/cron alternative.

## Final Working Commands

The following commands produced the best quality GIF (`timelapse.gif`) with Bayer dithering:

### Step 1: Generate the PNG frames

Two ways to get chronologically-ordered `dashboard_*.png` frames:

**Option A — `simulate` (recommended, fast, no cron needed)**

Fetches real weather data once, then renders every hour of the forecast
window from that single cached fetch — a full 24-hour set of frames in a
few seconds. Also works for any lat/lon, not just your configured location
(see `readme.md`'s "Dashboard Simulation" section for the underlying flags):

```bash
cargo build --features cli

# Optionally target a different location for this run only:
APP_API__PROVIDER=open_meteo \
APP_API__LATITUDE=-41.877741 \
APP_API__LONGITUDE=-70.356445 \
./scripts/simulate-24h.sh "$(date -u +%Y-%m-%d)" "$(date -u +%H)" "America/Argentina/Buenos_Aires"
```

The script renders each frame's PNG itself (`APP_DEV__DISABLE_PNG_OUTPUT=false`)
immediately after each `simulate` call, while `dashboard.svg` still has its
original repo-root-relative icon paths (`static/...`) — that's required for
icons to actually appear in the PNG. **Don't** post-process the SVGs under
`simulation_output/*.svg` with `render-svg` after the fact: those copies have
already had their icon paths rewritten to `../static/...` for browser
viewing only, so converting them produces PNGs with all icons silently
missing.

Frames land in `simulation_output/png/`; use that directory in place of
`arch_png/` in Steps 2-4 below. Note `simulate-24h.sh` can only cover the
real forecast window (today plus the next several days) — it can't
fabricate historical or far-future data.

**Option B — cron job (real-time capture, slow)**

Use a cron job to save `dashboard.png` to
`/workspaces/inky-weather-display/arch_png/` with filenames like
`dashboard_YYYYMMDD_HH.png` for proper chronological ordering, run over
real time (e.g. hourly for a day).

If you're converting pre-saved SVGs instead of PNGs directly, use this
script:
```bash
for file in arch_svg/*.svg; do
    filename=$(basename "$file" .svg)
    cp "$file" ./dashboard.svg
    cargo run
    mv dashboard.png "arch_png/${filename}.png"
done
```

### Step 2: Generate Optimized Palette

`cd` into whichever frame directory Step 1 produced — `simulation_output/png`
for Option A, or `arch_png` for Option B — then:
```bash
ffmpeg -framerate 3 -pattern_type glob -i 'dashboard_*.png' -vf "palettegen=max_colors=256:stats_mode=full" -y palette_corrected.png
```

**Parameters explained:**
- `framerate 3` - Matches final GIF frame rate
- `pattern_type glob` - Allows wildcard pattern for input files
- `palettegen=max_colors=256` - Generate full 256-color palette
- `stats_mode=full` - Analyze all frames for optimal color selection

### Step 3: Create GIF with Bayer Dithering
```bash
ffmpeg -framerate 3 -pattern_type glob -i 'dashboard_*.png' -i palette_corrected.png -lavfi "paletteuse=dither=bayer:bayer_scale=3:diff_mode=rectangle" -y timelapse_bayer.gif
```

**Parameters explained:**
- `framerate 3` - 3 frames per second (optimal for weather data visualization)
- `pattern_type glob -i 'dashboard_*.png'` - Input all PNG files in chronological order
- `-i palette_corrected.png` - Use the optimized palette from step 1
- `paletteuse=dither=bayer` - Use Bayer dithering algorithm
- `bayer_scale=3` - Dithering intensity (5 = optimal quality vs file size)
- `diff_mode=rectangle` - Optimize frame differences for better compression
- `-y` - Overwrite output file without prompting

### Step 4: Replace Current GIF
```bash
mv timelapse.gif timelapse_imagemagick_backup.gif
mv timelapse_bayer.gif timelapse.gif
```

## Results

**File specifications:**
- **Size**: 1.9MB
- **Resolution**: 1600x960 (full resolution)
- **Frame rate**: 3fps
- **Duration**: ~24 seconds (73 frames)
- **Color depth**: 256 colors with optimized palette
- **Dithering**: Bayer pattern with scale 3


## Alternative Commands Tested

### Floyd-Steinberg Dithering (good alternative)
```bash
ffmpeg -framerate 3 -pattern_type glob -i 'dashboard_*.png' -i palette_full.png -lavfi "paletteuse=dither=floyd_steinberg" -y timelapse_floyd.gif
```

### No Dithering (sharp but may have color banding)
```bash
ffmpeg -framerate 3 -pattern_type glob -i 'dashboard_*.png' -i palette.png -lavfi "paletteuse=dither=none" -y timelapse_no_dither.gif
```

### ImageMagick Alternative (larger file size)
```bash
convert -delay 33 -loop 0 dashboard_*.png timelapse_imagemagick.gif
```

## Color Palette Information

The optimized palette is generated from the actual PNG files and includes:
- **Named colors**: black, white, red, blue, green, yellow, orange, purple
- **64 unique hex colors** extracted from weather icon SVGs
- **Total**: 72 unique colors optimized for weather dashboard display

## Notes

- Bayer dithering works particularly well for weather data with gradual color transitions
- The `bayer_scale=3` setting provides the best balance between quality and file size
- Rectangle diff mode optimizes frame-to-frame compression
- Full stats mode palette generation ensures optimal color selection across all frames
- 3fps frame rate is ideal for weather timelapse data (shows changes without being too fast)

## Usage

To recreate this GIF quality with new PNG files:
1. Place PNG files in `/workspaces/inky-weather-display/arch_png/`
2. Ensure files are named with sortable timestamps (e.g., `dashboard_YYYYMMDD_HH.png`)
3. Run the two-step process above
4. The resulting GIF will have optimal quality for weather dashboard visualization
