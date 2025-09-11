#!/bin/bash

# Usage: ./run.sh path/to/file.rs

# Check if a file argument was provided
if [ -z "$1" ]; then
  echo "Usage: $0 <file>"
  exit 1
fi

FILE="$1"

# Detect session type (X11 or Wayland)
if [ "$XDG_SESSION_TYPE" = "wayland" ]; then
  echo "Detected Wayland session"
  SESSION="wayland"
elif [ "$XDG_SESSION_TYPE" = "x11" ]; then
  echo "Detected X11 session"
  SESSION="x11"
else
  echo "Could not detect session type, defaulting to X11"
  SESSION="x11"
fi

# Allow docker to connect to X server (only for X11)
if [ "$SESSION" = "x11" ]; then
  xhost +local:docker
fi

# Run container with appropriate environment
if [ "$SESSION" = "x11" ]; then
  sudo docker run --rm -it \
    --network=host \
    -e DISPLAY=$DISPLAY \
    -e WINIT_UNIX_BACKEND=x11 \
    -v /tmp/.X11-unix:/tmp/.X11-unix \
    -v $(realpath "$FILE"):/app/$(basename "$FILE") \
    mars9563/tty-doc /app/$(basename "$FILE")
else
  sudo docker run --rm -it \
    --network=host \
    -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY \
    -e XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR \
    -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY \
    -v $(realpath "$FILE"):/app/$(basename "$FILE") \
    mars9563/tty-doc /app/$(basename "$FILE")
fi
