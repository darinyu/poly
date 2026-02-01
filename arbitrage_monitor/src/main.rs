mod kalshi;
mod polymarket;
mod arbitrage;

use anyhow::{Context, Result};
use colored::*;
use dotenv::dotenv;
use std::env;
use tokio::time::{sleep, Duration};

use kalshi::{KalshiClient, KalshiMarket};
use polymarket::{PolymarketClient, PolymarketMarket};
use arbitrage::{detect_arbitrage, ArbitrageOpportunity};

fn display_markets(target_team: &str, kalshi: &KalshiMarket, polymarket: &PolymarketMarket) {
    let now = chrono::Local::now().format("%H:%M:%S");
    
    // Header
    println!("\n{}", "═".repeat(100).cyan());
    println!("{}", format!("[{}] Market: {} (Outcome: {})", now, kalshi.title, target_team).bold());
    println!("{}", "─".repeat(100).cyan());
    
    // Column headers
    println!("{:<20} | {:<16} | {:<16} | {:<16} | {:<16}", 
        "Level", "Poly Price", "Poly Vol", "Kalshi Price", "Kalshi Vol");
    println!("{}", "─".repeat(100).dimmed());

    // Prepare Polymarket levels
    let p_bids = &polymarket.bids;
    let p_asks = &polymarket.asks;

    // Prepare Kalshi levels
    let mut k_bids = Vec::new();
    let mut k_asks = Vec::new();
    
    if let Some(ob) = &kalshi.orderbook {
        // Kalshi binary markets: 
        // Yes Bid = ob.yes.highest_price
        // Yes Ask = 100 - ob.no.highest_price
        
        k_bids = ob.yes.clone();
        k_bids.sort_by(|a, b| b.0.cmp(&a.0)); // Descending yes prices (bids)
        
        let mut k_asks_raw = ob.no.clone();
        k_asks_raw.sort_by(|a, b| b.0.cmp(&a.0)); // Descending no prices (bids for no) -> Ascending yes prices (asks for yes)
        k_asks = k_asks_raw;
    }

    // Show 5 levels of Asks (in reverse order, so best ask is closest to center)
    for i in (0..5).rev() {
        let p_ask = p_asks.get(i);
        let k_ask = k_asks.get(i);
        
        let p_price = p_ask.map(|a| format!("${:.4}", a.price.parse::<f64>().unwrap_or(0.0))).unwrap_or_default();
        let p_vol = p_ask.map(|a| format!("{:.0}", a.size.parse::<f64>().unwrap_or(0.0))).unwrap_or_default();
        
        let k_price = k_ask.map(|a| format!("${:.2}", (100 - a.0) as f64 / 100.0)).unwrap_or_default();
        let k_vol = k_ask.map(|a| format!("{}", a.1)).unwrap_or_default();

        println!("{:<20} | {:<16} | {:<16} | {:<16} | {:<16}", 
            format!("Ask #{}", i + 1).red(), p_price, p_vol, k_price, k_vol);
    }

    println!("{}", "─".repeat(100).dimmed());

    // Show 5 levels of Bids
    for i in 0..5 {
        let p_bid = p_bids.get(i);
        let k_bid = k_bids.get(i);
        
        let p_price = p_bid.map(|b| format!("${:.4}", b.price.parse::<f64>().unwrap_or(0.0))).unwrap_or_default();
        let p_vol = p_bid.map(|b| format!("{:.0}", b.size.parse::<f64>().unwrap_or(0.0))).unwrap_or_default();
        
        let k_price = k_bid.map(|b| format!("${:.2}", b.0 as f64 / 100.0)).unwrap_or_default();
        let k_vol = k_bid.map(|b| format!("{}", b.1)).unwrap_or_default();

        println!("{:<20} | {:<16} | {:<16} | {:<16} | {:<16}", 
            format!("Bid #{}", i + 1).green(), p_price, p_vol, k_price, k_vol);
    }

    println!("{}", "─".repeat(100).dimmed());

    // Summary / Spreads
    let best_p_bid = polymarket.best_bid;
    let best_p_ask = polymarket.best_ask;
    let best_k_bid = kalshi.yes_bid as f64 / 100.0;
    let best_k_ask = kalshi.yes_ask as f64 / 100.0;

    let p_to_k = best_k_bid - best_p_ask;
    let k_to_p = best_p_bid - best_k_ask;

    println!("{:<20} | {:<35} | {:<35}", 
        "Potential Arb",
        if p_to_k > 0.0 { format!("Poly -> Kalshi: +${:.4} 🚨", p_to_k).green().bold().to_string() } else { format!("Poly -> Kalshi: ${:.4}", p_to_k).dimmed().to_string() },
        if k_to_p > 0.0 { format!("Kalshi -> Poly: +${:.4} 🚨", k_to_p).green().bold().to_string() } else { format!("Kalshi -> Poly: ${:.4}", k_to_p).dimmed().to_string() }
    );

    println!("{}", "═".repeat(100).cyan());
}

