# hate.fun Quick Start Guide

Get up and running with hate.fun in 5 minutes using native tools (no Docker required).

## Prerequisites

Run the setup checker to verify prerequisites:

```bash
./scripts/setup-native.sh
```

Required tools:
- **Rust & Cargo** - For building the program
- **Solana CLI** - For deployment and interaction
- **cargo-build-sbf** - For building Solana programs (part of Agave toolchain)

## Steps

### 1. Check Prerequisites

```bash
./scripts/setup-native.sh
```

This will check what's installed and provide installation instructions for missing tools.

### 2. Build the Program

```bash
./scripts/build-native.sh
```

Output: `dist/program/hate_fun.so` (~32KB)

### 3. Start Local Test Validator

```bash
./scripts/start-validator.sh
```

Wait for: `✅ Validator is ready!`

### 4. Deploy to Validator

```bash
./scripts/deploy-native.sh
```

Output: Program ID saved to `.program-id`

### 5. Run Tests

```bash
./scripts/test-native.sh
```

This runs:
- Unit tests (5/5 passing)
- Verifies deployment
- Shows program info

### 6. Run Integration Tests (Optional)

```bash
cargo test --test integration_client -- --ignored --nocapture
```

This runs comprehensive integration tests:
- ✅ test_create_bucket
- ✅ test_deposit_and_flush
- ✅ test_full_flow (multiple flips)
- ✅ test_close_bucket_before_flip
- ✅ test_validation_fees_too_high

All 5 tests should pass!

## Stop Everything

```bash
./scripts/stop-validator.sh
```

## Connection Info

- **RPC Endpoint**: `http://localhost:8899`
- **WebSocket**: `ws://localhost:8900`
- **Network**: Local test validator (not devnet/mainnet)

## Creating Your First Bucket

You can use the integration test client as a reference for building transactions.

### Instruction: `create_bucket` (discriminator: 0)

**Accounts** (in order):
1. Payer (signer, writable)
2. Bucket PDA (writable)
3. Main bucket PDA (writable)
4. Escrow A PDA (writable)
5. Escrow B PDA (writable)
6. System program

**Data** (142 bytes):
- [0..32] address_a (Pubkey)
- [32..64] address_b (Pubkey)
- [64..96] creator_address (Pubkey)
- [96..98] creator_fee_bps (u16, 0-2000)
- [98..100] claimer_fee_bps (u16, 0-2000)
- [100..108] initial_last_swap (u64, min 100,000)
- [108..110] min_increase_bps (u16, 100-5000)
- [110..142] seed (32 bytes)

**Example parameters:**
```rust
address_a: Keypair::generate().pubkey()
address_b: Keypair::generate().pubkey()
creator_address: wallet.pubkey()
creator_fee_bps: 500       // 5%
claimer_fee_bps: 50        // 0.5%
initial_last_swap: 1_000_000_000  // 1 SOL
min_increase_bps: 500      // 5%
seed: rand::random()       // 32 random bytes
```

See `tests/integration_client.rs` for complete implementation examples.

## Troubleshooting

### Prerequisites missing

Run:
```bash
./scripts/setup-native.sh
```

Follow the installation instructions provided.

### Validator won't start

```bash
# Check if another validator is running
pgrep -f test-validator

# Stop existing validator
./scripts/stop-validator.sh

# Check logs
tail -f .validator-logs/validator.log
```

### Build fails

```bash
# Verify cargo-build-sbf is installed
which cargo-build-sbf

# If missing, install Agave:
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

### Deployment fails

```bash
# Check validator is running
solana cluster-version --url http://localhost:8899

# Check balance
solana balance --url http://localhost:8899

# Request airdrop if needed
solana airdrop 10 --url http://localhost:8899

# Check binary exists
ls -lh dist/program/hate_fun.so
```

## Full Documentation

- **[NATIVE-TESTING.md](NATIVE-TESTING.md)** - Complete native testing guide
- **[INTEGRATION_TESTS.md](INTEGRATION_TESTS.md)** - Integration test documentation
- **[spell.md](spell.md)** - Technical specification
- **[README.md](README.md)** - Project overview

## Need Help?

1. Check [NATIVE-TESTING.md](NATIVE-TESTING.md) for detailed troubleshooting
2. Review logs: `tail -f .validator-logs/validator.log`
3. Verify validator health: `solana cluster-version --url http://localhost:8899`
4. Check program info: `solana program show $(cat .program-id) --url http://localhost:8899`

## What's Next?

- Study the integration tests in `tests/integration_client.rs`
- Try modifying test parameters
- Build your own client SDK
- Deploy to devnet for more realistic testing

## Development Workflow

```bash
# Make changes to code
vim src/instructions/some_file.rs

# Rebuild
./scripts/build-native.sh

# Redeploy (validator must be running)
solana program deploy dist/program/hate_fun.so --program-id dist/program/hate_fun-keypair.json

# Test
cargo test --test integration_client -- --ignored --nocapture
```

Happy hacking! 🚀
