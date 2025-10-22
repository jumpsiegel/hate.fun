# Testing Guide for hate.fun (Hate Bucket) Program

## Testing Approach

Due to Pinocchio's use of custom types (`[u8; 32]` for `Pubkey`, custom `AccountInfo`, etc.) that differ from `solana-program`, the program is **incompatible with `solana-program-test`** for direct processor testing.

Instead, we use a hybrid testing approach:

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

### 2. End-to-End Testing (Manual)

For full integration testing, deploy the program to a local validator or devnet and interact via client SDK.

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

### 📋 Manual E2E Tests (via Deployment)

- [ ] **Happy path**: create → deposit → flush → claim
- [ ] **Validation: Fees too high** (>20%)
- [ ] **Validation: Creator same as address A/B**
- [ ] **Validation: Min increase out of bounds**
- [ ] **Validation: Initial swap too low**
- [ ] **Flush below threshold** - should fail
- [ ] **Flush at exact threshold** - should succeed
- [ ] **Flush above threshold** - transfers entire balance
- [ ] **Multiple flips** - pot grows each time
- [ ] **Close before flip** - creator recovers funds
- [ ] **Close after flip** - should fail
- [ ] **Close with non-empty escrows** - should fail
- [ ] **Claim before 3 epochs** - should fail
- [ ] **Claim after 3 epochs** - distributes correctly
- [ ] **Direct deposits to main_bucket** - included in payout

## Testing Checklist

Before deployment to mainnet:

- [ ] All unit tests passing
- [ ] Program builds successfully
- [ ] Deployed and tested on devnet
- [ ] Happy path tested end-to-end
- [ ] All validation tests passed
- [ ] Edge cases verified
- [ ] Fee calculations verified with real transactions
- [ ] Epoch timeout verified
- [ ] Security audit completed (recommended)

## Known Limitations

- No automated integration tests due to Pinocchio/solana-program incompatibility
- Epoch advancement requires actual time or validator config changes
- Client SDK needed for comprehensive testing (in development)

## Future Testing Improvements

- [ ] TypeScript client SDK with test suite
- [ ] Rust client with integration tests using deployed program
- [ ] Test fixtures for common scenarios
- [ ] Automated devnet deployment and testing pipeline
- [ ] Property-based testing for mathematical operations
