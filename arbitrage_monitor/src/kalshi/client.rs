use anyhow::{Context, Result};
use colored::Colorize;
use reqwest;
use rsa::RsaPrivateKey;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

use super::auth::{generate_signature};

/// Kalshi market data
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct KalshiMarket {
    pub ticker: String,
    pub title: String,
    pub yes_bid: i32,
    pub yes_ask: i32,
    pub no_bid: i32,
    pub no_ask: i32,
    pub last_price: i32,
    pub volume_24h: i32,
    pub open_interest: i32,
    pub orderbook: Option<KalshiOrderbook>,
}



#[derive(Debug, Deserialize, Clone)]
pub struct KalshiOrderbook {
    pub yes: Vec<(i32, i32)>, // [price, size]
    pub no: Vec<(i32, i32)>,  // [price, size]
}

#[derive(Debug, Deserialize)]
struct MarketResponse {
    market: KalshiMarket,
}

/// Kalshi REST API client
pub struct KalshiClient {
    client: reqwest::Client,
    api_key: String,
    private_key: RsaPrivateKey,
    base_url: String,
    debug: bool,
}

impl KalshiClient {
    pub fn new(api_key: String, private_key: RsaPrivateKey) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            private_key,
            base_url: "https://api.elections.kalshi.com".to_string(),
            debug: false,
        }
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Fetch market data for a given ticker
    pub async fn get_market(&self, ticker: &str) -> Result<KalshiMarket> {
        // Generate timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();

        // Generate signature
        let path = format!("/trade-api/v2/markets/{}", ticker);
        let signature = generate_signature(&self.private_key, timestamp, "GET", &path)?;

        // Make request
        let url = format!("{}{}", self.base_url, path);
        
        let response = self.client
            .get(&url)
            .header("KALSHI-ACCESS-KEY", &self.api_key)
            .header("KALSHI-ACCESS-TIMESTAMP", timestamp.to_string())
            .header("KALSHI-ACCESS-SIGNATURE", signature)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Kalshi API error {}: {}", status, text));
        }

        let market_response: MarketResponse = response
            .json()
            .await
            .context("Failed to parse Kalshi response")?;

        let mut market = market_response.market;
        
        // Fetch orderbook as well
        if let Ok(orderbook) = self.get_orderbook(ticker).await {
            market.orderbook = Some(orderbook);
        }

        Ok(market)
    }

    /// Fetch orderbook for a given ticker
    pub async fn get_orderbook(&self, ticker: &str) -> Result<KalshiOrderbook> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();

        let path = format!("/trade-api/v2/markets/{}/orderbook", ticker);
        let signature = generate_signature(&self.private_key, timestamp, "GET", &path)?;

        let url = format!("{}{}", self.base_url, path);
        
        let response = self.client
            .get(&url)
            .header("KALSHI-ACCESS-KEY", &self.api_key)
            .header("KALSHI-ACCESS-TIMESTAMP", timestamp.to_string())
            .header("KALSHI-ACCESS-SIGNATURE", signature)
            .send()
            .await
            .context("Failed to send orderbook request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Kalshi API error {}: {}", status, text));
        }

        let body = response.text().await.context("Failed to read Kalshi orderbook body")?;
        
        if self.debug {
            println!("\n{} {}", "[DEBUG] Kalshi Raw Orderbook:".yellow().bold(), body);
        }

        #[derive(Deserialize)]
        struct OrderbookResponse {
            orderbook: KalshiOrderbook,
        }

        let ob_response: OrderbookResponse = serde_json::from_str(&body)
            .context("Failed to parse Kalshi orderbook response")?;

        Ok(ob_response.orderbook)
    }



    /// Resolve an event ticker to a specific market ticker
    pub async fn resolve_market_ticker(&self, event_ticker: &str, target_team: Option<&str>) -> Result<String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();

        let path = "/trade-api/v2/markets";
        let signature = generate_signature(&self.private_key, timestamp, "GET", path)?;

        let url = format!("{}{}", self.base_url, path);
        
        let event_ticker_upper = event_ticker.to_uppercase();
        
        let response = self.client
            .get(&url)
            .header("KALSHI-ACCESS-KEY", &self.api_key)
            .header("KALSHI-ACCESS-TIMESTAMP", timestamp.to_string())
            .header("KALSHI-ACCESS-SIGNATURE", signature)
            .query(&[("event_ticker", &event_ticker_upper)])
            .send()
            .await
            .context("Failed to fetch markets for event")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Kalshi API error {}: {}", status, text));
        }

        let markets_response: MarketsResponse = response
            .json()
            .await
            .context("Failed to parse Kalshi response")?;

        if markets_response.markets.is_empty() {
            anyhow::bail!("No markets found for event ticker: {}", event_ticker);
        }

        // 1. Try to find a match for the target team
        if let Some(team) = target_team {
            let team_lower = team.to_lowercase();
            
            // First pass: look for exact ticker suffix match
            for market in &markets_response.markets {
                if market.ticker.to_lowercase().ends_with(&format!("-{}", team_lower)) {
                    return Ok(market.ticker.clone());
                }
            }
            
            // Second pass: look for team name as the subject in the title
            for market in &markets_response.markets {
                if let Some(title) = &market.title {
                    let title_lower = title.to_lowercase();
                    // "Will GAM Esports win..." is better than "Will TSW win the GAM vs TSW..."
                    if title_lower.starts_with(&format!("will {}", team_lower)) {
                        return Ok(market.ticker.clone());
                    }
                }
            }
            
            // Third pass: any mention in ticker or title
            for market in &markets_response.markets {
                let matches_title = market.title.as_ref().map(|t| t.to_lowercase().contains(&team_lower)).unwrap_or(false);
                let matches_ticker = market.ticker.to_lowercase().contains(&team_lower);
                
                if matches_title || matches_ticker {
                    return Ok(market.ticker.clone());
                }
            }
        }

        // 2. Fallback: Return the first active/binary market
        Ok(markets_response.markets[0].ticker.clone())
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MarketsResponse {
    markets: Vec<MarketInfo>,
    cursor: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarketInfo {
    pub ticker: String,
    pub title: Option<String>,
}
