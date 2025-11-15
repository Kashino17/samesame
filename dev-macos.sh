#!/bin/bash

# Development script for macOS Client

set -e

echo "🚀 Starting SameSame macOS Client in development mode..."
echo ""

# Navigate to macOS client directory
cd "$(dirname "$0")/macos-client"

# Install npm dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing npm dependencies..."
    npm install
fi

# Run in development mode with debug logs
echo "Starting development server..."
RUST_LOG=info npm run tauri dev
