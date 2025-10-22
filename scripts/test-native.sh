#!/bin/bash
# Test hate.fun program on local validator
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

RPC_URL="http://localhost:8899"
PROGRAM_ID_FILE=".program-id"

echo "========================================"
echo "hate.fun Native Testing"
echo "========================================"
echo ""

# Check if program is deployed
if [ ! -f "$PROGRAM_ID_FILE" ]; then
    echo -e "${RED}Error: Program ID file not found: $PROGRAM_ID_FILE${NC}"
    echo ""
    echo "Deploy the program first:"
    echo "  ./scripts/deploy-native.sh"
    echo ""
    exit 1
fi

PROGRAM_ID=$(cat $PROGRAM_ID_FILE)

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

# Verify program is deployed
echo "Verifying program deployment..."
if solana program show $PROGRAM_ID --url $RPC_URL >/dev/null 2>&1; then
    echo -e "${GREEN}✓ Program is deployed${NC}"
    echo ""
    solana program show $PROGRAM_ID --url $RPC_URL
else
    echo -e "${RED}Error: Program not found at $PROGRAM_ID${NC}"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run unit tests
echo -e "${BLUE}Running unit tests...${NC}"
echo ""
cargo test
echo ""
echo -e "${GREEN}✓ Unit tests passed${NC}"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Show what integration tests would look like
echo -e "${YELLOW}Integration Testing Status${NC}"
echo ""
echo "The program is deployed and ready for integration testing."
echo ""
echo "To perform end-to-end tests, you need a client SDK to:"
echo "  1. Generate keypairs for test accounts"
echo "  2. Build and sign transactions"
echo "  3. Send instructions to the program"
echo "  4. Parse and verify account data"
echo ""
echo "Recommended next steps:"
echo ""
echo "  Option 1 - TypeScript Client:"
echo "    npm install @solana/web3.js"
echo "    # Write client in TypeScript using @solana/web3.js"
echo ""
echo "  Option 2 - Rust Client:"
echo "    # Add solana-sdk and solana-client to Cargo.toml"
echo "    # Write integration tests using deployed program"
echo ""
echo "  Option 3 - Manual Testing with Solana CLI:"
echo "    # You can manually craft transactions using solana CLI"
echo ""
echo "Test scenarios to implement:"
echo "  ✓ Unit tests (DONE)"
echo "  ⧗ Create bucket"
echo "  ⧗ Deposit to escrow"
echo "  ⧗ Flush escrow (flip control)"
echo "  ⧗ Wait 3 epochs"
echo "  ⧗ Claim payout"
echo "  ⧗ Close bucket"
echo "  ⧗ Validation edge cases"
echo ""
echo "See TESTING.md for detailed test scenarios."
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}Testing environment is ready!${NC}"
echo ""
echo "Program ID: $PROGRAM_ID"
echo "RPC URL: $RPC_URL"
echo ""
