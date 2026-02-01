#!/usr/bin/env python3
"""
Polymarket CLOB Real-Time Monitor
==================================

A production-ready WebSocket client for monitoring Polymarket's Central Limit Order Book (CLOB)
to detect latency arbitrage opportunities during live sports/esports matches.

HOW TO FIND ASSET_IDs FOR LPL MATCHES:
--------------------------------------
1. Use the Gamma Markets API to search for markets:
   GET https://gamma-api.polymarket.com/markets?limit=100

2. Filter by slug or title (e.g., "jdg-vs-al" or "lpl"):
   GET https://gamma-api.polymarket.com/markets?slug=jdg-vs-al-2026-01-30

3. From the response, extract the `tokens` array. Each token has:
   - `token_id`: This is your asset_id for WebSocket subscription
   - `outcome`: The outcome name (e.g., "JDG Wins", "AL Wins")
   - `price`: Current market price

Example code snippet to fetch asset_ids:
```python
import requests
response = requests.get('https://gamma-api.polymarket.com/markets', params={'slug': 'jdg-vs-al'})
markets = response.json()
for market in markets:
    for token in market.get('tokens', []):
        print(f"Outcome: {token['outcome']}, Asset ID: {token['token_id']}")
```

WebSocket Documentation:
------------------------
- Endpoint: wss://ws-subscriptions-clob.polymarket.com/ws/
- Channel: market (public, no authentication required)
- Events: trades, book (order book updates)
"""

import asyncio
import json
import time
import websockets
from datetime import datetime
from typing import Dict, List, Optional, Set
from collections import deque
from dataclasses import dataclass
import sys
import argparse
import requests


# ANSI color codes for terminal output
class Colors:
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    BOLD = '\033[1m'
    RESET = '\033[0m'


@dataclass
class Trade:
    """Represents a trade event"""
    timestamp: float
    side: str
    price: float
    size: float
    asset_id: str


@dataclass
class BookSnapshot:
    """Represents order book state"""
    timestamp: float
    best_bid: Optional[float]
    best_ask: Optional[float]
    asset_id: str


class VolatilityMonitor:
    """Monitors price movements and volume spikes for volatility alerts"""
    
    def __init__(self, window_seconds: int = 5, price_threshold: float = 0.02, volume_multiplier: float = 3.0):
        self.window_seconds = window_seconds
        self.price_threshold = price_threshold
        self.volume_multiplier = volume_multiplier
        
        # Store recent trades per asset_id
        self.trade_history: Dict[str, deque] = {}
        # Track baseline volume per asset_id
        self.baseline_volume: Dict[str, float] = {}
        # Track last price per asset_id
        self.last_price: Dict[str, float] = {}
    
    def add_trade(self, trade: Trade) -> Optional[str]:
        """
        Add a trade and check for volatility alerts.
        Returns alert message if triggered, None otherwise.
        """
        asset_id = trade.asset_id
        current_time = trade.timestamp
        
        # Initialize structures for new asset
        if asset_id not in self.trade_history:
            self.trade_history[asset_id] = deque()
            self.baseline_volume[asset_id] = 0.0
            self.last_price[asset_id] = trade.price
        
        # Add trade to history
        self.trade_history[asset_id].append(trade)
        
        # Remove trades outside the window
        while self.trade_history[asset_id] and \
              (current_time - self.trade_history[asset_id][0].timestamp) > self.window_seconds:
            self.trade_history[asset_id].popleft()
        
        alerts = []
        
        # Check price movement
        if self.last_price[asset_id] > 0:
            price_change = abs(trade.price - self.last_price[asset_id]) / self.last_price[asset_id]
            if price_change > self.price_threshold:
                direction = "UP" if trade.price > self.last_price[asset_id] else "DOWN"
                alerts.append(f"PRICE SPIKE {direction}: {price_change*100:.2f}% change")
        
        self.last_price[asset_id] = trade.price
        
        # Check volume spike
        window_volume = sum(t.size for t in self.trade_history[asset_id])
        
        if self.baseline_volume[asset_id] > 0:
            volume_ratio = window_volume / self.baseline_volume[asset_id]
            if volume_ratio > self.volume_multiplier:
                alerts.append(f"VOLUME SPIKE: {volume_ratio:.1f}x baseline")
        
        # Update baseline (rolling average)
        if len(self.trade_history[asset_id]) >= 3:
            self.baseline_volume[asset_id] = window_volume / self.window_seconds
        
        if alerts:
            return f"{Colors.BOLD}{Colors.RED}🚨 VOLATILITY ALERT [{asset_id}]: {' | '.join(alerts)}{Colors.RESET}"
        
        return None


