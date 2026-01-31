#!/usr/bin/env python3
"""
Example: Custom Trading Strategy Integration

This demonstrates how to extend the PolymarketCLOBMonitor
to implement a custom latency arbitrage strategy.
"""

import asyncio
from polymarket_clob_monitor import PolymarketCLOBMonitor, Trade, Colors


class ArbitrageMonitor(PolymarketCLOBMonitor):
    """
    Extended monitor with custom arbitrage detection logic
    """
    
    def __init__(self, asset_ids: list):
        super().__init__(asset_ids)
        
        # Track prices across multiple assets for arbitrage
        self.asset_prices = {}
        self.arbitrage_threshold = 0.01  # 1% price difference
    
    def _handle_trade(self, data: dict):
        """Override to add custom logic"""
        # Call parent implementation for standard output
        super()._handle_trade(data)
        
        # Custom arbitrage detection
        asset_id = data.get('asset_id', 'UNKNOWN')
        price = float(data.get('price', 0))
        
        # Store latest price
        self.asset_prices[asset_id] = price
        
        # Check for arbitrage opportunities across assets
        self._check_arbitrage()
    
    def _check_arbitrage(self):
        """
        Detect arbitrage opportunities between correlated assets
        
        Example: If monitoring "JDG Wins" and "AL Wins" for the same match,
        their combined probability should equal 1.0. If not, there's an arbitrage.
        """
        if len(self.asset_prices) < 2:
            return
        
        prices = list(self.asset_prices.values())
        total_probability = sum(prices)
        
        # In a binary market, prices should sum to ~1.0
        # If they don't, there's potential arbitrage
        deviation = abs(1.0 - total_probability)
        
        if deviation > self.arbitrage_threshold:
            self._log_arbitrage(deviation, total_probability)
    
    def _log_arbitrage(self, deviation: float, total_prob: float):
        """Log arbitrage opportunity"""
        print(f"\n{Colors.BOLD}{Colors.MAGENTA}💰 ARBITRAGE OPPORTUNITY DETECTED!{Colors.RESET}")
        print(f"{Colors.MAGENTA}   Total Probability: {total_prob:.4f} (should be ~1.0){Colors.RESET}")
        print(f"{Colors.MAGENTA}   Deviation: {deviation*100:.2f}%{Colors.RESET}")
        print(f"{Colors.MAGENTA}   Prices: {self.asset_prices}{Colors.RESET}\n")
        
        # Here you would:
        # 1. Calculate optimal bet sizes
        # 2. Place orders via Polymarket API
        # 3. Log the trade for analysis


class VolumeImbalanceMonitor(PolymarketCLOBMonitor):
    """
    Monitor for order flow imbalance (more buys than sells = bullish signal)
    """
    
    def __init__(self, asset_ids: list):
        super().__init__(asset_ids)
        self.buy_volume = {}
        self.sell_volume = {}
        self.imbalance_threshold = 2.0  # 2:1 buy/sell ratio
    
    def _handle_trade(self, data: dict):
        """Track buy/sell volume"""
        super()._handle_trade(data)
        
        asset_id = data.get('asset_id', 'UNKNOWN')
        side = data.get('side', 'UNKNOWN').upper()
        size = float(data.get('size', 0))
        
        # Initialize counters
        if asset_id not in self.buy_volume:
            self.buy_volume[asset_id] = 0
            self.sell_volume[asset_id] = 0
        
        # Track volume by side
        if side == 'BUY':
            self.buy_volume[asset_id] += size
        elif side == 'SELL':
            self.sell_volume[asset_id] += size
        
        # Check for imbalance
        self._check_imbalance(asset_id)
    
    def _check_imbalance(self, asset_id: str):
        """Detect significant order flow imbalance"""
        buy_vol = self.buy_volume.get(asset_id, 0)
        sell_vol = self.sell_volume.get(asset_id, 0)
        
        if sell_vol == 0:
            return
        
        ratio = buy_vol / sell_vol
        
        if ratio > self.imbalance_threshold:
            print(f"\n{Colors.BOLD}{Colors.GREEN}📈 BULLISH SIGNAL [{asset_id}]{Colors.RESET}")
            print(f"{Colors.GREEN}   Buy/Sell Ratio: {ratio:.2f}:1{Colors.RESET}")
            print(f"{Colors.GREEN}   Buy Volume: {buy_vol:.2f} | Sell Volume: {sell_vol:.2f}{Colors.RESET}\n")
        
        elif ratio < (1 / self.imbalance_threshold):
            print(f"\n{Colors.BOLD}{Colors.RED}📉 BEARISH SIGNAL [{asset_id}]{Colors.RESET}")
            print(f"{Colors.RED}   Buy/Sell Ratio: {ratio:.2f}:1{Colors.RESET}")
            print(f"{Colors.RED}   Buy Volume: {buy_vol:.2f} | Sell Volume: {sell_vol:.2f}{Colors.RESET}\n")


async def main():
    """Example usage"""
    print(f"{Colors.BOLD}{Colors.CYAN}Custom Strategy Example{Colors.RESET}\n")
    
    # Example: Monitor both outcomes of a binary market
    asset_ids = [
        "21742633143463906290569050155826241533067272736897614950488156847949938836455",  # JDG Wins
        # Add the "AL Wins" asset_id here for arbitrage detection
    ]
    
    print("Choose strategy:")
    print("1. Arbitrage Monitor (detects mispriced binary markets)")
    print("2. Volume Imbalance Monitor (detects order flow signals)")
    print("3. Standard Monitor (default)")
    
    choice = input("> ").strip()
    
    if choice == "1":
        monitor = ArbitrageMonitor(asset_ids)
    elif choice == "2":
        monitor = VolumeImbalanceMonitor(asset_ids)
    else:
        monitor = PolymarketCLOBMonitor(asset_ids)
    
    await monitor.start()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print(f"\n{Colors.CYAN}👋 Strategy stopped{Colors.RESET}")
