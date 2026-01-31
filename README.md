# Poly - Market Monitors

Collection of market monitoring tools for prediction markets.

## Projects

### 1. Kalshi Monitor (Rust)
Fast REST API monitor with 0.5s polling.

```bash
cd kalshi_monitor
cargo run
```

**Features:**
- ⚡ 0.5 second updates
- 🦀 Rust performance
- 🔐 RSA-PSS authentication

### 2. Polymarket Monitor (Python)
CLOB market monitor for Polymarket.

```bash
cd polymarket_monitor
./run_monitor.sh
```

**Features:**
- 📊 Full orderbook display
- 🐍 Python implementation
- 🔄 Auto-reconnect

### 3. Kalshi Monitor (Python)
Alternative Python implementation for Kalshi.

```bash
python3 kalshi_monitor_rest.py
```

**Features:**
- 🐍 Simple Python
- 📡 REST API
- 🔐 RSA-PSS authentication

## Requirements

```bash
pip install -r requirements.txt
```

## Structure

```
poly/
├── kalshi_monitor/          # Rust Kalshi monitor
│   ├── src/main.rs
│   ├── find_lol_markets.py
│   └── switch_market.sh
├── polymarket_monitor/      # Python Polymarket monitor
│   ├── polymarket_clob_monitor.py
│   ├── run_monitor.sh
│   └── stop_monitor.sh
├── kalshi_monitor_rest.py   # Python Kalshi monitor (REST)
├── kalshi_monitor_ws.py     # Python Kalshi monitor (alt)
└── requirements.txt
```
