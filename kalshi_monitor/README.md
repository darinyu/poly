# 🦀 Kalshi Orderbook Monitor (Rust)

High-performance WebSocket monitor for Kalshi's orderbook with real-time spread and fair price calculations.

## Quick Start

### 1. Install Rust (if not already installed)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Set Up Credentials
```bash
cp .env.example .env
nano .env  # Add your Kalshi API credentials
```

### 3. Build and Run
```bash
cargo build
cargo run
```

## Configuration (.env file)

```env
KALSHI_API_KEY=your_api_key_here
KALSHI_PRIVATE_KEY_PATH=/path/to/private_key.pem
TICKER=KXLPL-24FEB01-T1
WS_URL=wss://demo-api.kalshi.co/trade-api/ws/v2
```

## Example Output

```
🦀 Kalshi Orderbook Monitor
══════════════════════════════════════════════════════════════════════
Monitoring: KXLPL-24FEB01-T1
WebSocket: wss://demo-api.kalshi.co/trade-api/ws/v2
══════════════════════════════════════════════════════════════════════

══════════════════════════════════════════════════════════════════════
[11:15:30] TOP OF BOOK
──────────────────────────────────────────────────────────────────────
Best Bid: 52¢ (52.0%) | Qty: 150
Best Ask: 54¢ (54.0%) | Qty: 200
Spread:   2¢ (3.77%)
Fair:     53.0¢ (53.0%)
══════════════════════════════════════════════════════════════════════
```

## Features

✅ WebSocket connection to Kalshi demo environment  
✅ Real-time orderbook updates  
✅ Top-of-book display (best bid/ask)  
✅ Spread calculation (cents and percentage)  
✅ Fair price calculation (mid-point)  
✅ No-vig probability (cents → percentage)  
✅ Auto-reconnection on connection drop  
✅ Color-coded terminal output  

## Commands

```bash
# Build (compile)
cargo build

# Build optimized version
cargo build --release

# Run
cargo run

# Run optimized version
cargo run --release

# Check for errors without building
cargo check

# Clean build artifacts
cargo clean
```

## Troubleshooting

### "cargo: command not found"
```bash
source $HOME/.cargo/env
```

### WebSocket connection fails
- Verify .env file has correct credentials
- Check ticker symbol is valid
- Ensure using demo environment URL

### Compilation errors
Read the error message - Rust's compiler is very helpful and will guide you!

## For Absolute Beginners

See [RUST_SETUP_GUIDE.md](RUST_SETUP_GUIDE.md) for a complete step-by-step tutorial.

## Project Structure

```
kalshi_monitor/
├── Cargo.toml              # Dependencies and project config
├── .env                    # Your credentials (gitignored)
├── .env.example            # Template
├── src/
│   └── main.rs             # Main application code
├── RUST_SETUP_GUIDE.md     # Beginner's guide
└── README.md               # This file
```

## Learning Rust

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
