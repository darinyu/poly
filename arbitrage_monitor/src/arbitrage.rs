use crate::kalshi::KalshiMarket;
use crate::polymarket::PolymarketMarket;

#[derive(Debug)]
pub struct ArbitrageOpportunity {
    pub buy_platform: String,
    pub sell_platform: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_cents: f64,
    pub profit_pct: f64,
}

/// Detect arbitrage opportunities between Kalshi and Polymarket
pub fn detect_arbitrage(
    kalshi: &KalshiMarket,
    polymarket: &PolymarketMarket,
) -> Option<ArbitrageOpportunity> {
    // Convert Kalshi cents to dollars for comparison
    let kalshi_bid = kalshi.yes_bid as f64 / 100.0;
    let kalshi_ask = kalshi.yes_ask as f64 / 100.0;
    
    let poly_bid = polymarket.best_bid;
    let poly_ask = polymarket.best_ask;

    // Check if we can buy on Polymarket and sell on Kalshi
    if kalshi_bid > poly_ask && poly_ask > 0.0 {
        let profit = kalshi_bid - poly_ask;
        let profit_pct = (profit / poly_ask) * 100.0;
        
        return Some(ArbitrageOpportunity {
            buy_platform: "Polymarket".to_string(),
            sell_platform: "Kalshi".to_string(),
            buy_price: poly_ask,
            sell_price: kalshi_bid,
            profit_cents: profit * 100.0,
            profit_pct,
        });
    }

    // Check if we can buy on Kalshi and sell on Polymarket
    if poly_bid > kalshi_ask && kalshi_ask > 0.0 {
        let profit = poly_bid - kalshi_ask;
        let profit_pct = (profit / kalshi_ask) * 100.0;
        
        return Some(ArbitrageOpportunity {
            buy_platform: "Kalshi".to_string(),
            sell_platform: "Polymarket".to_string(),
            buy_price: kalshi_ask,
            sell_price: poly_bid,
            profit_cents: profit * 100.0,
            profit_pct,
        });
    }

    None
}
