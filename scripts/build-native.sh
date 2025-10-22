#!/bin/bash
# Native build script for hate.fun Solana program
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Building hate.fun Solana program..."

# Check if cargo-build-sbf is available
if ! command -v cargo-build-sbf >/dev/null 2>&1; then
    echo -e "${RED}Error: cargo-build-sbf not found${NC}"
    echo ""
    echo "cargo-build-sbf is required to build Solana programs."
    echo "It's part of the Agave toolchain."
    echo ""
    echo "To install, run:"
    echo "  ./scripts/setup-native.sh"
    echo ""
    exit 1
fi

# Create output directory
mkdir -p dist/program

# Build the program
echo "Running: cargo build-sbf --manifest-path=Cargo.toml --sbf-out-dir=dist/program"
cargo build-sbf --manifest-path=Cargo.toml --sbf-out-dir=dist/program

echo ""
echo -e "${GREEN}✓ Build successful!${NC}"
echo ""
echo "Program binary: dist/program/hate_fun.so"

# Show file info
if [ -f "dist/program/hate_fun.so" ]; then
    FILE_SIZE=$(du -h dist/program/hate_fun.so | cut -f1)
    echo "File size: $FILE_SIZE"
fi

echo ""
echo "Next steps:"
echo "  1. Start validator: ./scripts/start-validator.sh"
echo "  2. Deploy program: ./scripts/deploy-native.sh"
