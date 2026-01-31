#!/bin/bash
# Find available Kalshi LoL markets

# Load API key from .env
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

API_KEY="${KALSHI_API_KEY}"

if [ -z "$API_KEY" ]; then
    echo "❌ KALSHI_API_KEY not found in .env"
    exit 1
fi

echo "🔍 Searching for League of Legends markets on Kalshi..."
echo ""

curl -s \
  -H "Authorization: Bearer $API_KEY" \
  "https://api.elections.kalshi.com/trade-api/v2/markets?limit=50&series_ticker=KXLOLGAME" | \
python3 << 'EOF'
import sys, json
from datetime import datetime

data = json.load(sys.stdin)
markets = data.get('markets', [])

if not markets:
    print("❌ No LoL markets found")
    sys.exit(1)

print(f"Found {len(markets)} LoL markets:\n")
print("=" * 100)

# Sort by expected expiration time
markets_sorted = sorted(markets, key=lambda x: x.get('expected_expiration_time', ''))

for i, m in enumerate(markets_sorted, 1):
    ticker = m['ticker']
    title = m['title']
    status = m['status']
    
    # Get bid/ask
    yes_bid = m.get('yes_bid', 0)
    yes_ask = m.get('yes_ask', 0)
    spread = yes_ask - yes_bid if yes_bid and yes_ask else 0
    
    # Get volume
    volume_24h = m.get('volume_24h', 0)
    open_interest = m.get('open_interest', 0)
    
    # Parse expiration time
    exp_time = m.get('expected_expiration_time', 'Unknown')
    try:
        dt = datetime.fromisoformat(exp_time.replace('Z', '+00:00'))
        exp_str = dt.strftime('%b %d, %Y %H:%M UTC')
    except:
        exp_str = exp_time
    
    # Status indicator
    status_icon = "🟢" if status == "active" else "🔴"
    
    print(f"{i}. {status_icon} {title}")
    print(f"   Ticker: {ticker}")
    print(f"   Expires: {exp_str}")
    print(f"   Bid/Ask: {yes_bid}¢ / {yes_ask}¢ (Spread: {spread}¢)")
    print(f"   Volume: {volume_24h} | Open Interest: {open_interest}")
    print("-" * 100)

print("\n💡 To monitor a market, update your .env file:")
print("   TICKER=<ticker_from_above>")
print("\nOr run: echo 'TICKER=KXLOLGAME-...' >> .env")
EOF
