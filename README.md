# Polymarket CLOB Real-Time Monitor

A production-ready Python WebSocket client for monitoring Polymarket's Central Limit Order Book (CLOB) to detect latency arbitrage opportunities during live sports/esports matches (specifically LPL matches).

## Features

✅ **Real-time WebSocket Connection** to Polymarket CLOB  
✅ **Trade Monitoring** - Track all matched orders with price, size, and side  
✅ **Order Book Updates** - Monitor best bid/ask with spread calculation  
✅ **Volatility Alerts** - Automatic detection of price spikes (>2%) and volume surges (3x)  
✅ **Heartbeat Mechanism** - Ping/Pong to maintain connection health  
✅ **Auto-Reconnection** - Exponential backoff retry logic  
✅ **Color-Coded Terminal Output** - Easy-to-read real-time data  
✅ **Multi-Asset Support** - Monitor multiple markets simultaneously  

## Installation

```bash
# Install dependencies
pip install -r requirements.txt
```

## Quick Start

### Step 1: Find Asset IDs for LPL Matches

Use the helper script to search for markets:

```bash
# Search for LPL matches
python fetch_asset_ids.py LPL

# Search for specific teams
python fetch_asset_ids.py "JDG vs AL"

# Browse all recent markets
python fetch_asset_ids.py
```

The script will display:
- Market questions and descriptions
- Token outcomes (e.g., "JDG Wins", "AL Wins")
- Asset IDs (token_id) for each outcome
- Copy-paste ready lists

### Step 2: Run the Monitor

```bash
python polymarket_clob_monitor.py
```

When prompted, enter the asset IDs (comma-separated):
```
> 21742633143463906290569050155826241533067272736897614950488156847949938836455,12345...
```

Or press Enter to use demo mode.

## Output Format

### Trade Events
```
[2026-01-30 22:06:40.123][TRADE] Asset: 21742... | Side: BUY | Price: 0.5234 | Size: 100.00
```

### Order Book Updates
```
[2026-01-30 22:06:40.456][BOOK] Asset: 21742... | Best Bid: 0.5200 | Best Ask: 0.5250 | Spread: 0.0050 (96.2 bps)
```

### Volatility Alerts
```
🚨 VOLATILITY ALERT [21742...]: PRICE SPIKE UP: 2.34% change | VOLUME SPIKE: 4.2x baseline
```

## Architecture

### Core Components

1. **PolymarketCLOBMonitor** - Main WebSocket client
   - Manages connection lifecycle
   - Handles subscriptions
   - Routes messages to handlers

2. **VolatilityMonitor** - Real-time analysis engine
   - Tracks price movements in 5-second windows
   - Detects volume anomalies
   - Triggers alerts for arbitrage opportunities

3. **Heartbeat System** - Connection health
   - Sends ping every 30 seconds
   - Monitors pong responses
   - Auto-reconnects on failure

### WebSocket Protocol

**Endpoint**: `wss://ws-subscriptions-clob.polymarket.com/ws/market`

**Subscription Message**:
```json
{
  "auth": {},
  "markets": ["<asset_id>"],
  "assets_ids": ["<asset_id>"],
  "type": "subscribe"
}
```

**Event Types**:
- `trade` - Matched order execution
- `book` - Order book snapshot/update
- `pong` - Heartbeat response

## Configuration

Edit the monitor class constants for custom behavior:

```python
PING_INTERVAL = 30      # Heartbeat interval (seconds)
PONG_TIMEOUT = 10       # Max wait for pong (seconds)
window_seconds = 5      # Volatility window (seconds)
price_threshold = 0.02  # Price alert threshold (2%)
volume_multiplier = 3.0 # Volume spike threshold (3x)
```

## API Reference

### Gamma Markets API

**Base URL**: `https://gamma-api.polymarket.com`

**Get Markets**:
```
GET /markets?limit=100&slug=<market-slug>
```

**Response Structure**:
```json
{
  "question": "Will JDG beat AL?",
  "slug": "jdg-vs-al-2026-01-30",
  "tokens": [
    {
      "outcome": "JDG Wins",
      "token_id": "21742633143463906290569050155826241533067272736897614950488156847949938836455",
      "price": "0.52"
    }
  ]
}
```

## Use Cases

### Latency Arbitrage Detection

Monitor multiple outcomes simultaneously to detect:
- **Price discrepancies** between correlated markets
- **Delayed updates** creating temporary mispricings
- **Volume surges** indicating insider information or breaking news

### Live Trading Strategy

1. Subscribe to all outcomes for a match (e.g., JDG Win, AL Win)
2. Monitor for volatility alerts
3. Calculate implied probabilities from best bid/ask
4. Execute trades when spreads widen or prices diverge

### Market Making

Track order book depth and spread to:
- Identify liquidity gaps
- Optimize quote placement
- Manage inventory risk

## Troubleshooting

### Connection Issues

If you see repeated reconnection attempts:
- Check your internet connection
- Verify the WebSocket URL is correct
- Ensure no firewall is blocking WebSocket connections

### No Data Received

If connected but no trades/books appear:
- Verify asset IDs are correct (use `fetch_asset_ids.py`)
- Check if the market is active (not closed)
- Ensure the market has trading activity

### Invalid Asset IDs

Error: "Subscription failed"
- Asset IDs must be exact token_id from Gamma API
- Use the helper script to get current IDs
- Markets may expire or close

## Advanced Usage

### Programmatic Integration

```python
from polymarket_clob_monitor import PolymarketCLOBMonitor

# Create monitor
asset_ids = ["21742633143463906290569050155826241533067272736897614950488156847949938836455"]
monitor = PolymarketCLOBMonitor(asset_ids)

# Start monitoring
await monitor.start()
```

### Custom Event Handlers

Extend the monitor class to add custom logic:

```python
class CustomMonitor(PolymarketCLOBMonitor):
    def _handle_trade(self, data: dict):
        super()._handle_trade(data)
        # Add your custom logic here
        # e.g., send to database, trigger alerts, etc.
```

## Performance Notes

- **Latency**: Typical WebSocket latency is 50-200ms
- **Throughput**: Can handle 100+ messages/second
- **Memory**: Volatility monitor keeps 5 seconds of trade history per asset
- **CPU**: Minimal overhead, suitable for running on low-end hardware

## Security Considerations

- This monitor uses **public, unauthenticated** WebSocket endpoints
- No API keys or credentials required
- Read-only access to market data
- Cannot place orders or access private data

## License

This is a development tool for educational and research purposes.

## Support

For issues or questions:
1. Check the Polymarket documentation
2. Verify asset IDs using the helper script
3. Review WebSocket connection logs

---

**Happy Trading! 🚀📈**
