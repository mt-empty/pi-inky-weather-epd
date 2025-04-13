# Pi Inky Weather Display

This is a weather display powered by a Raspberry pi and a 7.3in 7 color E-Paper (aka E-ink) display. Current and forecasted weather data is obtained from the Australian Bureau of Meteorology API.

![alt text](./misc/dashboard.png)

## Setup on Raspberry Pi 

1. Get six character hash from https://geohash.softeng.co

2. Manually install bersion **1.5.0** release of the inky library, refer to the official [documentation](https://github.com/pimoroni/inky?tab=readme-ov-file#install-stable-library-from-pypi-and-configure-manually)
3. Download and extract the latest release from the [releases page](https://github.com/mt-empty/pi-inky-weather-epd/releases) for your architecture
4. Create a config file at `~/.config/pi-inky-weather-epd.toml` with the following content:
```toml
[api]
location = "<Six character hash>"

[misc]
python_script_path = "/home/dietpi/Pimoroni/inky/examples/7color/image.py" # this is located in the files created by the inky library
python_path = "<The path to the python executable created by inky install script>" # should be in the environment created by the inky library
```
5. Create a cron job to run the script every hour 
   
   ```
   0 * * * * cd <extracted directory location>arm-unknown-linux-gnueabihf && sudo ./pi-inky-weather-epd > <extracted directory location>/inky.log 2>&1
   ```

### Troubleshooting

cd into the extracted directory and run the script manually to see if there are any errors

```bash
cd <extracted directory location>arm-unknown-linux-gnueabihf
sudo ./pi-inky-weather-epd
```

### Config

You can override the default config which are located at [./config/](./config/) by creating a file at:
```bash
~/.config/pi-inky-weather-epd.toml
```

### Dietpi distro only

For **dietpi** distro, I had to set this config for the installion script to work:
```
include-system-site-packages = true
```
I also had to modify the installation script to have pip3 point to the environment pip3 as opposed to the system pip3

I also had to modify to manually install script to have pip3 point to environment pip3 as oppose to system pip3


## TODO
- [ ] Testing: create a script that auto generates some/all weather variations
- [ ] Find out all the supported colors for the e-ink display
  - [x] purple/whitish color?
  - [x] Deep Pink
  - [ ] Other? 


## Inky Impression 7.3

### supported colors

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
```
[255, 248, 220, 255], // Cornsilk
[255, 250, 205, 255], // Lemon Chiffon
[255, 20, 147, 255],  // DeepPink
```

### Documentation

- EPD used: Inky Impression 7.3 https://shop.pimoroni.com/products/inky-impression-7-3?variant=40512683376723
- Actual Panel: Waveshare display https://www.waveshare.com/7.3inch-e-paper-hat-f.htm
- Panel documentation: https://www.waveshare.com/wiki/7.3inch_e-Paper_HAT_(F)_Manual#Overview
- API: https://github.com/bremor/bureau_of_meteorology/blob/main/api%20doc/API.md
- icons are based on: https://bas.dev/work/meteocons

## Developing

Your local config should go into `config/local.toml`, this file is not tracked
```bash
cp config/development.toml config/local.toml
cargo run
```

### mDns

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
  User <your username or the raspberry pi>
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
