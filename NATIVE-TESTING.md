# Native Testing Guide (No Docker/Podman)

This guide explains how to test hate.fun natively on your local machine without Docker or Podman.

## Prerequisites

You need the following tools installed:

1. **Rust & Cargo** - For building the program
2. **Solana CLI** - For interacting with Solana
3. **Agave/Solana tools** - For building and running a test validator
   - `cargo-build-sbf` - Builds Solana programs
   - `agave-validator` or `solana-test-validator` - Runs local validator

### Quick Setup Check

Run the setup script to check what's installed and get installation instructions:

```bash
./scripts/setup-native.sh
```

This will check your environment and provide specific installation instructions for any missing tools.

## Installation Instructions

### Install Rust (if needed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Install Solana CLI (if needed)

```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

Add to PATH (add to your ~/.bashrc or ~/.zshrc):
```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

### Install Agave (includes cargo-build-sbf and validator)

**Option 1 - From release (recommended):**
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

**Option 2 - From crates.io:**
```bash
cargo install agave-install
agave-install init
```

After installation, ensure `~/.local/share/solana/install/active_release/bin` is in your PATH.

### Verify Installation

```bash
./scripts/setup-native.sh
```

You should see all green checkmarks ✓.

## Workflow

### 1. Build the Program

```bash
./scripts/build-native.sh
```

This compiles the Solana program using `cargo-build-sbf` and outputs `dist/program/hate_fun.so`.

**Output:**
- Binary: `dist/program/hate_fun.so` (~20KB)
- Build artifacts in `target/deploy/`

### 2. Start Local Test Validator

```bash
./scripts/start-validator.sh
```

This starts a local Solana test validator with:
- RPC endpoint: `http://localhost:8899`
- WebSocket: `ws://localhost:8900`
- Ledger data: `.validator-ledger/`
- Logs: `.validator-logs/validator.log`

The validator runs in the background. Check status:
```bash
solana cluster-version --url http://localhost:8899
```

View logs:
```bash
tail -f .validator-logs/validator.log
```

### 3. Deploy the Program

```bash
./scripts/deploy-native.sh
```

This:
1. Verifies the validator is running
2. Airdrops SOL to your wallet if needed
3. Deploys the program to the local validator
4. Saves the program ID to `.program-id`

**Program ID** is saved in `.program-id` for reference.

### 4. Run Tests

```bash
./scripts/test-native.sh
```

This:
1. Verifies the validator and program are accessible
2. Runs unit tests with `cargo test`
3. Shows program info
4. Provides guidance for integration testing

### 5. Stop the Validator

```bash
./scripts/stop-validator.sh
```

Stops the background validator process.

To clean up all validator data:
```bash
rm -rf .validator-ledger .validator-logs
```

## Testing Workflow

### Current Status

✅ **Unit tests** - Implemented and passing
- Threshold calculations
- Fee calculations
- Validation logic

⚠️ **Integration tests** - Require client SDK

The program is deployed and ready for testing, but full end-to-end tests require a client SDK to build and send transactions.

### Integration Testing Options

#### Option 1: TypeScript Client (Recommended)

Create a TypeScript client using `@solana/web3.js`:

```bash
mkdir tests/client
cd tests/client
npm init -y
npm install @solana/web3.js
```

Example test structure:
```typescript
import { Connection, Keypair, PublicKey } from '@solana/web3.js';

const connection = new Connection('http://localhost:8899', 'confirmed');
const programId = new PublicKey(fs.readFileSync('../../.program-id', 'utf-8').trim());

// Test: Create bucket
async function testCreateBucket() {
  const creator = Keypair.generate();
  const addressA = Keypair.generate().publicKey;
  const addressB = Keypair.generate().publicKey;

  // Build create_bucket instruction
  // Send transaction
  // Verify bucket created
}
```

#### Option 2: Rust Client

Add to your `Cargo.toml`:
```toml
[dev-dependencies]
solana-sdk = "2.0"
solana-client = "2.0"
```

