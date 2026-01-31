#!/usr/bin/env python3
"""
Helper script to fetch asset IDs from Polymarket Gamma API
Usage: python fetch_asset_ids.py [search_term]
       python fetch_asset_ids.py --slug [slug]
Example: python fetch_asset_ids.py "LPL"
         python fetch_asset_ids.py --slug lol-dcg-cfo-2026-01-31
"""

import requests
import sys
import json


def fetch_markets(search_term: str = "", limit: int = 100, by_slug: bool = False):
    """
    Fetch markets from Gamma API
    
    Args:
        search_term: Optional search term to filter markets
        limit: Maximum number of markets to return
        by_slug: If True, search by exact slug match using API parameter
    """
    base_url = "https://gamma-api.polymarket.com/markets"
    params = {"limit": limit}
    
    # If searching by slug, use the API's slug parameter for exact match
    if by_slug and search_term:
        params["slug"] = search_term
        print(f"🔍 Searching for market with slug: '{search_term}'...\n")
    else:
        print(f"🔍 Searching for markets matching: '{search_term}'...\n")
    
    try:
        response = requests.get(base_url, params=params, timeout=10)
        response.raise_for_status()
        markets = response.json()
        
        # If not using slug parameter, filter markets by search term
        if search_term and not by_slug:
            search_lower = search_term.lower()
            markets = [
                m for m in markets 
                if search_lower in m.get('question', '').lower() or 
                   search_lower in m.get('description', '').lower() or
                   search_lower in m.get('slug', '').lower()
            ]
        
        if not markets:
            print(f"❌ No markets found matching '{search_term}'")
            return
        
        print(f"✅ Found {len(markets)} market(s)\n")
        print("=" * 80)
        
        for idx, market in enumerate(markets, 1):
            print(f"\n📊 Market {idx}: {market.get('question', 'N/A')}")
            print(f"   Slug: {market.get('slug', 'N/A')}")
            print(f"   Description: {market.get('description', 'N/A')[:100]}...")
            print(f"   Active: {market.get('active', False)}")
            print(f"   Closed: {market.get('closed', False)}")
            
            # Show clobTokenIds if available
            clob_token_ids = market.get('clobTokenIds')
            if clob_token_ids:
                try:
                    # Parse the JSON string
                    import ast
                    if isinstance(clob_token_ids, str):
                        token_ids = ast.literal_eval(clob_token_ids)
                    else:
                        token_ids = clob_token_ids
                    
                    print(f"\n   💰 Asset IDs ({len(token_ids)}):")
                    outcomes = market.get('outcomes', '[]')
                    if isinstance(outcomes, str):
                        outcomes = ast.literal_eval(outcomes)
                    
                    for i, token_id in enumerate(token_ids):
                        outcome = outcomes[i] if i < len(outcomes) else f"Outcome {i+1}"
                        print(f"      • {outcome}")
                        print(f"        Asset ID: {token_id}")
                except Exception:
                    pass
            
            # Fallback to tokens array if clobTokenIds not available
            tokens = market.get('tokens', [])
            if tokens and not clob_token_ids:
                print(f"\n   💰 Tokens ({len(tokens)}):")
                for token in tokens:
                    outcome = token.get('outcome', 'N/A')
                    token_id = token.get('token_id', 'N/A')
                    price = token.get('price', 'N/A')
                    
                    print(f"      • Outcome: {outcome}")
                    print(f"        Asset ID: {token_id}")
                    print(f"        Price: {price}")
            
            print("\n" + "-" * 80)
        
        # Print summary with copy-paste ready asset IDs
        print("\n" + "=" * 80)
        print("📋 COPY-PASTE READY ASSET IDs:")
        print("=" * 80)
        
        all_asset_ids = []
        for market in markets:
            # Try clobTokenIds first
            clob_token_ids = market.get('clobTokenIds')
            if clob_token_ids:
                try:
                    import ast
                    if isinstance(clob_token_ids, str):
                        token_ids = ast.literal_eval(clob_token_ids)
                    else:
                        token_ids = clob_token_ids
                    all_asset_ids.extend(token_ids)
                except Exception:
                    pass
            
            # Fallback to tokens array
            if not clob_token_ids:
                for token in market.get('tokens', []):
                    token_id = token.get('token_id')
                    if token_id:
                        all_asset_ids.append(token_id)
        
        if all_asset_ids:
            print("\nComma-separated list:")
            print(",".join(all_asset_ids))
            
            print("\nPython list:")
            print(json.dumps(all_asset_ids, indent=2))
        
    except requests.exceptions.RequestException as e:
        print(f"❌ Error fetching markets: {e}")
        sys.exit(1)


def main():
    """Main entry point"""
    args = sys.argv[1:]
    
    if not args:
        print("⚠️  No search term provided. Fetching all recent markets...")
        print("   Usage: python fetch_asset_ids.py [search_term]")
        print("   Usage: python fetch_asset_ids.py --slug [slug]")
        print("   Example: python fetch_asset_ids.py LPL")
        print("   Example: python fetch_asset_ids.py --slug lol-dcg-cfo-2026-01-31\n")
        fetch_markets("")
        return
    
    # Check if using --slug flag
    if args[0] == "--slug" and len(args) > 1:
        slug = args[1]
        fetch_markets(slug, by_slug=True)
    else:
        search_term = " ".join(args)
        fetch_markets(search_term)


if __name__ == "__main__":
    main()