class PolymarketCLOBMonitor:
    """Main WebSocket client for monitoring Polymarket CLOB"""
    
    WS_URL = "wss://ws-subscriptions-clob.polymarket.com/ws/market"
    PING_INTERVAL = 20  # Built-in WebSocket ping interval
    PING_TIMEOUT = 20   # Timeout for ping response
    BOOK_DEPTH = 5      # Number of price levels to display in order book
    
    def __init__(self, asset_ids: List[str], show_full_book: bool = False):
        self.asset_ids = asset_ids
        self.websocket: Optional[websockets.WebSocketClientProtocol] = None
        self.running = False
        self.reconnect_delay = 1  # Initial reconnect delay in seconds
        self.max_reconnect_delay = 60  # Max delay between reconnects
        self.volatility_monitor = VolatilityMonitor()
        self.subscribed_assets: Set[str] = set()
        # Track last trade price and mid-price per asset
        self.last_trade_price: Dict[str, float] = {}
        self.last_mid_price: Dict[str, float] = {}
        self.show_full_book = show_full_book  # Whether to show full order book depth
    
    def _get_timestamp(self) -> str:
        """Get formatted timestamp for logging"""
        return datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
    
    def _log(self, message: str, color: str = Colors.CYAN):
        """Log a message with timestamp and color"""
        print(f"{color}[{self._get_timestamp()}] {message}{Colors.RESET}")
    
    # Removed custom ping/pong - using built-in WebSocket ping instead
    
    async def _subscribe_to_assets(self):
        """Subscribe to market data for all configured asset_ids"""
        if not self.websocket:
            return
        
        for asset_id in self.asset_ids:
            if asset_id in self.subscribed_assets:
                continue
                
            subscribe_msg = {
                "auth": {},
                "assets_ids": [asset_id],
                "type": "MARKET"
            }
            
            try:
                await self.websocket.send(json.dumps(subscribe_msg))
                self.subscribed_assets.add(asset_id)
                self._log(f"✅ Subscribed to asset: {asset_id}", Colors.GREEN)
            except Exception as e:
                self._log(f"❌ Failed to subscribe to {asset_id}: {e}", Colors.RED)
    
    def _handle_trade(self, data: dict):
        """Process trade event"""
        try:
            asset_id = data.get('asset_id', 'UNKNOWN')
            side = data.get('side', 'UNKNOWN').upper()
            price = float(data.get('price', 0))
            size = float(data.get('size', 0))
            
            # Create trade object
            trade = Trade(
                timestamp=time.time(),
                side=side,
                price=price,
                size=size,
                asset_id=asset_id
            )
            
            # Track last trade price
            self.last_trade_price[asset_id] = price
            
            # Color code by side
            side_color = Colors.GREEN if side == "BUY" else Colors.RED
            
            # Output trade
            print(f"[{self._get_timestamp()}]{Colors.BOLD}[TRADE]{Colors.RESET} "
                  f"Asset: {asset_id} | "
                  f"Side: {side_color}{side}{Colors.RESET} | "
                  f"Price: {Colors.CYAN}{price:.4f}{Colors.RESET} | "
                  f"Size: {Colors.MAGENTA}{size:.2f}{Colors.RESET}")
            
            # Check for volatility alerts
            alert = self.volatility_monitor.add_trade(trade)
            if alert:
                print(alert)
                
        except Exception as e:
            self._log(f"❌ Error processing trade: {e}", Colors.RED)
    
    def _handle_book(self, data: dict):
        """Process order book update"""
        try:
            asset_id = data.get('asset_id', 'UNKNOWN')
            
            # Extract bids and asks
            bids = data.get('bids', [])
            asks = data.get('asks', [])
            
            # API returns asks sorted HIGHEST to LOWEST, so LAST item is best (lowest) ask
            # API returns bids sorted LOWEST to HIGHEST, so LAST item is best (highest) bid
            best_bid = float(bids[-1]['price']) if bids else None
            best_ask = float(asks[-1]['price']) if asks else None
            
            # Calculate mid-price and spread
            mid_price = None
            spread = None
            spread_bps = None
            
            if best_bid and best_ask:
                mid_price = (best_bid + best_ask) / 2
                spread = best_ask - best_bid
                spread_bps = (spread / best_bid * 10000) if best_bid > 0 else None
                self.last_mid_price[asset_id] = mid_price
            
            # Truncate asset ID for readability
            asset_display = asset_id[:20] + "..." if len(asset_id) > 20 else asset_id
            
            # Show full order book if enabled
            if self.show_full_book and (bids or asks):
                self._display_full_book(asset_id, asset_display, bids, asks, mid_price, spread, spread_bps)
            else:
                # Show compact summary
                self._display_book_summary(asset_id, asset_display, best_bid, best_ask, mid_price, spread, spread_bps)
                  
        except Exception as e:
            self._log(f"❌ Error processing book: {e}", Colors.RED)
    
    def _display_book_summary(self, asset_id: str, asset_display: str, best_bid: Optional[float], 
                             best_ask: Optional[float], mid_price: Optional[float], 
                             spread: Optional[float], spread_bps: Optional[float]):
        """Display compact book summary (original format)"""
        # Format output
        bid_str = f"{Colors.GREEN}{best_bid:.4f}{Colors.RESET}" if best_bid else "N/A"
        ask_str = f"{Colors.RED}{best_ask:.4f}{Colors.RESET}" if best_ask else "N/A"
        
        # Add mid-price
        mid_str = f" | Mid: {Colors.CYAN}{mid_price:.4f}{Colors.RESET}" if mid_price else ""
        
        # Add spread
        spread_str = ""
        if spread and spread_bps:
            # Color code spread: green if tight (<1%), yellow if medium, red if wide
            spread_pct = spread_bps / 100
            if spread_pct < 1:
                spread_color = Colors.GREEN
            elif spread_pct < 5:
                spread_color = Colors.YELLOW
            else:
                spread_color = Colors.RED
            spread_str = f" | Spread: {spread_color}{spread:.4f} ({spread_bps:.1f} bps){Colors.RESET}"
        
        # Add last trade price if available
        last_trade_str = ""
        if asset_id in self.last_trade_price:
            last_price = self.last_trade_price[asset_id]
            last_trade_str = f" | Last Trade: {Colors.MAGENTA}{last_price:.4f}{Colors.RESET}"
        
        print(f"[{self._get_timestamp()}]{Colors.BOLD}[BOOK]{Colors.RESET} "
              f"Asset: {asset_display} | "
              f"Bid: {bid_str} | "
              f"Ask: {ask_str}"
              f"{mid_str}"
              f"{spread_str}"
              f"{last_trade_str}")
    
    def _display_full_book(self, asset_id: str, asset_display: str, bids: list, asks: list,
                          mid_price: Optional[float], spread: Optional[float], spread_bps: Optional[float]):
        """Display full order book with depth"""
        print(f"\n{Colors.BOLD}{'='*80}{Colors.RESET}")
        print(f"{Colors.BOLD}[{self._get_timestamp()}] ORDER BOOK - Asset: {asset_display}{Colors.RESET}")
        
        # Show mid-price and spread
        if mid_price:
            spread_pct = spread_bps / 100 if spread_bps else 0
            spread_color = Colors.GREEN if spread_pct < 1 else (Colors.YELLOW if spread_pct < 5 else Colors.RED)
            print(f"Mid: {Colors.CYAN}{mid_price:.4f}{Colors.RESET} | "
                  f"Spread: {spread_color}{spread:.4f} ({spread_bps:.1f} bps){Colors.RESET}")
        
        # Show last trade if available
        if asset_id in self.last_trade_price:
            print(f"Last Trade: {Colors.MAGENTA}{self.last_trade_price[asset_id]:.4f}{Colors.RESET}")
        
        print(f"{Colors.BOLD}{'-'*80}{Colors.RESET}")
        
        # Filter and display asks - only show levels closest to best ask
        print(f"{Colors.BOLD}{Colors.RED}ASKS (Sell Orders):{Colors.RESET}")
        print(f"{'Price':<12} {'Size':<15} {'Cumulative':<15}")
        
        cumulative_ask_size = 0
        # API returns asks sorted HIGHEST to LOWEST (0.99 -> 0.44)
        # We want the LAST X items (closest to mid-price)
        display_asks = asks[-self.BOOK_DEPTH:] if len(asks) > self.BOOK_DEPTH else asks
        
        # Display from highest to lowest (furthest to closest to mid)
        # display_asks is already in correct order after slicing
        for ask in display_asks:
            price = float(ask.get('price', 0))
            size = float(ask.get('size', 0))
            cumulative_ask_size += size
            print(f"{Colors.RED}{price:<12.4f}{Colors.RESET} {size:<15.2f} {cumulative_ask_size:<15.2f}")
        
        # Show mid-price line
        if mid_price:
            print(f"{Colors.CYAN}{'─'*12} MID: {mid_price:.4f} {'─'*50}{Colors.RESET}")
        
        # Filter and display bids - only show levels closest to best bid
        print(f"{Colors.BOLD}{Colors.GREEN}BIDS (Buy Orders):{Colors.RESET}")
        print(f"{'Price':<12} {'Size':<15} {'Cumulative':<15}")
        
        cumulative_bid_size = 0
        # API returns bids sorted LOWEST to HIGHEST (0.01 -> 0.43)
        # We want the LAST X items (closest to mid-price)
        display_bids = bids[-self.BOOK_DEPTH:] if len(bids) > self.BOOK_DEPTH else bids
        
        # Reverse to show from highest (closest to mid) to lowest (furthest from mid)
        for bid in reversed(display_bids):
            price = float(bid.get('price', 0))
            size = float(bid.get('size', 0))
            cumulative_bid_size += size
            print(f"{Colors.GREEN}{price:<12.4f}{Colors.RESET} {size:<15.2f} {cumulative_bid_size:<15.2f}")
        
        # Show total liquidity
        total_liquidity = cumulative_bid_size + cumulative_ask_size
        print(f"{Colors.BOLD}{'-'*80}{Colors.RESET}")
        print(f"Total Liquidity (top {self.BOOK_DEPTH} levels): {Colors.CYAN}{total_liquidity:.2f}{Colors.RESET}")
        
        # Show warning if spread is very wide (indicates low liquidity)
        if spread_bps and spread_bps > 10000:  # > 100% spread
            print(f"{Colors.YELLOW}⚠️  Warning: Very wide spread indicates low liquidity{Colors.RESET}")
        
        print(f"{Colors.BOLD}{'='*80}{Colors.RESET}\n")
    
    async def _handle_message(self, message):
        """Process incoming WebSocket message"""
        try:
            # Skip binary/empty messages (from WebSocket pings/pongs)
            if isinstance(message, bytes) or not message or (isinstance(message, str) and message.strip() == ''):
                return
                
            data = json.loads(message)
            
            # Handle both list and dict responses
            messages_to_process = data if isinstance(data, list) else [data]
            
            for msg in messages_to_process:
                msg_type = msg.get('event_type') or msg.get('type')
                
                if msg_type == 'trade':
                    self._handle_trade(msg)
                
                elif msg_type == 'book':
                    self._handle_book(msg)
                
                elif msg_type == 'price_change':
                    # Price change events - could be used for additional alerts
                    pass  # Silently ignore for now
                
                elif msg_type == 'last_trade_price':
                    # Track last trade price for display in book updates
                    asset_id = msg.get('asset_id', 'UNKNOWN')
                    price = msg.get('price')
                    if price:
                        self.last_trade_price[asset_id] = float(price)
                        # Optionally log it
                        asset_display = asset_id[:20] + "..." if len(asset_id) > 20 else asset_id
                        self._log(f"💰 Last Trade Price [{asset_display}]: {Colors.MAGENTA}{float(price):.4f}{Colors.RESET}", Colors.CYAN)
                
                elif msg_type == 'subscribed':
                    self._log(f"✅ Subscription confirmed", Colors.GREEN)
                
                elif msg_type == 'error':
                    self._log(f"❌ Server error: {msg.get('message', 'Unknown error')}", Colors.RED)
                
                elif msg_type:
                    # Log unknown message types for debugging (only once per type)
                    if not hasattr(self, '_logged_types'):
                        self._logged_types = set()
                    if msg_type not in self._logged_types:
                        self._log(f"📩 New message type: {msg_type}", Colors.YELLOW)
                        self._logged_types.add(msg_type)
                
        except json.JSONDecodeError:
            # Silently skip - likely WebSocket control frames
            pass
        except Exception as e:
            self._log(f"❌ Error handling message: {e}", Colors.RED)
    
    async def _connect_and_listen(self):
        """Establish WebSocket connection and listen for messages"""
        try:
            self._log(f"🔌 Connecting to {self.WS_URL}...", Colors.CYAN)
            
            async with websockets.connect(
                self.WS_URL,
                ping_interval=self.PING_INTERVAL,  # Built-in WebSocket ping
                ping_timeout=self.PING_TIMEOUT,
                close_timeout=10
            ) as websocket:
                self.websocket = websocket
                self.subscribed_assets.clear()
                
                self._log("✅ Connected successfully!", Colors.GREEN)
                
                # Subscribe to assets
                await self._subscribe_to_assets()
                
                # Listen for messages
                try:
                    async for message in websocket:
                        await self._handle_message(message)
                except websockets.exceptions.ConnectionClosed:
                    self._log("⚠️  Connection closed by server", Colors.YELLOW)
                    
        except Exception as e:
            self._log(f"❌ Connection error: {e}", Colors.RED)
    
    async def start(self):
        """Start the monitor with automatic reconnection"""
        self.running = True
        self._log("🚀 Starting Polymarket CLOB Monitor", Colors.CYAN)
        self._log(f"📊 Monitoring {len(self.asset_ids)} asset(s): {', '.join(self.asset_ids)}", Colors.CYAN)
        
        while self.running:
            try:
                await self._connect_and_listen()
            except KeyboardInterrupt:
                self._log("⏹️  Shutting down...", Colors.YELLOW)
                self.running = False
                break
            except Exception as e:
                self._log(f"❌ Unexpected error: {e}", Colors.RED)
            
            if self.running:
                # Exponential backoff for reconnection
                self._log(f"🔄 Reconnecting in {self.reconnect_delay} seconds...", Colors.YELLOW)
                await asyncio.sleep(self.reconnect_delay)
                self.reconnect_delay = min(self.reconnect_delay * 2, self.max_reconnect_delay)
            else:
                # Reset delay on successful connection
                self.reconnect_delay = 1
        
        self._log("👋 Monitor stopped", Colors.CYAN)
    
    async def stop(self):
        """Stop the monitor gracefully"""
        self.running = False
        if self.websocket:
            try:
                await self.websocket.close()
            except Exception:
                pass  # Already closed

