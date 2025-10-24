# Testing Guide for hate.fun (Hate Bucket) Program

## Testing Approach

The hate.fun program uses a comprehensive testing strategy:

### 1. Unit Tests ✅

Located in `tests/integration_test.rs`, these test:
- Threshold calculations
- Fee calculations
- Validation logic (bounds checking)
- Mathematical operations

**Run unit tests:**
```bash
cargo test
```

**Status:** ✅ 5/5 passing

### 2. Integration Tests ✅

Located in `tests/integration_client.rs`, these test the full program flow on a local validator:
- Bucket creation with PDAs
- Deposit and flush operations
- Multi-flip escalation scenarios
- Fee validation
- Close bucket functionality (including bug fix)

**Run integration tests:**
```bash
# Start validator first
./scripts/start-validator.sh

# Deploy program
./scripts/deploy-native.sh

# Run tests
cargo test --test integration_client -- --ignored --nocapture
```

**Status:** ✅ 5/5 passing (including recent bug fix for close_bucket)

## Building the Program

```bash
# Build for deployment
./scripts/build.sh

# Or manually:
cargo build-sbf --manifest-path=Cargo.toml --sbf-out-dir=dist/program
```

The compiled `.so` file will be in `dist/program/` as `hate_fun.so`.

## Testing on Local Validator

### 1. Start Local Validator

```bash
solana-test-validator
```

### 2. Deploy Program

```bash
solana program deploy dist/program/hate_fun.so
```

Note the program ID from the deployment output.

### 3. Test Scenarios

#### Create a Bucket

```bash
# Coming soon: CLI tool or TypeScript client
```

#### Test Flow: Create → Deposit → Flush → Claim

1. Create bucket with addresses A and B
2. Deposit to escrow A (below threshold) - should NOT flip
3. Deposit more to reach threshold
4. Flush escrow A - should flip target to B
5. Wait 3 epochs (~6-9 days on mainnet, configurable on local validator)
6. Claim payout - should distribute to creator, claimer, winner

## Test Scenarios Covered

### ✅ Unit Tests (Automated)

- [x] Threshold calculation (5% minimum increase)
- [x] Fee calculation (creator + claimer)
- [x] Fee validation (max 20% combined)
- [x] Min increase bounds (1-50%)
- [x] Initial swap minimum (0.0001 SOL)

### ✅ Integration Tests (Automated via tests/integration_client.rs)

- [x] **test_create_bucket**: Creates bucket with all PDAs ✅
- [x] **test_deposit_and_flush**: Deposit → flush → verify balance transfers ✅
- [x] **test_full_flow**: Multiple flips, pot escalation (1 SOL → 1.10 SOL → 2.30 SOL) ✅
- [x] **test_validation_fees_too_high**: Correctly rejects fees > 20% ✅
- [x] **test_close_bucket_before_flip**: Close bucket and recover rent (bug fixed!) ✅

### ⚠️ Additional Test Scenarios (Not Yet Implemented)

- [ ] **Validation: Creator same as address A/B**
- [ ] **Validation: Min increase out of bounds**
- [ ] **Validation: Initial swap too low**
- [ ] **Flush below threshold** - should fail
- [ ] **Flush at exact threshold** - should succeed
- [ ] **Close after flip** - should fail (bucket has flips)
- [ ] **Close with non-empty escrows** - should fail
- [ ] **Claim before 3 epochs** - should fail
- [ ] **Claim after 3 epochs** - distributes correctly (requires epoch advancement)
- [ ] **Direct deposits to main_bucket** - included in payout

## Testing Checklist

Before deployment to mainnet:

- [x] All unit tests passing (5/5) ✅
- [x] Program builds successfully ✅
- [x] Integration tests passing (5/5) ✅
- [x] Happy path tested end-to-end ✅
- [x] Bug fixes verified (close_bucket rent-exempt handling) ✅
- [ ] Deployed and tested on devnet
- [ ] Additional edge cases tested
- [ ] All validation tests passed
- [ ] Edge cases verified
- [ ] Fee calculations verified with real transactions
- [ ] Epoch timeout verified
- [ ] Security audit completed (recommended)

## Known Limitations

- ✅ ~~No automated integration tests~~ - **RESOLVED**: Full integration test suite implemented in `tests/integration_client.rs`
- Epoch advancement requires actual time or validator config changes (affects claim_payout testing)
- Some edge case scenarios still need manual testing

## Testing Resources

- **[INTEGRATION_TESTS.md](INTEGRATION_TESTS.md)** - Complete integration test documentation
- **[NATIVE-TESTING.md](NATIVE-TESTING.md)** - Native testing setup guide
- **tests/integration_client.rs** - Integration test source code (reference for building clients)

## Future Testing Improvements

- [ ] Test claim_payout with epoch advancement
- [ ] Additional validation edge cases
- [ ] TypeScript client SDK with test suite
- [ ] Test fixtures for common scenarios
- [ ] Automated devnet deployment and testing pipeline
- [ ] Property-based testing for mathematical operations
