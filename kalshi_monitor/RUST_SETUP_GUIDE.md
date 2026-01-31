# 🦀 Rust Setup Guide for Absolute Beginners

## Step 1: Install Rust (5 minutes)

### On macOS/Linux:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When prompted, press **Enter** to proceed with default installation.

After installation completes:
```bash
source $HOME/.cargo/env
```

### Verify Installation:
```bash
rustc --version
cargo --version
```

You should see version numbers like:
```
rustc 1.75.0
cargo 1.75.0
```

## Step 2: Understanding the Project Structure

```
kalshi_monitor/
├── Cargo.toml          ← Project configuration (like package.json or requirements.txt)
├── .env                ← Your API credentials (you'll create this)
├── .env.example        ← Template for .env
├── src/
│   └── main.rs         ← Your Rust code
└── README.md           ← Quick reference
```

## Step 3: Navigate to Project

```bash
cd /Users/zitingyu/poly/kalshi_monitor
```

## Step 4: Understanding Cargo.toml

`Cargo.toml` is like `package.json` (Node.js) or `requirements.txt` (Python). It lists your project's dependencies.

**Example:**
```toml
[package]
name = "kalshi_monitor"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }  ← Async runtime
serde = { version = "1", features = ["derive"] } ← JSON handling
```

The `features = ["full"]` part enables extra functionality.

## Step 5: Build the Project

```bash
cargo build
```

This will:
1. Download all dependencies (first time only - takes 2-5 minutes)
2. Compile your code
3. Create an executable in `target/debug/`

## Step 6: Create Your .env File

```bash
cp .env.example .env
nano .env  # or use any text editor
```

Add your credentials:
```
KALSHI_API_KEY=your_api_key_here
KALSHI_PRIVATE_KEY_PATH=/path/to/your/private_key.pem
TICKER=KXLPL-24FEB01-T1
```

Press `Ctrl+O` to save, `Ctrl+X` to exit.

## Step 7: Run the Monitor

```bash
cargo run
```

This compiles (if needed) and runs your program.

## Common Commands

```bash
# Build (compile) without running
cargo build

# Build optimized version (faster, but takes longer to compile)
cargo build --release

# Run the program
cargo run

# Check for errors without building
cargo check

# Clean build artifacts
cargo clean

# Update dependencies
cargo update
```

## Troubleshooting

### "cargo: command not found"
```bash
source $HOME/.cargo/env
```

### Compilation errors
Read the error message carefully - Rust's compiler is very helpful!

### WebSocket connection fails
- Check your .env file has correct credentials
- Verify you're using the demo environment URL

## Next Steps

Once the monitor is running, you'll see:
```
🦀 Kalshi Orderbook Monitor
Connected to: wss://demo-api.kalshi.co/trade-api/v2/stream
Monitoring: KXLPL-24FEB01-T1

[10:15:30] TOP OF BOOK
Best Bid: 52¢ (52.0%)
Best Ask: 54¢ (54.0%)
Spread: 2¢ (3.85%)
Fair Price: 53¢ (53.0%)
```

Press `Ctrl+C` to stop.

## Learning Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **Rust by Example**: https://doc.rust-lang.org/rust-by-example/
- **Cargo Book**: https://doc.rust-lang.org/cargo/
