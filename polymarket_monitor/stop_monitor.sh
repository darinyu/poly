#!/bin/bash
# Stop the background monitor

if [ -f monitor.pid ]; then
    PID=$(cat monitor.pid)
    echo "Stopping monitor (PID: $PID)..."
    kill $PID 2>/dev/null
    
    if [ $? -eq 0 ]; then
        echo "Monitor stopped successfully"
        rm monitor.pid
    else
        echo "Monitor process not found (may have already stopped)"
        rm monitor.pid
    fi
else
    echo "No monitor.pid file found"
    echo "Searching for running Python monitor processes..."
    
    # Find and display running monitor processes
    pgrep -f "polymarket_clob_monitor.py" | while read pid; do
        echo "Found process: $pid"
        ps -p $pid -o pid,command
        read -p "Kill this process? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            kill $pid
            echo "Killed process $pid"
        fi
    done
fi
