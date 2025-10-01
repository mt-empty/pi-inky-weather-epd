# High-Quality GIF Generation Commands

This document records the exact commands used to generate the high-quality timelapse GIF with optimal Bayer dithering.

## Final Working Commands

The following commands produced the best quality GIF (`timelapse.gif`) with Bayer dithering:

### Step 1: Save the generated PNG files to `/workspaces/inky-weather-display/arch_png/`

Use a cron job to save the dashboard.png in that directory with filenames like `dashboard_YYYYMMDD_HH.png` for proper chronological ordering.

Then use this script to convert SVGs to PNGs:
```bash
for file in arch_svg/*.svg; do
    filename=$(basename "$file" .svg)
    cp "$file" ./dashboard.svg
    cargo run
    mv dashboard.png "arch_png/${filename}.png"
done
```

### Step 2: Generate Optimized Palette
```bash
cd /workspaces/inky-weather-display/arch_png
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
