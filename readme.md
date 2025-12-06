# Pi Inky Weather Display

[![Build Status](https://github.com/mt-empty/pi-inky-weather-epd/actions/workflows/test.yml/badge.svg?branch=master)](https://github.com/mt-empty/pi-inky-weather-epd/actions/workflows/test.yml)

A weather display powered by a Raspberry Pi and a 7.3" 7-colour E-Paper (E-ink) display. The generation of the image is abstracted away from the hardware, so it can be used with any stack.

![Hourly timelapse](./misc/timelapse.gif)

Note, the gif and images are a bit outdated, specifically UV icon colour changes depending on the UV gradient.

## Hardware

- Raspberry Pi (Zero model requires soldering the GPIO header)
- [Inky Impression 7.3" E-Paper display](https://shop.pimoroni.com/products/inky-impression-7-3?variant=55186435244411)
- [3D printed case](https://github.com/mt-empty/inky-impression-7-3-colour-case) (optional)

![Dashboard Case](./misc/dashboard-case.png)

## Setup on Raspberry Pi

1. **Install the Inky library:**

   ```bash
   curl https://get.pimoroni.com/inky | bash
   ```

   For detailed installation steps, refer to the official [documentation](https://github.com/pimoroni/inky?tab=readme-ov-file#install-stable-library-from-pypi-and-configure-manually).

2. **Download the latest release:**

   Download the latest release for your architecture from the [releases page](https://github.com/mt-empty/pi-inky-weather-epd/releases) and extract it:

   Available architectures:
   - `arm-unknown-linux-gnueabihf` - Pi 1, Pi Zero/Zero W, Compute Module 1 (ARMv6)
   - `armv7-unknown-linux-gnueabihf` - Pi 2, Pi 3/3+/3A+, Pi 4/400, Zero 2 W, CM3/CM4 (32-bit OS, ARMv7)
   - `aarch64-unknown-linux-gnu` - Pi 3/3+/3A+, Pi 4/400, Zero 2 W, CM3/CM4 (64-bit OS, ARMv8)
   - `x86_64-unknown-linux-gnu` - x86 Linux

   ```bash
   unzip <YOUR_DOWNLOAD_RELEASE>.zip
   chmod +x pi-inky-weather-epd
   ```

3. **Configure your weather data provider and location:**

   Get your latitude and longitude from <https://www.latlong.net/> and create a configuration file:

   ```bash
   mkdir -p ~/.config
   cat > ~/.config/pi-inky-weather-epd.toml << EOF
   [api]
   latitude = YOUR_LATITUDE   # e.g., -33.8727 # Sydney
   longitude = YOUR_LONGITUDE # e.g., 151.2057
   provider = "open_meteo"    # "open_meteo" (worldwide) or "bom" (Australia only)
   EOF
   ```

   See [./config/development.toml](./config/development.toml) for more configuration examples.

4. **Set up an hourly cron job to update the display:**

   ```bash
   (crontab -l 2>/dev/null; echo "0 * * * * cd /path/to/extracted/files && ./pi-inky-weather-epd && sudo <PYTHON_PATH> <IMAGE_SCRIPT_PATH> --file dashboard.png --saturation <SATURATION>") | crontab -
   ```

   Replace:
   - `/path/to/extracted/files` with your installation directory
   - `<PYTHON_PATH>` with path to Python (e.g., `/usr/bin/python3`)
   - `<IMAGE_SCRIPT_PATH>` with path to Inky's `image.py` (e.g., `/home/pi/Pimoroni/inky/examples/7color/image.py`)
   - `<SATURATION>` with the desired saturation level depending on your display (e.g., `1.0`). If using the Inky Impression 7 colours, it is not recommended to change this for current icons

   **Example of complete cron command:**

   ```bash
   0 * * * * cd /home/pi/pi-inky-weather-epd && ./pi-inky-weather-epd && sudo /home/dietpi/env/bin/python3 /home/dietpi/Pimoroni/inky/examples/7color/image.py --file dashboard.png --saturation 1.0
   ```

## Configuration

You can override the default configs located at [./config/](./config/) by creating a file at:

```bash
~/.config/pi-inky-weather-epd.toml
```

### Configuration Examples

Here are example configurations. Note: some of these images are slightly outdated.

#### Default Configuration

<img src="./misc/dashboard-default.png" alt="Default configuration" width="600"/>

#### Imperial Units

```toml
[render_options]
temp_unit = "F"
wind_speed_unit = "mph"
```

#### Date Format

You can customise the date format using chrono strftime specifiers. The default is `"%A, %d %B"` (e.g., "Saturday, 06 December").

```toml
[render_options]
# Example formats:
# date_format = "%B %-d, %Y"     # December 6, 2025 (US style)
# date_format = "%d/%m/%Y"       # 06/12/2025 (Australia/UK)
# date_format = "%m/%d/%Y"       # 12/06/2025 (USA)
# date_format = "%Y-%m-%d"       # 2025-12-06 (ISO 8601)
# date_format = "%a, %-d %b"     # Sat, 6 Dec
# date_format = "%d.%m.%Y"       # 06.12.2025 (Germany)

date_format = "%A, %d %B"
```

See [chrono strftime documentation](https://docs.rs/chrono/latest/chrono/format/strftime/) for all available format specifiers.

#### Use Clear night Icon instead of Moon Phase icon when Time=night and Weather=clear

<img src="./misc/dashboard-without-moon-phase.png" alt="Moon phase configuration" width="600"/>

When the sky is clear, the moon phase icon is used instead of the clear night icon, you can disable with:

```toml
[render_options]
use_moon_phase_instead_of_clear_night = false
```

#### Set X-Axis Placement to be always at y=0

<img src="./misc/dashboard-x-axis-at-zero.png" alt="X-axis at minimum" width="600"/>

The x-axis is no longer at the bottom of the graph when the temperature is below zero, it is now always positioned at x = 0.

```toml
[render_options]
x_axis_always_at_min = false
```

#### Dark Theme

<img src="./misc/dashboard-dark.png" alt="Dark theme" width="600"/>

```toml
[colours]
background_colour   = "black"
text_colour         = "white"

x_axis_colour       = "white"
y_left_axis_colour  = "red"
y_right_axis_colour = "blue"

actual_temp_colour  = "red"
feels_like_colour   = "green"
rain_colour         = "blue"
```

#### Auto-Update Interval

Enable auto-update when a new release is available. This is disabled by default.

```toml
[release]
# Set to 0 to disable auto-updating
update_interval_days = 7
```

## Degraded Operation

The dashboard can still work using cached data for a while if the API is unreachable. A diagnostic icon and message appears on the display when issues occur.

| Diagnostic Type     | Priority | Icon                                                                                                |
| ------------------- | -------- | --------------------------------------------------------------------------------------------------- |
| **API Error**       | High     | <img src="./static/fill-svg-static/code-red.svg" alt="API Error" width="32" height="32" />          |
| **No Internet**     | Medium   | <img src="./static/fill-svg-static/code-orange.svg" alt="No Internet" width="32" height="32" />     |
| **Incomplete Data** | Low      | <img src="./static/fill-svg-static/code-yellow.svg" alt="Incomplete Data" width="32" height="32" /> |
| **Update Failed**   | Low      | <img src="./static/fill-svg-static/code-green.svg" alt="Update Failed" width="32" height="32" />    |

When multiple diagnostics occur, the highest priority diagnostic is displayed, lower priority ones are cascaded.

## Inky Impression 7.3

### Supported Colours at 1.0 Saturation (Without Dithering)

```rust
[0, 0, 0],        # Black
[255, 255, 255],  # White
[0, 255, 0],      # Green
[0, 0, 255],      # Blue
[255, 0, 0],      # Red
[255, 255, 0],    # Yellow
[255, 140, 0],    # Orange
```

## Documentation and Resources

- **EPD used:** [Inky Impression 7.3](https://shop.pimoroni.com/products/inky-impression-7-3?variant=40512683376723)
- **Actual Panel:** [Waveshare 7.3" E-Paper HAT](https://www.waveshare.com/7.3inch-e-paper-hat-f.htm)
- **Panel documentation:** [Waveshare Wiki](https://www.waveshare.com/wiki/7.3inch_e-Paper_HAT_(F)_Manual#Overview)
- **Open-Meteo API:** [Open-Meteo Weather Forecast API](https://open-meteo.com/en/docs) (default provider)
- **BOM API:** [Bureau of Meteorology API Documentation](https://github.com/bremor/bureau_of_meteorology/blob/main/api%20doc/API.md) (Australia only)
- **Icons:** [Custom SVG icons](./static/fill-svg-static/) overhauled from [Meteocons](https://bas.dev/work/meteocons)

## TODO

- [ ] Testing: Create a script that auto-generates some/all weather variations
  - This script should simulate different weather conditions (e.g., sunny, rainy, cloudy) and generate corresponding images for testing the display

### Wish List

- [ ] An algorithm that is smooth and only overshoots in the x dimension
- [ ] Rain gradient that looks like rain
- [ ] Overhaul the [line SVG icons](./static/line-svg-static/) to match display colours
- [ ] Inline all SVG icons into the template and have full control over all colours

## Contributing

Contributions are welcome!

### Setup

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

### Running Tests

```bash
# Run all tests with default open-meteo config
RUN_MODE=test cargo test

# Test BOM API specifically
RUN_MODE=test APP_API__PROVIDER=bom cargo test --test snapshot_provider_test snapshot_bom_dashboard -- --ignored
```

### Compile for Target Release

Example for Raspberry Pi Zero:

```bash
cross build --release --target arm-unknown-linux-gnueabihf
```

### Using mDNS for Easy Access

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

### Sending Image to Pi Over SSH

Once you have your SSH setup:

```bash
cargo run   # to generate the image

chmod +x ./scripts/send-img-to-pi.sh
./scripts/send-img-to-pi.sh
```

## Troubleshooting

- Execute `./pi-inky-weather-epd` separately and observe the logs for any errors, then open the generated image to see if it is correct
- Run the cron script manually to see if there are any issues

### Issues with Latest Version of Inky

If you encounter issues with the latest version of Inky, try manually installing version **1.5.0** of the Inky library. Refer to the official [documentation](https://github.com/pimoroni/inky?tab=readme-ov-file#install-stable-library-from-pypi-and-configure-manually).

### Special Instructions for DietPi

For **DietPi** distro working with version **1.5**, you may need to set `include-system-site-packages = true` in your Python virtual environment.

To do this, after creating your virtual environment (e.g., with `python3 -m venv /path/to/env`), open the file:

```text
/path/to/env/pyvenv.cfg
```

And add or set the following line:

```ini
include-system-site-packages = true
```

This allows the virtual environment to access system-wide Python packages, which may be required for the installation script to work.

You may also need to modify the Inky installation script so that `pip3` points to the created environment's `pip3` instead of the system `pip3`.
