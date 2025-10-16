#!/bin/bash

# ===============================
# TTY_DOC launcher with fallback
# ===============================

# Usage: ./run.sh path/to/file.txt
# This script will automatically detect Wayland/X11 and retry with XWayland if needed.

set -e

if [ -z "$1" ]; then
  echo "Usage: $0 <file>"
  exit 1
fi

FILE="$(realpath "$1")"

# Detect session type
SESSION="${XDG_SESSION_TYPE:-x11}"
echo "Detected session type: $SESSION"

# Helper function to run docker with X11
run_x11() {
  echo "🚀 Launching under X11 (XWayland)..."
  xhost +local:docker >/dev/null 2>&1 || true

  sudo docker run --rm -it \
    --network=host \
    -e DISPLAY=$DISPLAY \
    -e WINIT_UNIX_BACKEND=x11 \
    -v /tmp/.X11-unix:/tmp/.X11-unix \
    -v "$FILE":/app/$(basename "$FILE") \
    tty-doc /app/$(basename "$FILE")
}

# Helper function to run docker with Wayland
run_wayland() {
  echo "🚀 Launching under native Wayland..."
  sudo docker run --rm -it \
    --network=host \
    -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY \
    -e XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR \
    -v "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY":"$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" \
    -v "$FILE":/app/$(basename "$FILE") \
    tty-doc /app/$(basename "$FILE")
}

# Run based on session type
if [ "$SESSION" = "wayland" ]; then
  run_wayland || {
    echo "⚠️  Wayland launch failed. Retrying with X11 (XWayland)..."
    run_x11
  }
else
  run_x11
fi

# ===============================
# Stop Ollama model after container exits
# ===============================
echo "🛑 Stopping Ollama model llama2:latest..."
ollama stop llama2:latest || {
    echo "⚠️ Failed to stop Ollama model. Make sure it is running."
}
echo "✅ Ollama model stopped."