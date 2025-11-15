#!/bin/bash

# Build script for macOS Client

set -e

echo "🔨 Building SameSame macOS Client..."
echo ""

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ Error: This script must be run on macOS"
    exit 1
fi

# Check for required tools
command -v node >/dev/null 2>&1 || { echo "❌ Error: Node.js is required but not installed. Install from https://nodejs.org"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "❌ Error: Rust is required but not installed. Install from https://rustup.rs"; exit 1; }

# Navigate to macOS client directory
cd "$(dirname "$0")/macos-client"

# Install npm dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing npm dependencies..."
    npm install
fi

# Build the Tauri app
echo "🚀 Building Tauri application..."
npm run tauri build

echo ""
echo "✅ Build completed successfully!"
echo ""
echo "📍 The app bundle is located at:"
echo "   $(pwd)/src-tauri/target/release/bundle/macos/"
echo ""
echo "To run the app:"
echo "   open src-tauri/target/release/bundle/macos/SameSame\ Input\ Forwarder.app"
echo ""
