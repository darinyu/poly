#!/bin/bash
# Quick test script to run the monitor for 15 seconds

cd /Users/zitingyu/poly

# Activate virtual environment
. venv/bin/activate

# Run monitor with live Dota 2 match asset IDs
echo "41264804488835355265695098525678596135833794068920491098138408934378340337565,10488085140621281562634039768200473701174016782047309693731956948653474012052" | python polymarket_clob_monitor.py &

# Store PID
MONITOR_PID=$!

# Wait 15 seconds
sleep 15

# Kill the monitor
kill $MONITOR_PID 2>/dev/null

echo ""
echo "✅ Test complete!"
