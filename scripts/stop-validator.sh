#!/bin/bash
# Stop the local Solana test validator
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Stopping Solana test validator..."
echo ""

# Try to stop using PID file first
if [ -f .validator-pid ]; then
    PID=$(cat .validator-pid)
    if ps -p $PID > /dev/null 2>&1; then
        echo "Stopping validator (PID: $PID)..."
        kill $PID
        sleep 2

        # Force kill if still running
        if ps -p $PID > /dev/null 2>&1; then
            echo "Force stopping..."
            kill -9 $PID
        fi

        rm .validator-pid
        echo -e "${GREEN}✓ Validator stopped${NC}"
    else
        echo -e "${YELLOW}Validator not running (stale PID file)${NC}"
        rm .validator-pid
    fi
else
    # Try to find and kill by process name
    if pgrep -f "test-validator\|agave-validator" > /dev/null; then
        echo "Stopping validator by process name..."
        pkill -f "test-validator\|agave-validator" || true
        sleep 2
        echo -e "${GREEN}✓ Validator stopped${NC}"
    else
        echo -e "${YELLOW}No running validator found${NC}"
    fi
fi

echo ""
echo "To clean up ledger data, run:"
echo "  rm -rf .validator-ledger .validator-logs"
