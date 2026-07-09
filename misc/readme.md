# Showcase Assets

`misc/timelapse.gif` is generated deterministically from frozen fixture
data checked into `misc/showcase_fixtures/`, not from a live API call —
this keeps it reproducible and lets us hand-pick weather conditions for
visual variety instead of whatever happened to be live when someone last
ran the generator. Re-run it after any rendering change:

```bash
./scripts/generate-showcase.sh
```

This writes straight to `misc/timelapse.gif`; review the diff before
committing. To add a new scenario, drop a fixture pair (matching
`open_meteo_hourly_forecast.json` / `open_meteo_daily_forecast.json` shape,
see `tests/fixtures/`) into `misc/showcase_fixtures/<name>/` with a fixed,
contiguous hourly time window (the daily window needs enough trailing days
to survive timezone rollover — see comments in the script).
