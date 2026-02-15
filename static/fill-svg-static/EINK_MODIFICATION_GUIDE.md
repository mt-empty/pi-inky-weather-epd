# E-Ink Display SVG Icon Modification Guide

## Target Hardware

**Inky Impression 7.3" e-paper display** — supports exactly 7 colors:
- **Black**, **White**, **Red**, **Green**, **Yellow**, **Orange**, **Blue**

No gradients, no transparency, no arbitrary RGB values. All icons must use only these colors (plus greyscale which the display can approximate via dithering).

## Problem

The original icons (from [Bas Milius / Weather Icons](https://github.com/basmilius/weather-icons)) use:
- `<linearGradient>` definitions with subtle color transitions
- Near-invisible same-tone strokes (e.g., `stroke="#e6effc"` on white clouds)
- Non-palette RGB colors (ambers, teals, blue-grays)

These render poorly on e-ink where gradients become dithered blobs and low-contrast strokes vanish.

## Established Color Palette

### Sun (Day variants)
| Element | Fill | Stroke | Stroke-width |
|---------|------|--------|-------------|
| Sun center (large, standalone) | `yellow` | `black` | `8` |
| Sun center (small, behind cloud) | `yellow` | `black` | `4` |
| Sun rays (large, standalone) | `none` | `yellow` | `24` |
| Sun rays (small, behind cloud) | `none` | `yellow` | `12` |

### Moon (Night variants)
| Element | Fill | Stroke | Stroke-width |
|---------|------|--------|-------------|
| Moon (large, standalone) | `blue` | `black` | `6` |
| Moon (small, behind cloud) | `blue` | `black` | `4` |

### Clouds
| Element | Fill | Stroke | Stroke-width |
|---------|------|--------|-------------|
| Cloud (partly-cloudy, main) | `white` | `black` | `10` |
| Cloud (partly-cloudy, back) | `white` | `black` | `12` |
| Cloud (overcast, smaller) | `white` | `black` | `6` |
| Cloud (overcast, larger) | `white` | `black` | `7` |
| Cloud (extreme variant) | `url(#cloudGradient)` | `black` | `6` |

**Important: ViewBox padding for 2-cloud compositions**

When using two clouds together (small cloud behind big cloud), the viewBox must include padding to prevent clipping:
- **Small cloud symbol**: `viewBox="0 0 210.3 126.1"` (usage: `width="200.3" height="126.1"`)
- **Big cloud symbol**: `viewBox="0 0 350 223"` (usage: `width="350" height="222"`)

This 10px width padding on small clouds and 1px height padding on big clouds ensures the stroke borders render completely without being clipped at the edges.

**Extreme cloud gradient** (only gradient allowed — greyscale dithers acceptably on e-ink):
```xml
<linearGradient id="cloudGradient" x1="0" y1="0" x2="0" y2="1" gradientUnits="objectBoundingBox">
  <stop offset="0" stop-color="white"/>
  <stop offset=".9" stop-color="#6b7280"/>
  <stop offset="1" stop-color="#4b5563"/>
</linearGradient>
```

### Precipitation
| Element | Fill | Stroke | Stroke-width |
|---------|------|--------|-------------|
| Rain drops | `blue` | *(none — remove)* | — |
| Snowflakes | *(none)* | `blue` | `1.8` |
| Ice crystal / sleet gears | `white` | `blue` | *(default)* |

### Lightning
| Element | Fill | Stroke | Stroke-width |
|---------|------|--------|-------------|
| Lightning bolt | `yellow` | `black` | `4` |

### Fog
| Element | Fill | Stroke | Stroke-width |
|---------|------|--------|-------------|
| Fog lines | *(none)* | `white` | `24` (keep original) |

## Universal Transformation Rules

| Original gradient/style | Visual element | Replacement |
|--------------------------|----------------|-------------|
| `fill="url(#...)"` amber/yellow gradient | Sun center | `fill="yellow"` |
| `stroke="#f8af18"` | Sun circle stroke | `stroke="black"` |
| `stroke="#fbbf24"` | Sun rays stroke | `stroke="yellow"` |
| `fill="url(#...)"` white-blue gradient (`#f3f7fe`→`#deeafb`) | Cloud (normal) | `fill="white"` |
| `stroke="#e6effc"` | Cloud stroke (light) | `stroke="black"` |
| `fill="url(#...)"` gray gradient (`#9ca3af`→`#6b7280`) | Cloud (overcast/back) | `fill="white"` or `url(#cloudGradient)` for extreme |
| `stroke="#848b98"` or `stroke="#5b6472"` | Cloud stroke (dark) | `stroke="black"` |
| `fill="url(#...)"` teal gradient (`#86c3db`→`#5eafcf`) | Moon crescent | `fill="blue"` |
| `stroke="#72b9d5"` | Moon stroke | `stroke="black"` |
| `fill="url(#...)"` blue gradient (`#0b65ed`→`#0950bc`) | Rain drops | `fill="blue"` |
| `stroke="#0a5ad4"` | Rain drop stroke | **remove entirely** |
| `fill="url(#...)"` teal gradient (`#86c3db`→`#5eafcf`) | Ice crystal / sleet | `fill="white"` |
| `stroke="#86c3db"` | Ice crystal stroke | `stroke="blue"` |
| `fill="url(#...)"` amber gradient (`#f7b23b`→`#f59e0b`) | Lightning bolt | `fill="yellow"` |
| `stroke="#f6a823"` | Lightning bolt stroke | `stroke="black"` |
| `stroke="url(#...)"` gray gradient (`#d4d7dd`→`#bec1c6`) | Fog lines | `stroke="white"` |

## Modification Procedure

1. **Remove all `<linearGradient>` definitions** (except `cloudGradient` for extreme variants)
2. **Replace gradient fills** with flat palette colors per the table above
3. **Replace subtle strokes** with `stroke="black"` for visibility
4. **Remove rain drop strokes** entirely (the blue fill is sufficient)
5. **Add comments** explaining each change from original (e.g., `<!-- Was gradient fill, now solid yellow + black stroke -->`)
6. **Increase stroke-width** on clouds to 6-12 for e-paper legibility
7. **Ensure small clouds have stroke-width** — in 2-cloud compositions (overcast/extreme variants), the small background cloud must have `stroke-width="6"` or it will be nearly invisible
8. **Add viewBox padding** — use `210.3 126.1` for small cloud symbols (not `200.3`), and `350 223` for big cloud symbols (not `350 222`) to prevent stroke clipping at edges

## Icon Status (weather_code.rs)

### Already modified (fit for e-ink display)
- `clear-day.svg`, `clear-night.svg`
- `partly-cloudy-day.svg`, `partly-cloudy-night.svg`
- `partly-cloudy-day-drizzle.svg`, `partly-cloudy-night-drizzle.svg`
- `partly-cloudy-day-rain.svg`, `partly-cloudy-night-rain.svg`
- `partly-cloudy-day-snow.svg`, `partly-cloudy-night-snow.svg`
- `overcast-day.svg`, `overcast-night.svg`
- `overcast-day-drizzle.svg`, `overcast-night-drizzle.svg`
- `overcast-day-rain.svg`, `overcast-night-rain.svg`
- `overcast-day-snow.svg`, `overcast-night-snow.svg`
- `extreme-day.svg`, `extreme-night.svg`
- `extreme-day-drizzle.svg`, `extreme-night-drizzle.svg`
- `extreme-day-rain.svg`, `extreme-night-rain.svg`
- `extreme-day-snow.svg`, `extreme-night-snow.svg`

### Modified in this batch (February 2026)
- `fog-day.svg`, `fog-night.svg`
- `partly-cloudy-day-sleet.svg`, `partly-cloudy-night-sleet.svg`
- `overcast-day-sleet.svg`, `overcast-night-sleet.svg`
- `extreme-day-sleet.svg`, `extreme-night-sleet.svg`
- `thunderstorms-day.svg`, `thunderstorms-night.svg`
- `thunderstorms-day-rain.svg`, `thunderstorms-night-rain.svg`
- `thunderstorms-day-extreme-rain.svg`, `thunderstorms-night-extreme-rain.svg`

All 36 icons referenced by `weather_code.rs` are now e-ink adapted.

## Verification

```bash
# Validate all icon filenames referenced in code exist on disk
RUN_MODE=test cargo test --test icon_name_validation_test

# Run snapshot tests and review SVG output
RUN_MODE=test cargo test --test snapshot_provider_test
cargo insta review

# Visual inspection with VS Code SVG preview (dark background)
# Use extension: jock.svg
```

---

*Created: February 14, 2026*