def resolve_slug(slug: str) -> List[str]:
    """
    Resolve Polymarket event slug to asset IDs for Match Winner.
    Returns a list of asset IDs.
    """
    print(f"{Colors.YELLOW}🔍 Resolving Polymarket slug: {slug}...{Colors.RESET}")
    
    # Extract anchor from slug (e.g., "dota2-vg-yb1" -> "vg")
    parts = slug.split('-')
    if len(parts) < 2:
        print(f"{Colors.RED}❌ Invalid slug format: {slug}{Colors.RESET}")
        return []
    
    # For esports/sports, the first team is typically the second part
    # but we'll try to be smart about it.
    anchor = parts[1].lower()
    
    try:
        url = "https://gamma-api.polymarket.com/events"
        response = requests.get(url, params={"slug": slug})
        response.raise_for_status()
        
        events = response.json()
        if not events:
            print(f"{Colors.RED}❌ No event found for slug: {slug}{Colors.RESET}")
            return []
            
        event = events[0]
        markets = event.get("markets", [])
        
        # Find the match winner market: usually the one that doesn't contain sub-market keywords
        market = None
        for m in markets:
            q = m.get("question", "").lower()
            if not any(k in q for k in ["game", "blood", "handicap", "o/u", "spread", "total", "half", "1h", "2h"]):
                market = m
                break
        
        if not market:
            market = markets[0] if markets else None
            
        if not market:
            print(f"{Colors.RED}❌ No markets found in event{Colors.RESET}")
            return []

        print(f"{Colors.GREEN}✓ Resolved Question: {market.get('question')}{Colors.RESET}")
        
        clob_token_ids_raw = market.get("clobTokenIds", "[]")
        outcomes_raw = market.get("outcomes", "[]")
        
        token_ids = json.loads(clob_token_ids_raw)
        outcomes = json.loads(outcomes_raw)
        
        print(f"  Outcomes Found: {outcomes}")
        
        # Look for anchor in outcomes
        target_index = None
        for i, outcome in enumerate(outcomes):
            if anchor in outcome.lower():
                target_index = i
                print(f"{Colors.GREEN}✓ Selected Outcome: {outcome} (index {i}){Colors.RESET}")
                break
        
        if target_index is not None:
            asset_id = token_ids[target_index]
            return [asset_id]
        else:
            print(f"{Colors.YELLOW}⚠️  Warning: Anchor team '{anchor}' not found in outcomes. Using all tokens.{Colors.RESET}")
            return token_ids
            
    except Exception as e:
        print(f"{Colors.RED}❌ Error resolving slug: {e}{Colors.RESET}")
        return []


