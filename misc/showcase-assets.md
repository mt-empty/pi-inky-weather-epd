# Showcase Assets

`misc/timelapse.gif` and the config-example screenshots
(`misc/dashboard-*.png`) are generated deterministically from frozen
fixture data checked into `misc/showcase_fixtures/`, not from a live API
call — this keeps them reproducible and lets us hand-pick weather
conditions for visual variety instead of whatever happened to be live when
someone last ran the generator. Re-run after any rendering change:

```bash
./scripts/generate-showcase.sh
```

This writes straight to `misc/timelapse.gif` and `misc/dashboard-*.png`;
review the diff before committing.

To add a new GIF scenario, drop a fixture pair (matching
`open_meteo_hourly_forecast.json` / `open_meteo_daily_forecast.json` shape,
see `tests/fixtures/`) into `misc/showcase_fixtures/<name>/` with a fixed,
contiguous hourly time window (the daily window needs enough trailing days
to survive timezone rollover — see comments in the script).

The 4 config-example screenshots are each purpose-picked so the feature
they demonstrate is actually visible in the data, not just reusing one
generic fixture:

| Screenshot | Fixture | Why |
|---|---|---|
| `dashboard-default.png` | `gif` (9am hour) | General-purpose shot; the forward-looking graph shows the full day/rain/snow narrative |
| `dashboard-without-moon-phase.png` | `moon-phase` (clear night hour) | Needs a clear night to show the moon-phase-vs-clear-night icon difference; rendered with `use_moon_phase_instead_of_clear_night=false` |
| `dashboard-x-axis-at-zero.png` | `gif` (blizzard hour, -5°C) | Needs sub-zero temps so the fixed 0° reference line visibly separates from the chart's bottom edge |
| `dashboard-dark.png` | `gif` (9am hour) | Just a colour override, no special data needed |
