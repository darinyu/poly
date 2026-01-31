use anyhow::{Context, Result};
use colored::*;
use dotenv::dotenv;
use serde::Deserialize;
use std::env;
use tokio::time::{sleep, Duration};

/// Market data from Kalshi API
#[derive(Debug, Deserialize)]
struct MarketResponse {
    market: Market,
}

#[derive(Debug, Deserialize)]
struct Market {
    ticker: String,
    title: String,
    yes_bid: i32,
    yes_ask: i32,
    no_bid: i32,
    no_ask: i32,
    last_price: i32,
    volume_24h: i32,
    open_interest: i32,
}

/// Calculate fair price (mid-point) from best bid and ask
fn calculate_fair_price(best_bid: i32, best_ask: i32) -> f64 {
    (best_bid as f64 + best_ask as f64) / 2.0
}

/// Calculate spread
fn calculate_spread(best_bid: i32, best_ask: i32) -> i32 {
    best_ask - best_bid
}

/// Calculate spread as percentage
fn calculate_spread_percentage(best_bid: i32, best_ask: i32) -> f64 {
    if best_bid == 0 {
        return 0.0;
    }
    let spread = calculate_spread(best_bid, best_ask) as f64;
    let mid = calculate_fair_price(best_bid, best_ask);
    if mid == 0.0 {
        return 0.0;
    }
    (spread / mid) * 100.0
}

/// Convert cents to probability percentage
fn cents_to_probability(cents: i32) -> f64 {
    cents as f64
}

/// Display market data
fn display_market(market: &Market) {
    println!("\n{}", "═".repeat(70).cyan());
    println!("{}", format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), market.title).bold());
    println!("{}", format!("Ticker: {}", market.ticker).yellow());
    println!("{}", "─".repeat(70).cyan());
    
    // Check if market has active orders
    if market.yes_bid == 0 && market.yes_ask == 0 {
        println!("{}", "⚠️  No active orders in market".yellow());
        println!("{}", "═".repeat(70).cyan());
        return;
    }

    let best_bid = market.yes_bid;
    let best_ask = market.yes_ask;

    let fair_price = calculate_fair_price(best_bid, best_ask);
    let spread = calculate_spread(best_bid, best_ask);
    let spread_pct = calculate_spread_percentage(best_bid, best_ask);

    let bid_prob = cents_to_probability(best_bid);
    let ask_prob = cents_to_probability(best_ask);
    let fair_prob = fair_price;
    
    println!(
        "{} {}¢ ({:.1}%)",
        "Best Bid (Yes):".green().bold(),
        best_bid,
        bid_prob
    );
    
    println!(
        "{} {}¢ ({:.1}%)",
        "Best Ask (Yes):".red().bold(),
        best_ask,
        ask_prob
    );
    
    println!(
        "{} {}¢ ({:.2}%)",
        "Spread:        ".yellow().bold(),
        spread,
        spread_pct
    );
    
    println!(
        "{} {:.1}¢ ({:.1}%)",
        "Fair Price:    ".cyan().bold(),
        fair_price,
        fair_prob
    );

    println!("{}", "─".repeat(70).cyan());
    
    println!(
        "{} {}¢",
        "Last Price:    ".white(),
        market.last_price
    );
    
    println!(
        "{} {}",
        "24h Volume:    ".white(),
        market.volume_24h
    );
    
    println!(
        "{} {}",
        "Open Interest: ".white(),
        market.open_interest
    );
    
    println!("{}", "═".repeat(70).cyan());
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    let api_key = env::var("KALSHI_API_KEY")
        .context("KALSHI_API_KEY not found in .env file")?;
    let ticker = env::var("TICKER")
        .context("TICKER not found in .env file")?;
    let poll_interval = env::var("POLL_INTERVAL")
        .unwrap_or_else(|_| "3".to_string())
        .parse::<u64>()
        .unwrap_or(3);

    println!("{}", "🦀 Kalshi Market Monitor (REST API)".bold().cyan());
    println!("{}", "═".repeat(70).cyan());
    println!("Monitoring: {}", ticker.yellow());
    println!("Poll Interval: {}s", poll_interval.to_string().blue());
    println!("{}", "═".repeat(70).cyan());
    println!("\n{}", "Starting monitor...".yellow());

    // Create HTTP client
    let client = reqwest::Client::new();
    let api_url = format!("https://api.elections.kalshi.com/trade-api/v2/markets/{}", ticker);

    loop {
        match fetch_market(&client, &api_url, &api_key).await {
            Ok(market) => {
                display_market(&market);
            }
            Err(e) => {
                println!("{}", format!("\n❌ Error: {}", e).red());
                println!("{}", format!("   Details: {:?}", e).red());
            }
        }

        sleep(Duration::from_secs(poll_interval)).await;
    }
}

async fn fetch_market(client: &reqwest::Client, url: &str, api_key: &str) -> Result<Market> {
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .context("Failed to send request")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
        return Err(anyhow::anyhow!("API error {}: {}", status, body));
    }

    let market_response: MarketResponse = response
        .json()
        .await
        .context("Failed to parse JSON response")?;

    Ok(market_response.market)
}
