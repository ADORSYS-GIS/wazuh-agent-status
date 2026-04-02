# Use the latest stable Rust image to support modern dependencies
FROM rust:latest

# Install system dependencies for GTK, AppIndicator, and AppImage creation
RUN apt-get update && apt-get install -y \
    pkg-config \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    libxdo-dev \
    gcc \
    curl \
    wget \
    file \
    fuse \
    imagemagick \
    && rm -rf /var/lib/apt/lists/*

# Download linuxdeploy for AppImage creation
RUN wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage -O /usr/local/bin/linuxdeploy \
    && chmod +x /usr/local/bin/linuxdeploy

# Set the working directory
WORKDIR /usr/src/app

# Copy the project files
COPY . .

# Resize icons for best quality in tray/menu BEFORE building (so they are embedded correctly)
# We use 128x128 for the tray icon to ensure sharpness on High-DPI screens
RUN convert assets/wazuh-logo.png -resize 128x128 assets/wazuh-logo.png && \
    convert assets/green-dot.png -resize 24x24 assets/green-dot.png && \
    convert assets/gray-dot.png -resize 24x24 assets/gray-dot.png

# Build the application in release mode
RUN cargo build --release

# AppImage creation requires a desktop file and an icon
RUN mkdir -p AppDir/usr/bin && \
    cp target/release/wazuh-agent-status-rust-client AppDir/usr/bin/ && \
    mkdir -p AppDir/usr/share/icons/hicolor/256x256/apps/ && \
    # Create a 256x256 resolution version for the desktop environment icon
    convert assets/wazuh-logo.png -resize 256x256! assets/wazuh-logo-256.png && \
    cp assets/wazuh-logo-256.png AppDir/usr/share/icons/hicolor/256x256/apps/wazuh-logo.png && \
    cp wazuh-agent-status.desktop AppDir/ && \
    # AppImage creation
    LINUXDEPLOY_OUTPUT_VERSION=0.1.0 linuxdeploy --appimage-extract-and-run \
    --executable AppDir/usr/bin/wazuh-agent-status-rust-client \
    --appdir AppDir \
    --desktop-file wazuh-agent-status.desktop \
    --icon-file assets/wazuh-logo-256.png \
    --output appimage

CMD ["ls", "-lh", "*.AppImage"]
