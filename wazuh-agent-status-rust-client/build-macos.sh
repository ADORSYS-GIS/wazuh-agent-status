#!/bin/bash
# MacOS Build Script for Wazuh Agent Status Rust Client
# Run this script on a Mac machine with Rust installed.

set -e

echo "🍎 Preparing MacOS build..."

# 1. Install cargo-packager for .app and .dmg creation
if ! command -v cargo-packager &> /dev/null; then
    echo "Installing cargo-packager..."
    cargo install cargo-packager --locked
fi

# 2. Build for both Intel and Apple Silicon (Universal Binary)
echo "Building Universal Binary..."
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# 3. Create a thin wrapper or use lipo to combine them (optional)
# For now, we'll just build the native architecture for the test
cargo packager --release

echo "✅ MacOS App created in: ./target/release/wazuh-agent-status.app"
echo "💡 You can now send the .app (zipped) to your teammate."