Create integration tests in `tests/integration_native.rs`:
```rust
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::Keypair, transaction::Transaction};

#[test]
fn test_create_bucket() {
    let client = RpcClient::new("http://localhost:8899");
    // Build and send transactions
}
```

#### Option 3: Manual CLI Testing

You can manually test using Solana CLI commands, though this is more tedious:

```bash
# Create test keypairs
solana-keygen new --outfile test-creator.json
solana-keygen new --outfile test-address-a.json
solana-keygen new --outfile test-address-b.json

# Fund accounts
solana airdrop 10 test-creator.json --url http://localhost:8899

# Build and send raw transactions (requires manual instruction building)
```

## Test Scenarios

See [TESTING.md](TESTING.md) for comprehensive test scenarios to implement.

Key scenarios:
- ✓ Unit tests (threshold, fees, validation)
- ⧗ Create bucket with various parameters
- ⧗ Deposit to escrow accounts
- ⧗ Flush escrow (flip control)
- ⧗ Multiple flips (escalation)
- ⧗ Wait 3 epochs and claim payout
- ⧗ Close bucket before first flip
- ⧗ Validation edge cases

## Troubleshooting

### Build fails with "cargo-build-sbf not found"

Install Agave toolchain:
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

Ensure `~/.local/share/solana/install/active_release/bin` is in PATH.

### Validator won't start

Check if another validator is running:
```bash
pgrep -f test-validator
```

Kill existing validator:
```bash
./scripts/stop-validator.sh
```

Check logs for errors:
```bash
cat .validator-logs/validator.log
```

### Deploy fails - "Insufficient funds"

The deploy script should auto-airdrop SOL. If it fails, manually airdrop:
```bash
solana airdrop 10 --url http://localhost:8899
```

### Cannot connect to validator

Verify validator is running:
```bash
solana cluster-version --url http://localhost:8899
```

Check validator logs:
```bash
tail -f .validator-logs/validator.log
```

### Program not found after deployment

Check if program ID file exists:
```bash
cat .program-id
```

Verify program is deployed:
```bash
solana program show $(cat .program-id) --url http://localhost:8899
```

## Comparison with Docker

### Native Advantages
✅ Faster builds (no container overhead)
✅ Direct access to all tools
✅ Easier debugging
✅ No Docker installation required

### Docker Advantages
✅ Consistent environment across machines
✅ No tool installation needed
✅ Easy cleanup (just remove containers)

Choose based on your preference and environment.

## Files Created

Native testing creates these files/directories:

```
.validator-ledger/     # Validator blockchain data (can be deleted)
.validator-logs/       # Validator logs (can be deleted)
.validator-pid         # Validator process ID
.program-id            # Deployed program ID
dist/program/          # Compiled program binary
```

Add to `.gitignore`:
```
.validator-ledger/
.validator-logs/
.validator-pid
.program-id
```

## Next Steps

1. ✅ Environment setup complete
2. ✅ Program built and deployed
3. ✅ Unit tests passing
4. ⧗ Build client SDK (TypeScript or Rust)
5. ⧗ Implement integration tests
6. ⧗ Test all scenarios from TESTING.md

## Resources

- [Solana CLI Documentation](https://docs.solana.com/cli)
- [Solana Web3.js](https://solana-labs.github.io/solana-web3.js/)
- [Agave Validator](https://github.com/anza-xyz/agave)
- [hate.fun Technical Spec](spell.md)

## Quick Reference

```bash
# Setup
./scripts/setup-native.sh           # Check prerequisites

# Build & Deploy
./scripts/build-native.sh           # Build program
./scripts/start-validator.sh        # Start validator
./scripts/deploy-native.sh          # Deploy program
./scripts/test-native.sh            # Run tests

# Manage Validator
./scripts/stop-validator.sh         # Stop validator
tail -f .validator-logs/validator.log  # View logs

# Cleanup
rm -rf .validator-ledger .validator-logs .validator-pid .program-id

# Solana Commands
solana cluster-version --url http://localhost:8899
solana program show $(cat .program-id) --url http://localhost:8899
solana balance --url http://localhost:8899
solana epoch-info --url http://localhost:8899
```
