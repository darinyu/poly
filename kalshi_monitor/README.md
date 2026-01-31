# Kalshi Monitor (Rust)

Fast Kalshi market monitor using REST API with 0.5s polling.

## Quick Start

```bash
cargo build
cargo run
```

## Configuration

Edit `.env`:
```env
KALSHI_API_KEY=your_key
KALSHI_PRIVATE_KEY_PATH=./key.pem
TICKER=KXLOLGAME-26FEB02G2VIT-G2
POLL_INTERVAL=0.5
```

## Finding Markets

```bash
python3 find_lol_markets.py
```

## Switching Markets

```bash
./switch_market.sh <TICKER>
```

## Files

- `src/main.rs` - Rust monitor (REST API)
- `find_lol_markets.py` - List available LoL markets
- `switch_market.sh` - Quick market switcher
- `.env` - Configuration
- `key.pem` - Your private key
