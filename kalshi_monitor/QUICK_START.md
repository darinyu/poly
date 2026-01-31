# 🦀 Kalshi LoL Monitor - Quick Reference

## Finding Markets

**List all available LoL markets:**
```bash
python3 find_lol_markets.py
```

This shows:
- Market title and ticker
- Current bid/ask prices and spread
- 24h volume and open interest
- Expiration time

## Switching Markets

**Quick switch to a different market:**
```bash
./switch_market.sh KXLOLGAME-26FEB02G2VIT-G2
```

## Running the Monitor

**Build and run:**
```bash
./run.sh
```

**Or manually:**
```bash
cargo build
cargo run
```

## Useful Commands

```bash
# Find LoL markets
python3 find_lol_markets.py

# Switch to a market
./switch_market.sh <TICKER>

# Run monitor
./run.sh

# Stop monitor
Ctrl+C
```

## Files

- `find_lol_markets.py` - List available LoL markets
- `switch_market.sh` - Quickly switch markets
- `run.sh` - Build and run the monitor
- `.env` - Your API key and current ticker
- `src/main.rs` - Main Rust code

## Example Workflow

```bash
# 1. Find markets
python3 find_lol_markets.py

# 2. Pick a market and switch to it
./switch_market.sh KXLOLGAME-26FEB02LRNAVI-NAVI

# 3. Run the monitor
./run.sh
```

That's it! 🚀
