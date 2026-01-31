#!/bin/bash
# Helper script to run the Polymarket CLOB monitor in various modes

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Activate virtual environment
source venv/bin/activate

# Default values
MODE="foreground"
LOG_FILE="monitor_$(date +%Y%m%d_%H%M%S).log"
ASSET_IDS=""
FULL_BOOK=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --daemon)
            MODE="daemon"
            shift
            ;;
        --background)
            MODE="background"
            shift
            ;;
        --log)
            LOG_FILE="$2"
            shift 2
            ;;
        --assets)
            ASSET_IDS="$2"
            shift 2
            ;;
        --full-book)
            FULL_BOOK="--full-book"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--daemon|--background] [--log LOGFILE] [--assets ASSET_IDS] [--full-book]"
            exit 1
            ;;
    esac
done

case $MODE in
    foreground)
        echo "Running in foreground mode..."
        echo "Output will be displayed in terminal and saved to: $LOG_FILE"
        echo "Press Ctrl+C to stop"
        echo ""
        
        if [ -n "$ASSET_IDS" ]; then
            echo "$ASSET_IDS" | python polymarket_clob_monitor.py $FULL_BOOK 2>&1 | tee "$LOG_FILE"
        else
            python polymarket_clob_monitor.py $FULL_BOOK 2>&1 | tee "$LOG_FILE"
        fi
        ;;
        
    background)
        echo "Running in background mode..."
        echo "Output will be saved to: $LOG_FILE"
        echo "PID will be saved to: monitor.pid"
        echo ""
        
        if [ -n "$ASSET_IDS" ]; then
            echo "$ASSET_IDS" | nohup python polymarket_clob_monitor.py $FULL_BOOK > "$LOG_FILE" 2>&1 &
        else
            nohup python polymarket_clob_monitor.py $FULL_BOOK > "$LOG_FILE" 2>&1 &
        fi
        
        PID=$!
        echo $PID > monitor.pid
        echo "Monitor started with PID: $PID"
        echo "To stop: kill $PID  or  ./stop_monitor.sh"
        echo "To view logs: tail -f $LOG_FILE"
        ;;
        
    daemon)
        echo "Running as daemon (using screen)..."
        echo "Output will be saved to: $LOG_FILE"
        echo ""
        
        # Check if screen is installed
        if ! command -v screen &> /dev/null; then
            echo "Error: 'screen' is not installed. Install with: brew install screen"
            echo "Falling back to background mode..."
            MODE="background"
            exec "$0" --background --log "$LOG_FILE" ${ASSET_IDS:+--assets "$ASSET_IDS"} $FULL_BOOK
            exit 0
        fi
        
        SESSION_NAME="polymarket_monitor_$(date +%s)"
        
        if [ -n "$ASSET_IDS" ]; then
            screen -dmS "$SESSION_NAME" bash -c "source venv/bin/activate && echo '$ASSET_IDS' | python polymarket_clob_monitor.py $FULL_BOOK 2>&1 | tee '$LOG_FILE'"
        else
            screen -dmS "$SESSION_NAME" bash -c "source venv/bin/activate && python polymarket_clob_monitor.py $FULL_BOOK 2>&1 | tee '$LOG_FILE'"
        fi
        
        echo "Monitor started in screen session: $SESSION_NAME"
        echo "To attach: screen -r $SESSION_NAME"
        echo "To detach: Ctrl+A, then D"
        echo "To stop: screen -X -S $SESSION_NAME quit"
        echo "To view logs: tail -f $LOG_FILE"
        ;;
esac
