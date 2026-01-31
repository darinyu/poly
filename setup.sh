#!/bin/bash
# Setup script for Polymarket CLOB Monitor

echo "🚀 Setting up Polymarket CLOB Monitor..."

# Create virtual environment
if [ ! -d "venv" ]; then
    echo "📦 Creating virtual environment..."
    python3 -m venv venv
fi

# Activate virtual environment
echo "✅ Activating virtual environment..."
source venv/bin/activate

# Install dependencies
echo "📥 Installing dependencies..."
pip install -r requirements.txt

echo ""
echo "✅ Setup complete!"
echo ""
echo "To use the monitor:"
echo "  1. Activate the virtual environment: source venv/bin/activate"
echo "  2. Find asset IDs: python fetch_asset_ids.py LPL"
echo "  3. Run the monitor: python polymarket_clob_monitor.py"
echo "  4. Or try the demo: python demo.py"
echo ""
