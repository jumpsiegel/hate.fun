# Integration Tests Guide

This document explains how to run the Rust integration tests for hate.fun.

## Prerequisites

1. **Local validator running:**
   ```bash
   ./scripts/start-validator.sh
   ```

2. **Program deployed:**
   ```bash
   ./scripts/deploy-native.sh
   ```

3. **Dependencies installed:**
   - Cargo automatically installs when you run tests

## Running Tests

### Run all integration tests
```bash
cargo test --test integration_client -- --ignored --nocapture
```

### Run individual tests

**Create bucket:**
```bash
cargo test --test integration_client test_create_bucket -- --ignored --nocapture
```

**Deposit and flush:**
```bash
cargo test --test integration_client test_deposit_and_flush -- --ignored --nocapture
```

**Full flow (multiple flips):**
```bash
cargo test --test integration_client test_full_flow -- --ignored --nocapture
```

**Validation (fees too high):**
```bash
cargo test --test integration_client test_validation_fees_too_high -- --ignored --nocapture
```

**Close bucket:**
```bash
cargo test --test integration_client test_close_bucket_before_flip -- --ignored --nocapture
```

## Test Structure

The integration tests are located in `tests/integration_client.rs` and include:

### Helper Functions

- `get_program_id()` - Loads program ID from `.program-id` file
- `derive_*_pda()` - Functions to derive all PDAs
- `*_instruction()` - Instruction builders for all 5 instructions

### Test Cases

1. **test_create_bucket** - Tests bucket creation
2. **test_deposit_and_flush** - Tests deposit and flush flow
3. **test_full_flow** - Tests multiple competitive flips
4. **test_close_bucket_before_flip** - Tests closing before first flip (currently failing)
5. **test_validation_fees_too_high** - Tests fee validation

## Understanding Test Output

Tests use `--nocapture` flag to show detailed output:

```
=== Testing: Create Bucket ===

Airdropping 5 SOL to <address>
Bucket PDA: <address>
Main Bucket PDA: <address>
Escrow A PDA: <address>
Escrow B PDA: <address>

✓ Bucket created successfully!
Signature: <tx_signature>
Bucket account size: 159 bytes
```

## Test Results

Current status: **4 out of 5 tests passing**

✅ Passing:
- create_bucket
- deposit_and_flush
- full_flow
- validation_fees_too_high

⚠️ Failing:
- close_bucket_before_flip (known bug in program logic)

## Known Issues

### Issue: close_bucket_before_flip fails

**Error:** `EscrowsNotEmpty (0x8)`

**Cause:** When escrows are created as PDAs, they receive rent-exempt lamports (~0.89 SOL). The `close_bucket` instruction checks for exactly 0 lamports, but escrows never reach 0 immediately after creation.

**Fix needed:** Update `src/instructions/close_bucket.rs` to check if escrow balances equal the rent-exempt minimum instead of 0.

## Adding New Tests

To add a new test:

1. Add function to `tests/integration_client.rs`:
   ```rust
   #[test]
   #[ignore]
   fn test_your_scenario() {
       println!("\n=== Testing: Your Scenario ===\n");

       let client = setup_client();
       let program_id = get_program_id();

       // Your test code here
   }
   ```

2. Run with:
   ```bash
   cargo test --test integration_client test_your_scenario -- --ignored --nocapture
   ```

## Troubleshooting

### Validator not running
```bash
# Start validator
./scripts/start-validator.sh

# Check status
solana cluster-version --url http://localhost:8899
```

### Program ID file not found
```bash
# Deploy program
./scripts/deploy-native.sh

# Verify deployment
cat .program-id
```

### Airdrop failures
This is normal on local validator - tests will continue anyway.

### Tests timing out
Increase timeout in test or check validator logs:
```bash
tail -f .validator-logs/validator.log
```

## Performance Notes

- Tests run sequentially to avoid conflicts
- Each test creates new accounts with airdrops
- Expected runtime: ~15-20 seconds for all tests
- Each instruction uses 9,000-14,000 compute units

## Next Steps

Future tests to implement:

1. **claim_payout test** - Requires epoch advancement (complex)
2. **Insufficient balance test** - Try to flush below threshold
3. **Creator validation test** - Verify creator != address_a/b
4. **Min increase bounds test** - Test 1% and 50% limits
5. **Multiple deposits test** - Multiple deposits before flush

## Resources

- Full test results: `TEST_RESULTS.md`
- Program specification: `spell.md`
- Testing guide: `TESTING.md`
- Native setup: `NATIVE-TESTING.md`
