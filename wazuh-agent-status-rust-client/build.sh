#!/bin/bash
set -e

PROJECT_DIR="wazuh-agent-status-rust-client"
IMAGE_NAME="wazuh-rust-builder"

echo "🚀 Building Docker image: $IMAGE_NAME..."
docker build -t $IMAGE_NAME .

echo "🏗️ Extracting AppImage to host..."
CONTAINER_ID=$(docker create $IMAGE_NAME)
docker cp $CONTAINER_ID:/usr/src/app/Wazuh_Agent_Status-0.1.0-x86_64.AppImage .
docker rm $CONTAINER_ID

# Rename for easier use
mv Wazuh_Agent_Status-0.1.0-x86_64.AppImage wazuh-agent-status.AppImage
chmod +x wazuh-agent-status.AppImage

echo "✅ Build complete! Standalone AppImage is at: ./wazuh-agent-status.AppImage"
echo "💡 To run (no dependencies needed): ./wazuh-agent-status.AppImage"
