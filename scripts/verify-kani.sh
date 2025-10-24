#!/bin/bash

# Script to run Kani formal verification on hate.fun

set -e

echo "========================================"
echo "   Kani Formal Verification"
echo "========================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Kani is installed
if ! command -v cargo-kani &> /dev/null; then
    echo -e "${RED}✗ Kani is not installed${NC}"
    echo ""
    echo "Install Kani with:"
    echo "  cargo install --locked kani-verifier"
    echo "  cargo kani setup"
    echo ""
    echo "See KANI-VERIFICATION.md for full installation instructions."
    exit 1
fi

echo -e "${GREEN}✓ Kani is installed${NC}"
echo "Version: $(cargo kani --version)"
echo ""

# Parse arguments
HARNESS=""
VISUALIZE=""
VERBOSE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --harness)
            HARNESS="--harness $2"
            shift 2
            ;;
        --visualize)
            VISUALIZE="--visualize"
            shift
            ;;
        --verbose)
            VERBOSE="--verbose"
            shift
            ;;
        --list)
            echo "Available proof harnesses:"
            echo ""
            grep -n "#\[kani::proof\]" src/verification.rs -A 1 | grep "fn " | sed 's/.*fn /  - /' | sed 's/(.*)//'
            exit 0
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --harness NAME    Run specific proof harness"
            echo "  --visualize       Generate HTML visualization"
            echo "  --verbose         Show detailed output"
            echo "  --list            List available harnesses"
            echo "  --help            Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                                    # Run all proofs"
            echo "  $0 --harness verify_threshold_calculation"
            echo "  $0 --visualize --harness verify_payout_distribution_conservation"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

if [ -n "$HARNESS" ]; then
    echo "Running proof harness: $(echo $HARNESS | sed 's/--harness //')"
else
    echo "Running all proof harnesses..."
fi
echo ""

# Run Kani
echo "========================================"
echo "   Starting Verification"
echo "========================================"
echo ""

if cargo kani --tests $HARNESS $VISUALIZE $VERBOSE; then
    echo ""
    echo "========================================"
    echo -e "${GREEN}✓ All verifications passed!${NC}"
    echo "========================================"
    echo ""
    echo "Verified properties:"
    echo "  ✓ Threshold calculations never overflow"
    echo "  ✓ Payout distribution conserves total value"
    echo "  ✓ Fee validation enforces 20% limit"
    echo "  ✓ Min increase validation enforces 1-50% bounds"
    echo "  ✓ Threshold precision maintains guarantees"
    echo "  ✓ HF-01 vulnerability documented"
    echo "  ✓ Balance summation handles realistic values"
    echo "  ✓ Maximum fee calculations are safe"
    echo ""

    if [ -n "$VISUALIZE" ]; then
        echo "Visualization generated. Look for HTML files in the output."
    fi

    exit 0
else
    echo ""
    echo "========================================"
    echo -e "${RED}✗ Verification failed!${NC}"
    echo "========================================"
    echo ""
    echo "Kani found a counterexample. Review the output above for:"
    echo "  - Which assertion failed"
    echo "  - Concrete values that triggered the failure"
    echo "  - Stack trace to the failing code"
    echo ""
    echo "See KANI-VERIFICATION.md for troubleshooting guidance."
    exit 1
fi
