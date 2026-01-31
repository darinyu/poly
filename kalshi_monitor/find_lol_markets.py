#!/usr/bin/env python3
"""
Find available Kalshi LoL markets
"""

import os
import requests
from datetime import datetime

# Read API key from .env file
API_KEY = None
if os.path.exists('.env'):
    with open('.env') as f:
        for line in f:
            if line.startswith('KALSHI_API_KEY='):
                API_KEY = line.split('=', 1)[1].strip()
                break

if not API_KEY:
    print("❌ KALSHI_API_KEY not found in .env")
    exit(1)

print("🔍 Searching for League of Legends markets on Kalshi...\n")

# Fetch markets
url = "https://api.elections.kalshi.com/trade-api/v2/markets"
params = {
    'limit': 50,
    'series_ticker': 'KXLOLGAME'
}
headers = {
    'Authorization': f'Bearer {API_KEY}'
}

try:
    response = requests.get(url, params=params, headers=headers)
    response.raise_for_status()
    data = response.json()
except Exception as e:
    print(f"❌ Error fetching markets: {e}")
    exit(1)

markets = data.get('markets', [])

if not markets:
    print("❌ No LoL markets found")
    exit(1)

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

print("\n💡 To monitor a market:")
print("   ./switch_market.sh <TICKER>")
print("\nExample:")
print(f"   ./switch_market.sh {markets_sorted[0]['ticker']}")
