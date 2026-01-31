use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use colored::*;
use dotenv::dotenv;
use pkcs8::DecodePrivateKey;
use reqwest;
use rsa::{
    pkcs1::DecodeRsaPrivateKey,
    pss::{BlindedSigningKey, Signature},
    signature::{RandomizedSigner, SignatureEncoding},
    RsaPrivateKey,
};
use serde::Deserialize;
use sha2::Sha256;
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
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

/// Load RSA private key from PEM file
fn load_private_key(path: &str) -> Result<RsaPrivateKey> {
    let pem_content = fs::read_to_string(path)
        .context(format!("Failed to read private key from {}", path))?;
    
    // Try PKCS#1 format first (RSA PRIVATE KEY)
    if let Ok(private_key) = RsaPrivateKey::from_pkcs1_pem(&pem_content) {
        return Ok(private_key);
    }
    
    // Try PKCS#8 format (PRIVATE KEY)
    let private_key = RsaPrivateKey::from_pkcs8_pem(&pem_content)
        .context("Failed to parse private key (tried both PKCS#1 and PKCS#8 formats)")?;
    
    Ok(private_key)
}

/// Generate RSA-PSS signature for API authentication (matches Kalshi Python implementation)
fn generate_signature(private_key: &RsaPrivateKey, timestamp: u128, method: &str, path: &str) -> Result<String> {
    // Create message: timestamp + method + path
    let message = format!("{}{}{}", timestamp, method, path);
    
    // Create PSS signing key with SHA256
    let mut rng = rand::thread_rng();
    let signing_key = BlindedSigningKey::<Sha256>::new(private_key.clone());
    
    // Sign the message
    let signature: Signature = signing_key.sign_with_rng(&mut rng, message.as_bytes());
    
    // Base64 encode
    let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());
    
    Ok(signature_b64)
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

/// Fetch market data from Kalshi REST API
async fn get_market_data(
    client: &reqwest::Client,
    api_key: &str,
    private_key: &RsaPrivateKey,
    ticker: &str,
) -> Result<Market> {
    // Generate timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();

    // Generate signature
    let path = format!("/trade-api/v2/markets/{}", ticker);
    let signature = generate_signature(private_key, timestamp, "GET", &path)?;

    // Make request
    let url = format!("https://api.elections.kalshi.com{}", path);
    
    let response = client
        .get(&url)
        .header("KALSHI-ACCESS-KEY", api_key)
        .header("KALSHI-ACCESS-TIMESTAMP", timestamp.to_string())
        .header("KALSHI-ACCESS-SIGNATURE", signature)
        .send()
        .await
        .context("Failed to send request")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("API error {}: {}", status, text));
    }

    let market_response: MarketResponse = response
        .json()
        .await
        .context("Failed to parse response")?;

    Ok(market_response.market)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    let api_key = env::var("KALSHI_API_KEY")
        .context("KALSHI_API_KEY not found in .env file")?;
    let private_key_path = env::var("KALSHI_PRIVATE_KEY_PATH")
        .context("KALSHI_PRIVATE_KEY_PATH not found in .env file")?;
    let ticker = env::var("TICKER")
        .context("TICKER not found in .env file")?;
    let poll_interval = env::var("POLL_INTERVAL")
        .unwrap_or_else(|_| "0.5".to_string())
        .parse::<f64>()
        .unwrap_or(0.5);

    println!("{}", "🦀 Kalshi Market Monitor (Rust REST API)".bold().cyan());
    println!("{}", "═".repeat(70).cyan());
    println!("Monitoring: {}", ticker.yellow());
    println!("Poll Interval: {}s", format!("{}", poll_interval).cyan());
    println!("{}", "═".repeat(70).cyan());
    println!("\n{}", "Loading private key...".yellow());

    // Load private key
    let private_key = load_private_key(&private_key_path)?;
    println!("{}", "✓ Private key loaded".green());

    // Create HTTP client
    let client = reqwest::Client::new();

    println!("\n{}", "Starting monitor...".yellow());

    loop {
        match get_market_data(&client, &api_key, &private_key, &ticker).await {
            Ok(market) => {
                display_market(&market);
            }
            Err(e) => {
                println!("{}", format!("\n❌ Error: {}", e).red());
            }
        }

        sleep(Duration::from_secs_f64(poll_interval)).await;
    }
}
