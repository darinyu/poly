#!/bin/bash
# Systematic debugging script for Kalshi monitor

set -e  # Exit on error

echo "🔍 Kalshi Monitor - Systematic Debugging"
echo "========================================"
echo ""

# Step 1: Check Rust installation
echo "Step 1: Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "   source \$HOME/.cargo/env"
    exit 1
fi
echo "✓ Cargo found: $(cargo --version)"
echo ""

# Step 2: Check .env file
echo "Step 2: Checking .env file..."
if [ ! -f .env ]; then
    echo "❌ .env file not found"
    echo "   cp .env.example .env"
    echo "   Then edit .env with your credentials"
    exit 1
fi
echo "✓ .env file exists"
echo ""

# Step 3: Build
echo "Step 3: Building Rust code..."
cargo build 2>&1 | tee build.log
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "❌ Build failed. Check build.log for details"
    exit 1
fi
echo "✓ Build successful"
echo ""

# Step 4: Test API endpoint with curl
echo "Step 4: Testing Kalshi API endpoint..."
source .env
API_URL="https://demo-api.kalshi.co/trade-api/v2/markets/${TICKER}/orderbook"
echo "Testing: $API_URL"
echo ""

HTTP_CODE=$(curl -s -o api_response.json -w "%{http_code}" \
    -H "Authorization: Bearer ${KALSHI_API_KEY}" \
    "$API_URL")

echo "HTTP Status: $HTTP_CODE"

if [ "$HTTP_CODE" = "200" ]; then
    echo "✓ API connection successful!"
    echo ""
    echo "Response preview:"
    cat api_response.json | head -20
    echo ""
elif [ "$HTTP_CODE" = "401" ]; then
    echo "❌ 401 Unauthorized - Check your API key"
    cat api_response.json
    exit 1
elif [ "$HTTP_CODE" = "404" ]; then
    echo "❌ 404 Not Found - Check ticker symbol: $TICKER"
    cat api_response.json
    exit 1
else
    echo "❌ Unexpected status: $HTTP_CODE"
    cat api_response.json
    exit 1
fi

# Step 5: Run the monitor
echo "Step 5: Running monitor (Ctrl+C to stop)..."
echo ""
cargo run 2>&1 | tee run.log
