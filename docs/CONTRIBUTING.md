# Contributing

Contributions are welcome!

If you are going to work on an issue, mention so in the issue comments before you start working on the issue.

## Submitting a Pull Request

Before submitting, please make sure the following is done:

- That there is a related issue and it is referenced in the PR text.
- There are tests that cover the changes.
- Ensure cargo fmt, clippy and test passes, enable the git hooks [below](#development-setup).

## Development Setup

1. **Install Git hooks** (pre-push checks for formatting, tests, and version tags):

   ```bash
   ./scripts/setup-git-hooks.sh
   ```

2. **Create local config** in `config/local.toml`:

   ```bash
   cp config/development.toml config/local.toml
   # Edit config/local.toml with your location settings
   cargo run
   ```

   Use [zoom.earth](https://zoom.earth/maps/radar/#view=40.7128,-74.0060,5z) to simulate different weather conditions by finding a location with the desired weather and using its latitude and longitude in your config.

## Running Tests

```bash
# Run all tests — covers both providers and all render options in one pass,
# fully in parallel; each test builds its own settings value (see tests/helpers)
cargo test

# Run only the BOM dashboard snapshot test
cargo test --test snapshot_test provider::bom_dashboard

# Review snapshot changes (uses insta crate)
cargo insta review
```

## Dashboard Simulation

Generate 24 hours of dashboard images for testing time-dependent features or creating animations:

```bash
# Build with CLI support (not included in production builds)
cargo build --features cli

# Generate 24 hourly dashboards
./scripts/simulate-24h.sh [date] [start_hour] [timezone]

# Examples:
./scripts/simulate-24h.sh                                       # Default: today from midnight UTC
./scripts/simulate-24h.sh 2025-12-26 6                          # Start from 6am
./scripts/simulate-24h.sh 2025-12-26 0 "Australia/Melbourne"    # With specific timezone
```

**How it works:**
- Automatically fetches fresh weather data before simulation base on your current config
- Generates 24 hourly dashboards using consistent cached data
- Output saved to `simulation_output/` directory, including rendered PNGs in `simulation_output/png/`

To turn that 24-hour set straight into a timelapse GIF (optionally for a
different location, using live API data), see `./scripts/generate-timelapse-gif.sh` in
`misc/gif-generation-commands.md`.

## Cross-Compilation for Target Release

Example for Raspberry Pi Zero:

```bash
cross build --release --target arm-unknown-linux-gnueabihf
```

## Wish List

- [ ] An algorithm that is smooth and only overshoots in the x dimension
- [x] Rain gradient that looks like rain
- [ ] Overhaul the [line SVG icons](../static/line-svg-static/) to match display colours
- [ ] Inline all SVG icons into the template and have full control over all colours

## Development Tips

This is optional, but you can use **mDNS** to access your Pi by hostname instead of IP address

To do this, you need to install **avahi-daemon** on your Pi. This is a service that allows you to discover devices on the network using their hostname

```bash
sudo apt-get install avahi-daemon
# Modify /etc/hosts to include your <hostname>.local
# 127.0.0.1   <hostname>.local <hostname>
sudo systemctl enable avahi-daemon
sudo systemctl start avahi-daemon
```

The Pi should now be discoverable by `<hostname>.local`

Add to your `~/.ssh/config`:

```ssh-config
Host pizero
  Hostname <hostname>.local
  User <your-username>
  IdentityFile <path-to-your-private-key>
  ServerAliveInterval 60
  ServerAliveCountMax 240
```

SSH into it by running `ssh pizero`

## Sending Image to Pi Over SSH

Once you have your SSH setup:

```bash
cargo run   # to generate the image

chmod +x ./scripts/send-img-to-pi.sh
./scripts/send-img-to-pi.sh
```

## Troubleshooting

- Execute `./pi-inky-weather-epd` separately and observe the logs for any errors, then open the generated image to see if it is correct
- Run the cron script manually to see if there are any issues

### Intermittent "API Unreachable" on DietPi (Pi Zero W)

If the dashboard intermittently falls back to stale data with a DNS
resolution error (`failed to lookup address information: Temporary failure
in name resolution`), the likely cause on DietPi is DHCP lease renewals
clobbering `/etc/resolv.conf` and knocking out `systemd-resolved`'s stub
listener. See [issue #46](https://github.com/mt-empty/pi-inky-weather-epd/issues/46#issuecomment-4959404620)
for the root cause and fix (pinning `dhclient` to keep writing
`127.0.0.53`, setting an explicit upstream `DNS=` for `resolved`, and
cycling the interface — not just restarting a service — to pick up the fix).

### Issues with Latest Version of Inky

If you encounter issues with the latest version of Inky, try manually installing version **1.5.0** of the Inky library. Refer to the [official documentation](https://github.com/pimoroni/inky?tab=readme-ov-file#install-stable-library-from-pypi-and-configure-manually).

### Special Instructions for DietPi

For **DietPi** distro working with version **1.5**, you may need to set `include-system-site-packages = true` in your Python virtual environment.

To do this, after creating your virtual environment (e.g., with `python3 -m venv /path/to/env`), open the file:

```text
/path/to/env/pyvenv.cfg
```

And add or update this line:

```ini
include-system-site-packages = true
```

This allows the virtual environment to access system-wide Python packages, which may be required for the installation script to work.

You may also need to modify the Inky installation script so that `pip3` points to the created environment's `pip3` instead of the system `pip3`.
