# Yes another inky weather display

This project involves building a simple, static weather dashboard using Rust. The dashboard will display weather data fetched from an API, including temperature over time for a single day, and will integrate SVG icons and a single SVG graph.


![alt text](image.png)

## Tasks
- [x] fetch data from weather API
- [x] find proper svg weather icons
- [x] display weather data
- [x] display temperature over time using graph
- [x] convert svg to png
- [x] find proper font for e-ink display
  - [x] Genererate Roboto-Regular-Dashed.ttf
- [ ] create a program that auto generates some/all possible weather variations
- [x] modify existing icons
  - [x] wind speed with km/h
  - [x] evey other one to only use colors from the e-ink supported palette
- [ ] break the program into smaller parts
- [ ] propper error logging
  - [ ] add a warning icon when fetching data fails
  - [ ] when `NA` is returned, including for icons
- [ ] Find out all the supported colors for the e-ink display
  - [ ] Deep Pink 
  - [ ] purple white color?


https://github.com/bremor/bureau_of_meteorology/blob/main/api%20doc/API.md


http://geohash.co/

https://www.waveshare.com/wiki/7.3inch_e-Paper_HAT_(F)_Manual#Overview

### Supported colors
        [0, 0, 0],        # Black
        [255, 255, 255],  # White
        [0, 255, 0],      # Green
        [0, 0, 255],      # Blue
        [255, 0, 0],      # Red
        [255, 255, 0],    # Yellow
        [255, 20, 147] # Deep Pink
        [255, 140, 0],    # Orange
        
        # [255, 248, 220, 255], // Cornsilk
        # [255, 250, 205, 255], // Lemon Chiffon
        # [255, 20, 147, 255],  // DeepPink