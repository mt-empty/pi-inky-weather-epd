# Pi Inky Weather Display

This is a weather display powered by a Raspberry pi and a 7.3in 7 color E-Paper (aka E-ink) display. Current and forecasted weather data is obtained from the Australian Bureau of Meteorology API.

![alt text](./misc/dashboard.png)

## Hardware
- Raspberry Pi (zero model requires soldering the GPIO Header)
- 7.3in E-Paper display (Inky Impression 7.3)
- 3D printed case (optional)

## Setup on Raspberry Pi 

1. Install the Inky library:
   ```bash
   curl https://get.pimoroni.com/inky | bash
   ```
   For detailed installation steps, refer to the official [documentation](https://github.com/pimoroni/inky?tab=readme-ov-file#install-stable-library-from-pypi-and-configure-manually).

2. Download the latest release for your architecture from the [releases page](https://github.com/mt-empty/pi-inky-weather-epd/releases) and extract it:
   ```bash
   unzip <YOUR_DOWNLOAD_RELEASE>.tar.gz
   chmod +x pi-inky-weather-epd
   ```
3. Obtain a six-character geohash for your location from https://geohash.softeng.co

4. Create a configuration file with your geohash:
   ```bash
   echo -e '[api]\nlocation = "YOUR_GEOHASH"' > ~/.config/pi-inky-weather-epd.toml
   ```

   See [./config/development.toml](./config/default.toml) for example cities and their geohashes.

5. Set up an hourly cron job to update the display:
   ```bash
   (crontab -l 2>/dev/null; echo "0 * * * * cd /path/to/extracted/files && ./pi-inky-weather-epd && sudo PYTHON_PATH IMAGE_SCRIPT_PATH --file dashboard.png --saturation SATURATION") | crontab -
   ```
   Replace:
   - `/path/to/extracted/files` with your installation directory
   - `PYTHON_PATH` with path to Python (e.g., `/usr/bin/python3`)
   - `IMAGE_SCRIPT_PATH` with path to Inky's image.py (e.g., `/home/pi/Pimoroni/inky/examples/7color/image.py`)
   - `SATURATION` with the desired saturation level (e.g., `1.0`), it it not recommended change this for current icons

   Example of complete cron command:
   ```bash
   0 * * * * cd /home/pi/pi-inky-weather-epd && ./pi-inky-weather-epd && sudo /home/dietpi/env/bin/python3 /home/dietpi/Pimoroni/inky/examples/7color/image.py --file dashboard.png --saturation 1.0
   ```

## Configuration

You can override the default configs located at [./config/](./config/) by creating a file at:
```bash
~/.config/pi-inky-weather-epd.toml
```

### Special Instructions for DietPi

For **dietpi** distro, set this config for the installation script to work:
```
include-system-site-packages = true
```
I also had to modify the installation script to have pip3 point to the environment pip3 as opposed to the system pip3

## Inky Impression 7.3

### supported colors at 1.0 Saturation

```
[0, 0, 0],        # Black
[255, 255, 255],  # White
[0, 255, 0],      # Green
[0, 0, 255],      # Blue
[255, 0, 0],      # Red
[255, 255, 0],    # Yellow
[255, 140, 0],    # Orange
```

#### Trial and error found colors

These colors where found by trail and error
```
[255, 248, 220, 255], // Cornsilk
[255, 250, 205, 255], // Lemon Chiffon
[255, 20, 147, 255],  // DeepPink
```

## Documentation

- EPD used: Inky Impression 7.3 https://shop.pimoroni.com/products/inky-impression-7-3?variant=40512683376723
- Actual Panel: Waveshare display https://www.waveshare.com/7.3inch-e-paper-hat-f.htm
- Panel documentation: https://www.waveshare.com/wiki/7.3inch_e-Paper_HAT_(F)_Manual#Overview
- API: https://github.com/bremor/bureau_of_meteorology/blob/main/api%20doc/API.md
- icons are based on: https://bas.dev/work/meteocons


## TODO
- [ ] Testing: create a script that auto generates some/all weather variations
  - This script should simulate different weather conditions (e.g., sunny, rainy, cloudy) and generate corresponding images for testing the display.

## Developing

Any Contributions are welcome!

### Setup

Your local config should go into `config/local.toml`:
```bash
cp config/development.toml config/local.toml
cargo run
```

### Compile for target release

Example for Raspberry Pi Zero:

```bash
cross build --release --target arm-unknown-linux-gnueabihf 
```

### Using mDNS for Easy Access

This is optional, but you can use **mDns** to access your pi by hostname instead of IP address.

To do this, you need to install **avahi-daemon** on your pi. This is a service that allows you to discover devices on the network using their hostname.

```bash
sudo apt-get install avahi-daemon
sudo systemctl enable avahi-daemon
sudo systemctl start avahi-daemon
```

The pi should now be discoverable by `<hostname>.local`

```
Host pizero
  Hostname <hostname>.local
  User <your username on the raspberry pi>
  IdentityFile <path to your private key>
  ServerAliveInterval 60
  ServerAliveCountMax 240
```

ssh into it by running `ssh pizero`

### Sending image to pi over ssh 

Once you have your ssh setup:

```bash
chmod +x ./misc/send-img-to-pi.sh
cargo run
./misc/send-img-to-pi.sh
```

## Troubleshooting

Check the logs for any errors:
```bash
tail -f inky.log
```
Then `cd` into the extracted directory and run the cron script manually to see if there are any errors

#### Issues with latest version of Inky 

If you encounter issues with the latest version of Inky, try manually installing version **1.5.0** release of the inky library, refer to the official [documentation](https://github.com/pimoroni/inky?tab=readme-ov-file#install-stable-library-from-pypi-and-configure-manually)