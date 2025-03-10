# Pi Inky weather display

This is a weather display powered by a Raspbery pi and a 7.3in 7 color E-Paper (aka E-ink) display. Current and forecasted weather data is obtained from the Australian Bureau of Meteorology API.

![alt text](./misc/dashboard.png)

## TODO
- [ ] create a script that auto generates some/all weather variations
- [ ] Find out all the supported colors for the e-ink display
  - [x] purple/whitish color?
  - [x] Deep Pink
  - [ ] Other? 


API: https://github.com/bremor/bureau_of_meteorology/blob/main/api%20doc/API.md


## Setup on Raspberry Pi Zero 

Get 6 character hash from https://geohash.softeng.co

### Install inky library

Refer to the officail [documentation](https://github.com/pimoroni/inky?tab=readme-ov-file#install-stable-library-from-pypi-and-configure-manually)

If version 2.0.0 doesn't work for you, try version 1.5.0

For **dietpi** distro, I had to set this config for the installion script to work:
```
include-system-site-packages = true
```
This enables the use of system wide python packages provided by apt

I also had to modify to manually install script to have pip3 point to environment pip3 as oppose to system pip3


```bash

0 * * * * cd /home/dietpi/arm-unknown-linux-gnueabihf && sudo ./pi-inky-weather-epd > /home/dietpi/inky.log 2>&1
```


### Supported colors

[0, 0, 0],        # Black
[255, 255, 255],  # White
[0, 255, 0],      # Green
[0, 0, 255],      # Blue
[255, 0, 0],      # Red
[255, 255, 0],    # Yellow
[255, 20, 147] # Deep Pink
[255, 140, 0],    # Orange

[255, 248, 220, 255], // Cornsilk
[255, 250, 205, 255], // Lemon Chiffon
[255, 20, 147, 255],  // DeepPink



EPD used: Inky Impression 7.3 https://shop.pimoroni.com/products/inky-impression-7-3?variant=40512683376723

Actual Panel: Waveshare display https://www.waveshare.com/7.3inch-e-paper-hat-f.htm

Panel documentation: https://www.waveshare.com/wiki/7.3inch_e-Paper_HAT_(F)_Manual#Overview


## Config

You can override the default config by creating a file at
```bash
~/.config/pi-inky-weather-epd.toml
```


## Executing on PI

```bash
0 * * * * cd /home/dietpi/arm-unknown-linux-gnueabihf && sudo ./pi-inky-weather-epd
```

## Development

Your local config should go into `config/local.toml`, this file is not tracked
```bash
cp config/development.toml config/local.toml
cargo run
```
### Developing 

### mDns

This is optional, but you can use **mDns** to access your pi by hostname instead of IP address.

```bash
sudo apt-get install avahi-daemon
sudo systemctl enable avahi-daemon
sudo systemctl start avahi-daemon
```

The pi should now be discoverable by `<hostname>.local`

```config
Host pizero
  Hostname <hostname>.local
  User <your username or the raspberry pi>
  IdentityFile <path to your private key>
  ServerAliveInterval 60
  ServerAliveCountMax 240
```

Now you can access the pi by name `<hostname>.local`

### Sending image to pi zero

Once you have your ssh setup:

```bash
chmod +x ./misc/send-img-to-pi.sh
./misc/send-img-to-pi.sh
```

https://github.com/esp-rs/awesome-esp-rust
https://harrystern.net/halldisplay.html