async def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="Polymarket CLOB Real-Time Monitor")
    parser.add_argument("--slug", type=str, help="Polymarket event slug to monitor")
    parser.add_argument("--asset-id", type=str, help="Comma-separated asset IDs to monitor")
    parser.add_argument("--full-book", action="store_true", help="Show full order book depth")
    
    args = parser.parse_args()
    
    print(f"{Colors.BOLD}{Colors.CYAN}")
    print("=" * 70)
    print("  Polymarket CLOB Real-Time Monitor")
    print("  High-Frequency Trading & Latency Arbitrage Detection")
    print("=" * 70)
    print(f"{Colors.RESET}\n")
    
    show_full_book = args.full_book
    if show_full_book:
        print(f"{Colors.GREEN}📊 Full order book mode enabled{Colors.RESET}\n")
    
    asset_ids = []
    
    if args.slug:
        asset_ids = resolve_slug(args.slug)
    elif args.asset_id:
        asset_ids = [aid.strip() for aid in args.asset_id.split(",") if aid.strip()]
    
    # If no CLI arguments, fall back to interactive mode
    if not asset_ids:
        print(f"{Colors.YELLOW}Enter asset IDs to monitor (comma-separated):{Colors.RESET}")
        print(f"{Colors.YELLOW}Example: 21742633143463906290569050155826241533067272736897614950488156847949938836455{Colors.RESET}")
        print(f"{Colors.YELLOW}Or enter a slug to resolve it:{Colors.RESET}")
        print(f"{Colors.YELLOW}Example slug: dota2-vg-yb1-2026-02-01{Colors.RESET}")
        
        user_input = input("> ").strip()
        
        if user_input:
            if "-" in user_input and "." not in user_input and len(user_input) < 100:
                asset_ids = resolve_slug(user_input)
            else:
                asset_ids = [aid.strip() for aid in user_input.split(",") if aid.strip()]
        else:
            # Demo mode
            asset_ids = ["21742633143463906290569050155826241533067272736897614950488156847949938836455"]
            print(f"{Colors.CYAN}Using demo mode with example asset IDs{Colors.RESET}\n")
    
    if not asset_ids:
        print(f"{Colors.RED}❌ No asset IDs provided or resolved. Exiting.{Colors.RESET}")
        return
    
    # Create and start monitor
    monitor = PolymarketCLOBMonitor(asset_ids, show_full_book=show_full_book)
    
    try:
        await monitor.start()
    except KeyboardInterrupt:
        print(f"\n{Colors.YELLOW}⏹️  Interrupted by user{Colors.RESET}")
        await monitor.stop()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print(f"\n{Colors.CYAN}👋 Goodbye!{Colors.RESET}")
        sys.exit(0)
