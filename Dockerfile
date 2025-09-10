# -------- Builder stage --------
FROM rust:1.82-slim AS builder

# Install system dependencies for building egui/eframe
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libx11-dev \
    libxcb1-dev \
    libxrandr-dev \
    libxi-dev \
    libgl1-mesa-dev \
    libxrender-dev \
    libxinerama-dev \
    libxcursor-dev \
    libxft-dev \
    libxkbcommon-dev \
    libfontconfig1-dev \
    libasound2-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy project files
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release || true

COPY src ./src
RUN cargo build --release

# -------- Runtime stage --------
FROM debian:bookworm-slim

# Install ALL necessary runtime libraries for GUI applications
RUN apt-get update && apt-get install -y \
    libx11-6 libxext6 libxrandr2 libxi6 libxrender1 \
    libxcursor1 libxinerama1 libxft2 libxkbcommon0 \
    libgl1-mesa-glx libgl1-mesa-dri libglu1-mesa \
    libfontconfig1 libasound2 \
    libxcb1 libxcb-dri2-0 libxcb-dri3-0 libxcb-present0 \
    libxcb-sync1 libxshmfence1 libxxf86vm1 \
    libdrm2 libdrm-amdgpu1 libdrm-intel1 libdrm-nouveau2 libdrm-radeon1 \
    x11-apps mesa-utils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/tty_doc /app/tty_doc
RUN chmod +x /app/tty_doc

# Simple entrypoint that just runs the app
RUN echo '#!/bin/bash\necho "Starting tty_doc..."\nexec /app/tty_doc "$@"' > /app/entrypoint.sh && chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]