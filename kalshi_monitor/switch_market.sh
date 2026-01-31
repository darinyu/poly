#!/bin/bash
# Quick script to switch to a different market

if [ -z "$1" ]; then
    echo "Usage: ./switch_market.sh <TICKER>"
    echo ""
    echo "Example: ./switch_market.sh KXLOLGAME-26FEB02G2VIT-G2"
    echo ""
    echo "💡 Run ./find_lol_markets.sh to see available markets"
    exit 1
fi

TICKER=$1

# Update .env file
if [ -f .env ]; then
    # Use sed to replace the TICKER line
    if grep -q "^TICKER=" .env; then
        # macOS compatible sed
        sed -i '' "s|^TICKER=.*|TICKER=$TICKER|" .env
        echo "✅ Updated .env with ticker: $TICKER"
    else
        echo "TICKER=$TICKER" >> .env
        echo "✅ Added ticker to .env: $TICKER"
    fi
else
    echo "❌ .env file not found"
    exit 1
fi

echo ""
echo "🦀 Rebuild and run the monitor:"
echo "   ./run.sh"
