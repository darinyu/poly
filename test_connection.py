#!/usr/bin/env python3
"""
Quick test script to verify WebSocket connection works
"""
import asyncio
import json
import websockets

async def test_connection():
    """Test the Polymarket CLOB WebSocket connection"""
    
    # Test asset IDs from the live Dota 2 match
    asset_ids = [
        "41264804488835355265695098525678596135833794068920491098138408934378340337565",
        "10488085140621281562634039768200473701174016782047309693731956948653474012052"
    ]
    
    url = "wss://ws-subscriptions-clob.polymarket.com/ws/market"
    
    print(f"🔌 Connecting to {url}...")
    
    try:
        async with websockets.connect(url, ping_interval=None) as websocket:
            print("✅ Connected!")
            
            # Subscribe to assets
            for asset_id in asset_ids:
                subscribe_msg = {
                    "auth": {},
                    "assets_ids": [asset_id],
                    "type": "MARKET"
                }
                
                await websocket.send(json.dumps(subscribe_msg))
                print(f"📤 Sent subscription for {asset_id[:20]}...")
            
            # Listen for messages for 10 seconds
            print("\n📡 Listening for messages (10 seconds)...\n")
            
            try:
                async with asyncio.timeout(10):
                    async for message in websocket:
                        data = json.loads(message)
                        # Handle both list and dict responses
                        if isinstance(data, list):
                            for item in data:
                                msg_type = item.get('event_type') or item.get('type')
                                print(f"📩 Received: {msg_type} - {str(item)[:150]}...")
                        else:
                            msg_type = data.get('event_type') or data.get('type')
                            print(f"📩 Received: {msg_type} - {str(data)[:150]}...")
            except asyncio.TimeoutError:
                print("\n⏱️  Test complete!")
                
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(test_connection())
