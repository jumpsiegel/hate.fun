#!/bin/bash
# Setup script for native (non-Docker) hate.fun development
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================"
echo "hate.fun Native Development Setup"
echo "========================================"
echo ""

# Check for required tools
echo "Checking for required tools..."
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Track what's missing
MISSING_TOOLS=()

# Check Rust/Cargo
if command_exists cargo; then
    CARGO_VERSION=$(cargo --version)
    echo -e "${GREEN}✓${NC} Cargo installed: $CARGO_VERSION"
else
    echo -e "${RED}✗${NC} Cargo not found"
    MISSING_TOOLS+=("cargo")
fi

# Check Solana CLI
if command_exists solana; then
    SOLANA_VERSION=$(solana --version)
    echo -e "${GREEN}✓${NC} Solana CLI installed: $SOLANA_VERSION"
else
    echo -e "${RED}✗${NC} Solana CLI not found"
    MISSING_TOOLS+=("solana")
fi

# Check for cargo-build-sbf
if command_exists cargo-build-sbf; then
    echo -e "${GREEN}✓${NC} cargo-build-sbf installed"
else
    echo -e "${YELLOW}✗${NC} cargo-build-sbf not found (needed for building Solana programs)"
    MISSING_TOOLS+=("cargo-build-sbf")
fi

# Check for agave-validator or solana-test-validator
if command_exists agave-validator; then
    echo -e "${GREEN}✓${NC} agave-validator installed"
elif command_exists solana-test-validator; then
    echo -e "${GREEN}✓${NC} solana-test-validator installed"
else
    echo -e "${YELLOW}✗${NC} Test validator not found (needed for local testing)"
    MISSING_TOOLS+=("test-validator")
fi

echo ""

# If nothing is missing, we're good
if [ ${#MISSING_TOOLS[@]} -eq 0 ]; then
    echo -e "${GREEN}All required tools are installed!${NC}"
    echo ""
    echo "You're ready to:"
    echo "  1. Build: ./scripts/build-native.sh"
    echo "  2. Start validator: ./scripts/start-validator.sh"
    echo "  3. Deploy: ./scripts/deploy-native.sh"
    exit 0
fi

# Show what's missing and how to install
echo -e "${YELLOW}Missing tools detected. Installation instructions:${NC}"
echo ""

for tool in "${MISSING_TOOLS[@]}"; do
    case $tool in
        cargo)
            echo "━━━ Install Rust and Cargo ━━━"
            echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
            echo ""
            ;;
        solana)
            echo "━━━ Install Solana CLI ━━━"
            echo "  Visit: https://docs.solana.com/cli/install-solana-cli-tools"
            echo "  Or run: sh -c \"\$(curl -sSfL https://release.solana.com/stable/install)\""
            echo ""
            ;;
        cargo-build-sbf|test-validator)
            echo "━━━ Install Agave (includes cargo-build-sbf and agave-validator) ━━━"
            echo "  Agave is the Solana validator client and includes build tools."
            echo ""
            echo "  Option 1 - Install from release:"
            echo "    sh -c \"\$(curl -sSfL https://release.anza.xyz/stable/install)\""
            echo ""
            echo "  Option 2 - Install from crates.io:"
            echo "    cargo install agave-install"
            echo "    agave-install init"
            echo ""
            echo "  After installation, ensure ~/.local/share/solana/install/active_release/bin"
            echo "  is in your PATH"
            echo ""
            break
            ;;
    esac
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "After installing missing tools, run this script again to verify."
echo ""

exit 1
