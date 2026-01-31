#!/bin/bash
# Quick build and run script for Kalshi monitor

echo "🦀 Building Kalshi Monitor..."
cargo build

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "🚀 Running monitor (Ctrl+C to stop)..."
    echo ""
    cargo run
else
    echo ""
    echo "❌ Build failed. Check errors above."
    exit 1
fi
