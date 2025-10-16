# ================================
# 1️⃣ Stage: Build (Rust 1.82)
# ================================
FROM rust:1.82 AS builder

# Install system dependencies for eframe/egui GUI and network features
RUN apt-get update && apt-get install -y \
    libssl-dev pkg-config libgtk-3-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libwayland-dev build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifest files first (for dependency caching)
COPY Cargo.toml Cargo.lock ./

# Copy full source code
COPY src ./src

# Build in release mode
RUN cargo build --release

# ================================
# 2️⃣ Stage: Runtime (Slim Debian)
# ================================
FROM debian:bookworm-slim

# Install only required runtime libraries for GUI (X11 + Wayland) and network
RUN apt-get update && apt-get install -y \
    libgtk-3-0 \
    libx11-6 \
    libx11-xcb1 \
    libxcb1 \
    libxcb-render0 \
    libxcb-shape0 \
    libxcb-xfixes0 \
    libxrandr2 \
    libxi6 \
    libxrender1 \
    libxinerama1 \
    libxcursor1 \
    libxkbcommon0 \
    libwayland-client0 \
    libwayland-cursor0 \
    libwayland-egl1 \
    libgl1 \
    libgl1-mesa-glx \
    libgl1-mesa-dri \
    libegl1-mesa \
    libgles2-mesa \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/tty_doc /usr/local/bin/tty_doc

# Working directory for mounted files
WORKDIR /workspace

# Set environment variables for both X11 and Wayland
ENV RUST_LOG=info \
    DISPLAY=:0 \
    WAYLAND_DISPLAY=wayland-0 \
    XDG_RUNTIME_DIR=/tmp

# Default entrypoint
ENTRYPOINT ["tty_doc"]
CMD [""]