fn display_arbitrage(opp: &ArbitrageOpportunity) {
    println!("\n{}", "🚨 ARBITRAGE OPPORTUNITY DETECTED! 🚨".green().bold());
    println!("{}", "═".repeat(70).yellow());
    println!("{}", format!("Buy on:  {} @ ${:.4}", opp.buy_platform, opp.buy_price).cyan());
    println!("{}", format!("Sell on: {} @ ${:.4}", opp.sell_platform, opp.sell_price).cyan());
    println!();
    println!("{}", format!("💰 Profit: {:.2}¢ ({:.2}%)", opp.profit_cents, opp.profit_pct).green().bold());
    println!("{}", "═".repeat(70).yellow());
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    let kalshi_api_key = env::var("KALSHI_API_KEY")
        .context("KALSHI_API_KEY not found in .env")?;
    let kalshi_key_path = env::var("KALSHI_PRIVATE_KEY_PATH")
        .context("KALSHI_PRIVATE_KEY_PATH not found in .env")?;
    
    let mut poll_interval = env::var("POLL_INTERVAL")
        .unwrap_or_else(|_| "0.5".to_string())
        .parse::<f64>()
        .unwrap_or(0.5);

    let verbose = env::var("VERBOSE")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";

    let debug = env::var("DEBUG")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";

    if debug {
        println!("{}", "⚠️  DEBUG MODE ENABLED - Slowing poll interval to 30.0s".yellow().bold());
        poll_interval = 30.0;
    }

    // Manual market configuration (can be event-level slugs/tickers)
    let polymarket_slug = env::var("POLYMARKET_SLUG")
        .context("POLYMARKET_SLUG not found in .env")?;
    let kalshi_input = env::var("KALSHI_TICKER")
        .context("KALSHI_TICKER not found in .env")?;
    
    let polymarket_ws_url = env::var("POLYMARKET_WS_URL")
        .unwrap_or_else(|_| "wss://ws-subscriptions-clob.polymarket.com/ws/market".to_string());

    if verbose || debug {
        println!("{}", "🔍 Arbitrage Monitor".bold().cyan());
        println!("{}", "═".repeat(70).cyan());
    }

    // Load Kalshi private key
    if verbose || debug {
        println!("\n{}", "Loading Kalshi credentials...".yellow());
    }
    let kalshi_private_key = kalshi::auth::load_private_key(&kalshi_key_path)?;
    if verbose || debug {
        println!("{}", "✓ Kalshi credentials loaded".green());
    }

    // Initialize Kalshi client
    let mut kalshi_client = KalshiClient::new(kalshi_api_key, kalshi_private_key);
    kalshi_client.set_debug(debug);

    // 1. Resolve Polymarket asset ID and anchor from slug
    if verbose || debug {
        println!("\n{}", "Resolving Polymarket market...".yellow());
    }
    let (polymarket_asset_id, anchor) = match polymarket::get_asset_id_and_anchor(&polymarket_slug, verbose, debug).await {
        Ok((asset_id, anchor)) => {
            if verbose || debug {
                println!("{}", format!("✓ Resolved Polymarket asset: {}...", &asset_id[..20]).green());
                println!("{}", format!("✓ Extracted anchor team:    {}", anchor).green());
            }
            (asset_id, anchor)
        },
        Err(e) => {
            println!("{}", format!("❌ Failed to resolve Polymarket market: {}", e).red());
            println!("{}", "Hint: Check POLYMARKET_SLUG in .env".yellow());
            return Ok(());
        }
    };

    if verbose || debug {
        println!("\n{}", "Resolving Kalshi market...".yellow());
    }
    let kalshi_ticker = match kalshi_client.resolve_market_ticker(&kalshi_input, Some(&anchor)).await {
        Ok(ticker) => {
            if verbose || debug {
                println!("{}", format!("✓ Resolved Kalshi ticker: {}", ticker).green());
            }
            ticker
        },
        Err(e) => {
            println!("{}", format!("❌ Failed to resolve Kalshi market: {}", e).red());
            println!("{}", "Hint: Check KALSHI_TICKER in .env".yellow());
            return Ok(());
        }
    };

    if verbose || debug {
        println!("\n{}", "═".repeat(70).cyan());
        println!("{}", "Configuration:".bold().green());
        println!("  Anchor Team:      {}", anchor.cyan());
        println!("  Polymarket Slug:  {}", polymarket_slug.cyan());
        println!("  Polymarket Asset: {}...", &polymarket_asset_id[..20].cyan());
        println!("  Kalshi Input:     {}", kalshi_input.cyan());
        println!("  Kalshi Ticker:    {}", kalshi_ticker.cyan());
        println!("  Poll Interval:    {}s", format!("{}", poll_interval).cyan());
        println!("  Verbose Mode:     {}", if verbose { "ON".green() } else { "OFF".dimmed() });
        println!("  Debug Mode:       {}", if debug { "ON".yellow().bold() } else { "OFF".dimmed() });
        println!("{}", "═".repeat(70).cyan());
    }
    
    let mut polymarket_client = PolymarketClient::new(polymarket_ws_url, polymarket_asset_id.clone());
    polymarket_client.set_debug(debug);

    // Track latest market data
    let mut last_kalshi_market: Option<KalshiMarket> = None;
    let mut last_polymarket_market: Option<PolymarketMarket> = None;

    if verbose || debug {
        println!("\n{}", "Starting arbitrage detection...".yellow());
        
        // Connect to Polymarket WebSocket once
        println!("{}", "Connecting to Polymarket WebSocket...".dimmed());
    }
    match polymarket_client.connect().await {
        Ok(_) => {
            if verbose || debug {
                println!("{}", "✓ Polymarket connected".green());
            }
        },
        Err(e) => {
            println!("{}", format!("❌ Failed to connect to Polymarket: {}", e).red());
            println!("{}", "Hint: Check POLYMARKET_ASSET_ID in .env".yellow());
            return Ok(());
        }
    }
    
    if verbose || debug {
        println!("{}", "Press Ctrl+C to stop\n".dimmed());
    }

    // Track last Kalshi fetch time
    let mut last_kalshi_fetch = std::time::Instant::now();
    let kalshi_interval = std::time::Duration::from_secs_f64(poll_interval);

    // Throttling for non-verbose mode
    let mut iter_count = 0;
    let mut last_display_time = std::time::Instant::now();
    let display_interval = std::time::Duration::from_secs(10);

    loop {
        // Fetch Kalshi data if interval elapsed
        let kalshi_future = async {
            if last_kalshi_fetch.elapsed() >= kalshi_interval {
                last_kalshi_fetch = std::time::Instant::now();
                
                match kalshi_client.get_market(&kalshi_ticker).await {
                    Ok(market) => Some(market),
                    Err(e) => {
                        println!("{}", format!("\n❌ Kalshi error: {}", e).red());
                        None
                    }
                }
            } else {
                None
            }
        };

        // Read next Polymarket update (non-blocking with timeout)
        let polymarket_future = async {
            match tokio::time::timeout(
                std::time::Duration::from_millis(500),
                polymarket_client.read_next_book()
            ).await {
                Ok(Ok(market)) => Some(market),
                Ok(Err(e)) => {
                    println!("{}", format!("\n❌ Polymarket error: {}", e).red());
                    println!("{}", "Reconnecting...".yellow());
                    
                    // Try to reconnect
                    if let Err(e) = polymarket_client.connect().await {
                        println!("{}", format!("❌ Reconnect failed: {}", e).red());
                    }
                    None
                },
                Err(_) => None, // Timeout - no new data
            }
        };

        // Run both concurrently
        let (kalshi_result, polymarket_result) = tokio::join!(kalshi_future, polymarket_future);

        // Process updates
        if let Some(market) = kalshi_result {
            last_kalshi_market = Some(market);
        }
        if let Some(market) = polymarket_result {
            last_polymarket_market = Some(market);
        }

        // Display and detect if we have at least one side
        if let (Some(k_market), Some(p_market)) = (&last_kalshi_market, &last_polymarket_market) {
            let should_display = if verbose {
                true 
            } else {
                iter_count >= 10 || last_display_time.elapsed() >= display_interval
            };

            if should_display {
                display_markets(&anchor, k_market, p_market);
                iter_count = 0;
                last_display_time = std::time::Instant::now();
            } else {
                iter_count += 1;
            }

            // Detect arbitrage - ALWAYS display if found
            if let Some(opportunity) = detect_arbitrage(k_market, p_market) {
                if !should_display {
                    display_markets(&anchor, k_market, p_market);
                }
                display_arbitrage(&opportunity);
            }
        }

        // Small sleep to prevent busy loop
        sleep(Duration::from_millis(10)).await;
    }
}
