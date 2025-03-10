#!/bin/sh

# You need to set up your ssh keys to use this script
# And it should be run from the root of the project

SATURATION=1.0

scp dashboard.png pizero:/tmp/dashboard.png

ssh pizero "sudo /home/dietpi/env/bin/python3 /home/dietpi/Pimoroni/inky/examples/7color/image.py /tmp/dashboard.png $SATURATION"