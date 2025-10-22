#!/bin/bash
# Start a local Solana test validator
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if validator command exists
if command -v agave-validator >/dev/null 2>&1; then
    VALIDATOR_CMD="agave-validator"
elif command -v solana-test-validator >/dev/null 2>&1; then
    VALIDATOR_CMD="solana-test-validator"
else
    echo -e "${RED}Error: No test validator found${NC}"
    echo ""
    echo "Neither agave-validator nor solana-test-validator found in PATH."
    echo ""
    echo "To install, run:"
    echo "  ./scripts/setup-native.sh"
    echo ""
    exit 1
fi

echo "========================================"
echo "Starting Solana Test Validator"
echo "========================================"
echo ""
echo "Using: $VALIDATOR_CMD"
echo ""

# Create log directory
mkdir -p .validator-logs

# Check if validator is already running
if pgrep -f "test-validator\|agave-validator" > /dev/null; then
    echo -e "${YELLOW}Warning: A validator appears to be already running${NC}"
    echo ""
    echo "To stop it, run:"
    echo "  ./scripts/stop-validator.sh"
    echo ""
    echo "Or kill manually:"
    echo "  pkill -f 'test-validator|agave-validator'"
    echo ""
    read -p "Start anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "Starting validator with:"
echo "  - RPC: http://localhost:8899"
echo "  - WebSocket: ws://localhost:8900"
echo "  - Ledger: .validator-ledger"
echo "  - Logs: .validator-logs/validator.log"
echo ""

# Start the validator in the background
# Use solana-test-validator with sensible defaults
solana-test-validator \
    --rpc-port 8899 \
    --ledger .validator-ledger \
    --reset \
    > .validator-logs/validator.log 2>&1 &

VALIDATOR_PID=$!
echo $VALIDATOR_PID > .validator-pid

echo -e "${GREEN}✓ Validator started (PID: $VALIDATOR_PID)${NC}"
echo ""
echo "Waiting for validator to be ready..."

# Wait for validator to be ready (max 30 seconds)
MAX_ATTEMPTS=30
ATTEMPT=0
while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    if solana cluster-version --url http://localhost:8899 >/dev/null 2>&1; then
        echo -e "${GREEN}✓ Validator is ready!${NC}"
        echo ""
        echo "Connection info:"
        echo "  RPC endpoint: http://localhost:8899"
        echo "  WebSocket: ws://localhost:8900"
        echo ""
        echo "View logs:"
        echo "  tail -f .validator-logs/validator.log"
        echo ""
        echo "Stop validator:"
        echo "  ./scripts/stop-validator.sh"
        echo ""
        exit 0
    fi
    ATTEMPT=$((ATTEMPT + 1))
    echo -n "."
    sleep 1
done

echo ""
echo -e "${RED}Error: Validator failed to start or is taking too long${NC}"
echo ""
echo "Check logs:"
echo "  tail .validator-logs/validator.log"
echo ""
exit 1
