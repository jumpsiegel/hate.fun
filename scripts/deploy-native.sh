#!/bin/bash
# Deploy hate.fun program to local test validator
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PROGRAM_PATH="dist/program/hate_fun.so"
RPC_URL="http://localhost:8899"
PROGRAM_ID_FILE=".program-id"

echo "========================================"
echo "Deploying hate.fun to Local Validator"
echo "========================================"
echo ""

# Check if program binary exists
if [ ! -f "$PROGRAM_PATH" ]; then
    echo -e "${RED}Error: Program binary not found: $PROGRAM_PATH${NC}"
    echo ""
    echo "Build the program first:"
    echo "  ./scripts/build-native.sh"
    echo ""
    exit 1
fi

# Check if validator is running
echo "Checking validator health..."
if ! solana cluster-version --url $RPC_URL >/dev/null 2>&1; then
    echo -e "${RED}Error: Cannot connect to validator at $RPC_URL${NC}"
    echo ""
    echo "Start the validator first:"
    echo "  ./scripts/start-validator.sh"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ Validator is running${NC}"
echo ""

# Configure Solana CLI to use local validator
echo "Configuring Solana CLI..."
solana config set --url $RPC_URL >/dev/null

# Check wallet balance, airdrop if needed
BALANCE=$(solana balance 2>/dev/null | awk '{print $1}' || echo "0")
echo "Wallet balance: $BALANCE SOL"

# Simple numeric comparison without bc
BALANCE_INT=$(echo "$BALANCE" | cut -d. -f1)
if [ -z "$BALANCE_INT" ] || [ "$BALANCE_INT" -lt 10 ]; then
    echo "Requesting airdrop..."
    solana airdrop 10 >/dev/null 2>&1 || echo -e "${YELLOW}Airdrop may have failed, continuing anyway...${NC}"
    sleep 2
fi

# Deploy the program
echo ""
echo "Deploying program..."
echo "  Path: $PROGRAM_PATH"
echo "  RPC: $RPC_URL"
echo ""

OUTPUT=$(solana program deploy $PROGRAM_PATH --url $RPC_URL 2>&1)
echo "$OUTPUT"

# Extract program ID from output
PROGRAM_ID=$(echo "$OUTPUT" | grep -oP 'Program Id: \K[A-Za-z0-9]+' || true)

if [ -z "$PROGRAM_ID" ]; then
    # Try alternative format
    PROGRAM_ID=$(echo "$OUTPUT" | grep -oP 'program id: \K[A-Za-z0-9]+' || true)
fi

if [ -z "$PROGRAM_ID" ]; then
    echo -e "${RED}Error: Could not extract program ID from deployment output${NC}"
    exit 1
fi

# Save program ID
echo "$PROGRAM_ID" > $PROGRAM_ID_FILE

echo ""
echo -e "${GREEN}✓ Deployment successful!${NC}"
echo ""
echo "Program ID: $PROGRAM_ID"
echo "Saved to: $PROGRAM_ID_FILE"
echo ""
echo "Verify deployment:"
echo "  solana program show $PROGRAM_ID --url $RPC_URL"
echo ""
echo "Next steps:"
echo "  ./scripts/test-native.sh"
