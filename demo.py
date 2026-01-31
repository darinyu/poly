#!/usr/bin/env python3
"""
Quick demo script to test the Polymarket CLOB monitor with example data
This simulates what the monitor looks like when receiving real data
"""

import asyncio
import time
from datetime import datetime


# ANSI color codes
class Colors:
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    BOLD = '\033[1m'
    RESET = '\033[0m'


def get_timestamp():
    return datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]


async def demo():
    """Simulate monitor output"""
    print(f"{Colors.BOLD}{Colors.CYAN}")
    print("=" * 70)
    print("  Polymarket CLOB Monitor - DEMO MODE")
    print("  Simulating real-time data stream")
    print("=" * 70)
    print(f"{Colors.RESET}\n")
    
    asset_id = "21742...836455"
    
    # Simulate connection
    print(f"{Colors.CYAN}[{get_timestamp()}] 🔌 Connecting to wss://ws-subscriptions-clob.polymarket.com/ws/market...{Colors.RESET}")
    await asyncio.sleep(0.5)
    print(f"{Colors.GREEN}[{get_timestamp()}] ✅ Connected successfully!{Colors.RESET}")
    await asyncio.sleep(0.3)
    print(f"{Colors.GREEN}[{get_timestamp()}] ✅ Subscribed to asset: {asset_id}{Colors.RESET}")
    await asyncio.sleep(0.5)
    
    # Simulate initial book snapshot
    print(f"\n[{get_timestamp()}]{Colors.BOLD}[BOOK]{Colors.RESET} "
          f"Asset: {asset_id} | "
          f"Best Bid: {Colors.GREEN}0.5200{Colors.RESET} | "
          f"Best Ask: {Colors.RED}0.5250{Colors.RESET} | "
          f"Spread: {Colors.YELLOW}0.0050 (96.2 bps){Colors.RESET}")
    await asyncio.sleep(1)
    
    # Simulate trades
    trades = [
        ("BUY", 0.5234, 100.00),
        ("SELL", 0.5230, 50.00),
        ("BUY", 0.5240, 200.00),
        ("BUY", 0.5245, 150.00),
    ]
    
    for side, price, size in trades:
        side_color = Colors.GREEN if side == "BUY" else Colors.RED
        print(f"[{get_timestamp()}]{Colors.BOLD}[TRADE]{Colors.RESET} "
              f"Asset: {asset_id} | "
              f"Side: {side_color}{side}{Colors.RESET} | "
              f"Price: {Colors.CYAN}{price:.4f}{Colors.RESET} | "
              f"Size: {Colors.MAGENTA}{size:.2f}{Colors.RESET}")
        await asyncio.sleep(0.8)
    
    # Simulate volatility alert
    print(f"\n{Colors.BOLD}{Colors.RED}🚨 VOLATILITY ALERT [{asset_id}]: "
          f"PRICE SPIKE UP: 2.34% change | VOLUME SPIKE: 4.2x baseline{Colors.RESET}\n")
    await asyncio.sleep(1)
    
    # More book updates
    print(f"[{get_timestamp()}]{Colors.BOLD}[BOOK]{Colors.RESET} "
          f"Asset: {asset_id} | "
          f"Best Bid: {Colors.GREEN}0.5240{Colors.RESET} | "
          f"Best Ask: {Colors.RED}0.5260{Colors.RESET} | "
          f"Spread: {Colors.YELLOW}0.0020 (38.5 bps){Colors.RESET}")
    await asyncio.sleep(1)
    
    # Simulate heartbeat
    print(f"{Colors.YELLOW}[{get_timestamp()}] 📡 Sent ping{Colors.RESET}")
    await asyncio.sleep(0.3)
    print(f"{Colors.YELLOW}[{get_timestamp()}] 📡 Received pong{Colors.RESET}")
    await asyncio.sleep(1)
    
    # More trades
    print(f"[{get_timestamp()}]{Colors.BOLD}[TRADE]{Colors.RESET} "
          f"Asset: {asset_id} | "
          f"Side: {Colors.RED}SELL{Colors.RESET} | "
          f"Price: {Colors.CYAN}0.5255{Colors.RESET} | "
          f"Size: {Colors.MAGENTA}75.00{Colors.RESET}")
    
    await asyncio.sleep(2)
    
    print(f"\n{Colors.CYAN}[{get_timestamp()}] Demo complete! This is what the real monitor looks like.{Colors.RESET}")
    print(f"{Colors.CYAN}[{get_timestamp()}] To run with real data, use: python polymarket_clob_monitor.py{Colors.RESET}\n")


if __name__ == "__main__":
    try:
        asyncio.run(demo())
    except KeyboardInterrupt:
        print(f"\n{Colors.CYAN}👋 Demo stopped{Colors.RESET}")
