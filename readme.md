# Yes another inky weather display

This is a weather display powered by a Raspbery pi and a 7.3in 7 color E-Paper (aka E-ink) display. Current and forecasted weather data is obtained from the Australian Bureau of Meteorology API. The display is updated every 1 hour.

![alt text](dashboard.png)

## TODO
- [ ] create a script that auto generates some/all weather variations
- [ ] Find out all the supported colors for the e-ink display
  - [ ] Deep Pink 
  - [ ] purple/whitish color?


API: https://github.com/bremor/bureau_of_meteorology/blob/main/api%20doc/API.md


## Setup 

Get 6 character hash from https://geohash.softeng.co



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


## Development

Your local config should go into `config/local.toml`, this file is not tracked
```bash
cp config/development.toml config/local.toml
```