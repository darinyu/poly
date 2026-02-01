use anyhow::{Context, Result};
use colored::Colorize;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;

/// Polymarket orderbook level
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct OrderbookLevel {
    pub price: String,
    pub size: String,
}

/// Polymarket market data
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PolymarketMarket {
    pub token_id: String,
    pub best_bid: f64,
    pub best_ask: f64,
    pub bids: Vec<OrderbookLevel>,
    pub asks: Vec<OrderbookLevel>,
}

/// Polymarket WebSocket client
pub struct PolymarketClient {
    ws_url: String,
    ws_stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    asset_id: String,
    debug: bool,
}

impl PolymarketClient {
    pub fn new(ws_url: String, asset_id: String) -> Self {
        Self { 
            ws_url,
            ws_stream: None,
            asset_id,
            debug: false,
        }
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Connect and subscribe to market updates
    pub async fn connect(&mut self) -> Result<()> {
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&self.ws_url)
            .await
            .context("Failed to connect to Polymarket WebSocket")?;

        let (mut write, read) = ws_stream.split();

        // Subscribe to market - matches Python format
        let subscribe_msg = json!({
            "auth": {},
            "assets_ids": [&self.asset_id],
            "type": "MARKET"
        });

        write
            .send(Message::Text(subscribe_msg.to_string()))
            .await
            .context("Failed to send subscribe message")?;

        // Reunite the split stream
        self.ws_stream = Some(write.reunite(read).map_err(|e| anyhow::anyhow!("Failed to reunite stream: {}", e))?);

        Ok(())
    }

    /// Read next orderbook update from WebSocket
    pub async fn read_next_book(&mut self) -> Result<PolymarketMarket> {
        let ws_stream = self.ws_stream.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Not connected. Call connect() first"))?;

        // Read messages until we get a book update
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if self.debug {
                        println!("\n{} {}", "[DEBUG] Polymarket Raw Message:".yellow().bold(), text);
                    }
                    // Parse as array or single object
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                        // Handle array of messages
                        let messages = if data.is_array() {
                            data.as_array().unwrap().clone()
                        } else {
                            vec![data]
                        };

                        for msg_data in messages {
                            let event_type = msg_data["event_type"].as_str()
                                .or_else(|| msg_data["type"].as_str());

                            if event_type == Some("book") {
                                return self.parse_orderbook(&msg_data);
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    return Err(anyhow::anyhow!("WebSocket closed"));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("WebSocket error: {}", e));
                }
                _ => {}
            }
        }

        Err(anyhow::anyhow!("No orderbook data received"))
    }

    fn parse_orderbook(&self, data: &serde_json::Value) -> Result<PolymarketMarket> {
        let mut bids: Vec<OrderbookLevel> = serde_json::from_value(
            data["bids"].clone()
        ).unwrap_or_default();
        
        let mut asks: Vec<OrderbookLevel> = serde_json::from_value(
            data["asks"].clone()
        ).unwrap_or_default();

        // Sort bids descending (best bid at the top)
        bids.sort_by(|a, b| {
            let a_p = a.price.parse::<f64>().unwrap_or(0.0);
            let b_p = b.price.parse::<f64>().unwrap_or(0.0);
            b_p.partial_cmp(&a_p).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Sort asks ascending (best ask at the top)
        asks.sort_by(|a, b| {
            let a_p = a.price.parse::<f64>().unwrap_or(0.0);
            let b_p = b.price.parse::<f64>().unwrap_or(0.0);
            a_p.partial_cmp(&b_p).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Best bid is the first item after sorting
        let best_bid = bids.first()
            .and_then(|b| b.price.parse::<f64>().ok())
            .unwrap_or(0.0);

        let best_ask = asks.first()
            .and_then(|a| a.price.parse::<f64>().ok())
            .unwrap_or(0.0);

        Ok(PolymarketMarket {
            token_id: self.asset_id.clone(),
            best_bid,
            best_ask,
            bids,
            asks,
        })
    }
}
