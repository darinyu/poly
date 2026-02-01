use anyhow::{Context, Result};
use colored::Colorize;
use serde_json;

/// Get Polymarket asset ID and anchor team from event slug
pub async fn get_asset_id_and_anchor(slug: &str, verbose: bool, debug: bool) -> Result<(String, String)> {
    let client = reqwest::Client::new();
    
    // 1. Extract anchor from slug (e.g., "dal" from "nba-dal-hou-2026-01-31")
    // Format is usually category-team1-team2-date
    let parts: Vec<&str> = slug.split('-').collect();
    let anchor = parts.get(1).context("Invalid slug format: could not extract anchor")?.to_lowercase();
    
    // Fetch event from Gamma API using slug query parameter
    let url = "https://gamma-api.polymarket.com/events";
    let response = client
        .get(url)
        .query(&[("slug", slug)])
        .send()
        .await
        .context("Failed to fetch Polymarket event")?;
    
    if !response.status().is_success() {
        anyhow::bail!("Polymarket API returned error: {}", response.status());
    }

    #[derive(serde::Deserialize)]
    struct Event {
        markets: Vec<Market>,
    }
    
    #[derive(serde::Deserialize)]
    struct Market {
        #[serde(rename = "clobTokenIds")]
        clob_token_ids: String,  // JSON string array
        outcomes: String,        // JSON string array
        question: String,
    }
    
    let events: Vec<Event> = response
        .json()
        .await
        .context("Failed to parse Polymarket events JSON array")?;
    
    let event = events.first().context("No event found for slug")?;
    
    // Find the match winner market: usually the one that doesn't contain "Game" or "First Blood" 
    // or is the generic one with the match title.
    let market = event.markets.iter().find(|m| {
        let q = m.question.to_lowercase();
        !q.contains("game") && !q.contains("blood") && !q.contains("handicap") && 
        !q.contains("o/u") && !q.contains("spread") && !q.contains("total") &&
        !q.contains("half") && !q.contains("1h") && !q.contains("2h")
    }).or(event.markets.first()).context("No markets found in event")?;

    if verbose || debug {
        println!("  Resolved Question: {}", market.question.cyan());
    }

    let token_ids: Vec<String> = serde_json::from_str(&market.clob_token_ids)
        .context("Failed to parse clobTokenIds JSON string")?;
    
    let outcomes: Vec<String> = serde_json::from_str(&market.outcomes)
        .context("Failed to parse outcomes JSON string")?;
        
    // Look for anchor in outcomes
    if verbose || debug {
        println!("  Outcomes Found:   {:?}", outcomes);
    }
    let mut target_index = None;
    for (i, outcome) in outcomes.iter().enumerate() {
        if outcome.to_lowercase().contains(&anchor) {
            target_index = Some(i);
            if verbose || debug {
                println!("  Selected Outcome: {} (index {})", outcome.green(), i);
            }
            break;
        }
    }
    
    let index = match target_index {
        Some(idx) => idx,
        None => {
            if verbose || debug {
                println!("{}", format!("  ⚠️  Warning: Anchor team '{}' not found in outcomes. Defaulting to first outcome ({}).", anchor, outcomes.first().unwrap_or(&"none".to_string())).red().bold());
            }
            0
        }
    };
    
    if let Some(asset_id) = token_ids.get(index) {
        return Ok((asset_id.clone(), anchor));
    }
    
    anyhow::bail!("No asset ID found for slug: {}", slug)
}
