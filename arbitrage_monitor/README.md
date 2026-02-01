# Arbitrage Monitor

Cross-platform arbitrage detector for Kalshi and Polymarket.

## Quick Start

1. **Configure `.env`:**
```bash
cp .env.example .env
# Edit .env with your credentials
```

2. **Build and run:**
```bash
cargo build
cargo run
```

## Configuration

`.env` file:
```env
# Kalshi
KALSHI_API_KEY=your_key
KALSHI_PRIVATE_KEY_PATH=../kalshi_monitor/key.pem
KALSHI_TICKER=KXLOLGAME-26FEB02G2VIT-G2

# Polymarket
POLYMARKET_TOKEN_ID=your_token_id
POLYMARKET_WS_URL=wss://ws-subscriptions-clob.polymarket.com/ws/market

# Settings
POLL_INTERVAL=0.5
VERBOSE=false  # Set to 'true' to show market data every poll
```

### Verbose Mode

- **`VERBOSE=false` (default):** Only shows output when arbitrage is detected
- **`VERBOSE=true`:** Shows market data every poll interval (0.5s)

**Recommended:** Keep verbose OFF for cleaner output, only see alerts when opportunities exist.

## Features

- ⚡ Real-time monitoring (0.5s updates)
- 🦀 Rust performance
- 📊 Kalshi REST API
- 📡 Polymarket WebSocket
- 💰 Automatic arbitrage detection
- 📈 Side-by-side orderbook display

## Output

```
🔍 Arbitrage Monitor
══════════════════════════════════════════════════════════════════════
[16:05:30] Will G2 Esports win the G2 Esports vs. Team Vitality League of Legends match?
──────────────────────────────────────────────────────────────────────
Kalshi:
  Bid: 59¢ (59.0%)
  Ask: 63¢ (63.0%)
  Spread: 4¢

Polymarket:
  Bid: $0.5800 (58.0%)
  Ask: $0.6200 (62.0%)
  Spread: $0.0400
══════════════════════════════════════════════════════════════════════

🚨 ARBITRAGE OPPORTUNITY DETECTED! 🚨
══════════════════════════════════════════════════════════════════════
Buy on:  Polymarket @ $0.6200
Sell on: Kalshi @ $0.5900

💰 Profit: -3.00¢ (-4.84%)
══════════════════════════════════════════════════════════════════════
```

## Project Structure

```
src/
├── main.rs              # Main application
├── arbitrage.rs         # Arbitrage detection logic
├── kalshi/
│   ├── mod.rs          # Module exports
│   ├── auth.rs         # RSA authentication
│   └── client.rs       # REST API client
└── polymarket/
    ├── mod.rs          # Module exports
    └── websocket.rs    # WebSocket client
```
