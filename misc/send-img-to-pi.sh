#!/bin/sh

# You need to set up your ssh keys to use this script
# And it should be run from the root of the project


# Default saturation if not provided
SATURATION=${1:-1.0}

REMOTE_HOST="pizero"

REMOTE_TMP="/tmp"
LOCAL_IMAGE="dashboard.png"
REMOTE_IMAGE="${REMOTE_TMP}/dashboard.png"
PYTHON_PATH="/home/dietpi/env/bin/python3"
INKY_SCRIPT="/home/dietpi/Pimoroni/inky/examples/7color/image.py"

# Transfer the image
scp "${LOCAL_IMAGE}" "${REMOTE_HOST}:${REMOTE_IMAGE}"

# Execute the remote command
ssh "${REMOTE_HOST}" "sudo ${PYTHON_PATH} ${INKY_SCRIPT} ${REMOTE_IMAGE} ${SATURATION}"