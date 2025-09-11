#!/bin/bash
# Allow docker to connect to X11
xhost +local:docker

# Run the container with X11 forwarding
sudo docker run --rm -it \
  --network=host \
  -e DISPLAY=$DISPLAY \
  -e WINIT_UNIX_BACKEND=x11 \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  -v $(pwd)/src/main.rs:/app/main.rs \
  mars9563/tty-doc /app/main.rs
