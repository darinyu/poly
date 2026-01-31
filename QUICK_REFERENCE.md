# Polymarket CLOB Monitor - Quick Reference

## 🚀 Quick Start (3 Steps)

```bash
# 1. Setup
./setup.sh && source venv/bin/activate

# 2. Find asset IDs
python fetch_asset_ids.py LPL

# 3. Run monitor
python polymarket_clob_monitor.py
```

## 📁 Files Overview

| File | Purpose |
|------|---------|
| `polymarket_clob_monitor.py` | Main WebSocket monitor (production-ready) |
| `fetch_asset_ids.py` | Helper to find asset IDs from Gamma API |
| `demo.py` | Simulated output demo (no connection needed) |
| `strategy_example.py` | Example trading strategy integrations |
| `setup.sh` | Automated setup script |
| `requirements.txt` | Python dependencies |
| `README.md` | Full documentation |

## 🎯 Common Commands

```bash
# Search for LPL matches
python fetch_asset_ids.py LPL

# Search for specific teams
python fetch_asset_ids.py "JDG vs AL"

# Run demo (no real connection)
python demo.py

# Run real monitor
python polymarket_clob_monitor.py

# Run custom strategy
python strategy_example.py
```

## 📊 Output Types

### Trade
```
[TRADE] Asset: XXX | Side: BUY/SELL | Price: 0.5234 | Size: 100.00
```

### Book
```
[BOOK] Asset: XXX | Best Bid: 0.5200 | Best Ask: 0.5250 | Spread: 0.0050 (96.2 bps)
```

### Volatility Alert
```
🚨 VOLATILITY ALERT: PRICE SPIKE UP: 2.34% | VOLUME SPIKE: 4.2x baseline
```

## 🔧 Configuration

Edit these constants in `polymarket_clob_monitor.py`:

```python
PING_INTERVAL = 30      # Heartbeat interval
PONG_TIMEOUT = 10       # Pong wait time
window_seconds = 5      # Volatility window
price_threshold = 0.02  # Price alert (2%)
volume_multiplier = 3.0 # Volume alert (3x)
```

## 🌐 API Endpoints

- **WebSocket**: `wss://ws-subscriptions-clob.polymarket.com/ws/market`
- **Gamma API**: `https://gamma-api.polymarket.com/markets`

## 🎨 Color Codes

- 🟢 **Green**: BUY trades, Best Bid
- 🔴 **Red**: SELL trades, Best Ask, Alerts
- 🔵 **Cyan**: Prices, Info messages
- 🟣 **Magenta**: Size, Arbitrage
- 🟡 **Yellow**: Spread, Heartbeat

## 🛠️ Troubleshooting

| Issue | Solution |
|-------|----------|
| `pip: command not found` | Use `pip3` or run `./setup.sh` |
| No data received | Verify asset IDs with `fetch_asset_ids.py` |
| Connection drops | Auto-reconnects with exponential backoff |
| Invalid asset ID | Market may be closed or ID incorrect |

## 📈 Strategy Ideas

1. **Arbitrage**: Monitor both outcomes, detect probability sum ≠ 1.0
2. **Volume Imbalance**: Track buy/sell ratio for directional signals
3. **Spread Trading**: Profit from wide bid-ask spreads
4. **News Trading**: React to volatility alerts faster than others

## 🔒 Security

- ✅ Public endpoints (no auth required)
- ✅ Read-only access
- ✅ No API keys needed
- ❌ Cannot place orders (monitor only)

## 📚 Resources

- [Polymarket Docs](https://docs.polymarket.com/)
- [WebSocket Protocol](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
- [Gamma API](https://gamma-api.polymarket.com/)

---

**Need Help?** Check the full [README.md](file:///Users/zitingyu/poly/README.md) or run `python demo.py` to see it in action!
