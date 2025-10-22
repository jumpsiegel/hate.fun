# Claude Code Session Context - hate.fun Project

## Project Status

✅ **IMPLEMENTATION COMPLETE** - All 5 instructions implemented and compiling cleanly without warnings.

✅ **NATIVE TESTING INFRASTRUCTURE READY** - Full native testing setup without Docker/Podman.

We have completed the full design phase, core implementation, and native testing infrastructure for a hate-driven fundraising mechanism on Solana. The program builds cleanly and is ready for deployment and integration testing.

## What We Built

A Solana program that creates competitive bidding wars between two opposing parties (Republicans vs Democrats, pro-life vs pro-choice, sports rivals, etc.). The system uses game theory and tribal psychology to maximize capital commitment through escalating competition.

## Implementation Progress

### ✅ Phase 1: Project Setup (COMPLETE)
- Initialized Rust/Cargo project with `cargo init --lib`
- Added Pinocchio v0.9 dependency
- Configured Solana program build settings (cdylib, release optimizations)
- Created project directory structure (src/, tests/, examples/, scripts/)
- Created build script at `scripts/build.sh`

### ✅ Phase 2: Core Program Structure (COMPLETE)
- Defined Bucket account structure (159 bytes)
- Implemented PDA derivation functions for all 4 PDAs
- Set up instruction enum with discriminators (0-4)
- Created instruction data parsing helpers
- Implemented custom error types (11 error variants)

### ✅ Phase 3: All 5 Instructions (COMPLETE)
- ✅ create_bucket - Full validation and PDA creation
- ✅ deposit_to_escrow - Transfer via System Program CPI
- ✅ flush_escrow - Threshold checking and state updates
- ✅ claim_payout - Fee distribution after 3 epochs
- ✅ close_bucket - Creator recovery before first flip

## File Structure

```
hate.fun/
├── Cargo.toml                       # Pinocchio 0.9, cdylib config, features
├── src/
│   ├── lib.rs                       # Entrypoint
│   ├── state.rs                     # Bucket struct + PDA helpers
│   ├── error.rs                     # Custom error types
│   ├── system_program.rs            # CPI helpers for create/transfer
│   └── instructions/
│       ├── mod.rs                   # Router + parsing helpers
│       ├── create_bucket.rs         # Instruction 0
│       ├── deposit_to_escrow.rs     # Instruction 1
│       ├── flush_escrow.rs          # Instruction 2
│       ├── claim_payout.rs          # Instruction 3
│       └── close_bucket.rs          # Instruction 4
├── scripts/
│   ├── build.sh                     # Original build wrapper
│   ├── setup-native.sh              # Check prerequisites & install guide
│   ├── build-native.sh              # Native build script
│   ├── start-validator.sh           # Start local test validator
│   ├── stop-validator.sh            # Stop validator
│   ├── deploy-native.sh             # Deploy to local validator
│   └── test-native.sh               # Run tests on local validator
├── tests/
│   └── integration_test.rs          # Unit tests
├── spell.md                         # Technical specification
├── todo.md                          # Implementation roadmap
├── README.md                        # Project overview & usage
├── TESTING.md                       # Manual testing guide
├── NATIVE-TESTING.md                # Native testing guide (NEW)
├── DOCKER.md                        # Docker testing guide (DEPRECATED)
├── QUICKSTART.md                    # Quick start guide
└── claude.md                        # This file

```

## Key Design Decisions Made

### Architecture
- **Factory pattern**: Anyone can create independent "hate buckets"
- **Fully immutable**: After creation, no parameter updates allowed
- **Permissionless**: Anyone can deposit, flush, or claim

### Economic Model
- **5% minimum increase**: Each flip must exceed previous by at least 5%
- **Full escrow flush**: Entire escrow balance transfers (not just minimum)
- **Creator fee**: Configurable (up to 20% combined with claimer fee)
- **Claimer fee**: Incentivizes anyone to trigger final payout after 3 epochs
- **Winner takes all**: Current target gets remaining funds after fees

### Game Mechanics
- Two competing addresses (A and B)
- Two escrow accounts (anyone can deposit to either anytime)
- Flipping requires escrow balance >= last_swap * 1.05
- After 3 Solana epochs (~6-9 days) of no flips, anyone can claim
- Perpetual griefing allowed but expensive (pot grows 5% each time)

### Safety Features
- Creator can only close bucket BEFORE first flip (prevents rug pulls)
- Creator must be different from addresses A and B
- Combined fees capped at 20%
- Minimum increase bounded between 1-50%

## The Five Instructions

### 1. create_bucket (Instruction 0)
**Accounts:** [payer, bucket, main_bucket, escrow_a, escrow_b, system_program]

**Data Layout:**
- [0..32] address_a: Pubkey
- [32..64] address_b: Pubkey
- [64..96] creator_address: Pubkey
- [96..98] creator_fee_bps: u16
- [98..100] claimer_fee_bps: u16
- [100..108] initial_last_swap: u64
- [108..110] min_increase_bps: u16
- [110..142] seed: [u8; 32]

**Implementation:** Creates 4 PDAs via System Program CPI, initializes bucket state

### 2. deposit_to_escrow (Instruction 1)
**Accounts:** [depositor, target_escrow, system_program]

**Data Layout:**
- [0..8] amount: u64

**Implementation:** Simple transfer via System Program CPI

### 3. flush_escrow (Instruction 2)
**Accounts:** [bucket, main_bucket, escrow_to_flush]

**Data Layout:** None

**Implementation:**
- Validates threshold: escrow >= last_swap * (10000 + min_increase_bps) / 10000
- Transfers entire escrow balance to main_bucket
- Flips current_target to opposite address
- Updates last_swap and last_flip_epoch

### 4. claim_payout (Instruction 3)
**Accounts:** [bucket, main_bucket, escrow_a, escrow_b, creator, claimer, winner]

**Data Layout:** None

**Implementation:**
- Validates 3 epoch timeout
- Calculates fees using u128 to prevent overflow
- Distributes: creator_cut → creator, claimer_cut → claimer, remainder → winner
- Closes all PDAs by zeroing lamports

### 5. close_bucket (Instruction 4)
**Accounts:** [creator, bucket, main_bucket, escrow_a, escrow_b]

**Data Layout:** None

**Implementation:**
- Validates creator signer
- Validates last_flip_epoch == creation_epoch (no flips)
- Validates escrows are empty
- Returns all lamports to creator

## Key Validation Rules

### create_bucket
- `creator_fee_bps + claimer_fee_bps <= 2000` (max 20%)
- `creator_address != address_a && creator_address != address_b`
- `min_increase_bps >= 100 && min_increase_bps <= 5000` (1-50%)
- `initial_last_swap >= 100_000` (min 0.0001 SOL)

### flush_escrow
- `escrow_balance >= last_swap * (10000 + min_increase_bps) / 10000`

### claim_payout
- `current_epoch >= last_flip_epoch + 3`

### close_bucket
- `signer == creator_address`
- `last_flip_epoch == creation_epoch` (no flips yet)
- `both escrows empty`

## Account Structure

```rust
#[repr(C)]
pub struct Bucket {
    pub address_a: Pubkey,           // 32 bytes
    pub address_b: Pubkey,           // 32 bytes
    pub creator_address: Pubkey,     // 32 bytes
    pub current_target: Pubkey,      // 32 bytes (points to A or B)
    pub last_swap: u64,              // 8 bytes
    pub creation_epoch: u64,         // 8 bytes
    pub last_flip_epoch: u64,        // 8 bytes
    pub creator_fee_bps: u16,        // 2 bytes
    pub claimer_fee_bps: u16,        // 2 bytes
    pub min_increase_bps: u16,       // 2 bytes
    pub bump: u8,                    // 1 byte
}
// Total: 159 bytes
```

## PDA Derivations

All PDAs use `find_program_address` from `pinocchio::pubkey`:

- **Bucket**: `["bucket", creator.key, seed_bytes]`
- **Main bucket**: `["main", bucket.key]`
- **A escrow**: `["escrow_a", bucket.key]`
- **B escrow**: `["escrow_b", bucket.key]`

## Technology Stack

- **Framework**: Pinocchio v0.9.2 (https://github.com/anza-xyz/pinocchio)
- **Platform**: Solana
- **Language**: Rust (edition 2021)

## Pinocchio Implementation Notes

### Key Learnings

1. **Pubkey Type**: `Pubkey` is a type alias for `[u8; 32]`, not a struct
   - Use: `let pubkey: Pubkey = [0u8; 32];`
   - NOT: `Pubkey::from_array([0u8; 32])`

2. **PDA Derivation**: Use free functions from `pinocchio::pubkey`
   - `find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8)`
   - `try_find_program_address(...)` for Option return

3. **Lamports Manipulation**: Use `borrow_mut_lamports_unchecked()`
   - Returns `&mut u64`, not `&mut &mut u64`
   - Example: `*account.borrow_mut_lamports_unchecked() += amount;`

4. **Account Data**: Use raw pointer casting for account data
   - `Bucket::from_account_info(account)` returns `&mut Bucket`
   - Uses unsafe pointer cast internally

5. **CPI**: Use `pinocchio::cpi::{invoke, invoke_signed}`
   - Build `Instruction` struct manually
   - Use `Seed` and `Signer` types for PDAs
   - **Important**: Temporary values in Seed arrays must be stored in variables first

6. **System Program CPI**: No high-level helpers provided
   - Created custom `system_program` module
   - Instruction discriminators: CreateAccount=0, Transfer=2
   - Data layout: [discriminator:u32, params...]

### Common Patterns

**Creating PDA Seeds:**
```rust
let bump_arr = [bump];  // Store in variable to extend lifetime
let seeds = [
    Seed::from(PREFIX),
    Seed::from(pubkey.as_ref()),
    Seed::from(&bump_arr),
];
```

**System Program CPI:**
```rust
system_program::create_account(
    from_account,
    to_account,
    lamports,
    space,
    owner_program_id,
    &seeds,
)?;
```

**Lamports Transfer:**
```rust
unsafe {
    *from.borrow_mut_lamports_unchecked() -= amount;
    *to.borrow_mut_lamports_unchecked() += amount;
}
```

## Custom Error Types

```rust
pub enum HateFunError {
    FeesTooHigh = 0,              // Combined fees > 20%
    CreatorMustBeDifferent = 1,   // Creator == A or B
    InvalidMinimumIncrease = 2,   // Not 1-50%
    InitialSwapTooLow = 3,        // < 0.0001 SOL
    InsufficientEscrowBalance = 4,// Below threshold
    ClaimTooEarly = 5,            // < 3 epochs
    UnauthorizedClose = 6,        // Not creator
    BucketHasFlips = 7,           // Can't close
    EscrowsNotEmpty = 8,          // Can't close
    InvalidEscrow = 9,            // Wrong PDA
    Overflow = 10,                // Math overflow
}
```

## Build & Deployment

### Native Build (Recommended)

**Prerequisites Check:**
```bash
./scripts/setup-native.sh
```

**Build Command:**
```bash
./scripts/build-native.sh
# or
cargo build-sbf --manifest-path=Cargo.toml --sbf-out-dir=dist/program
```

**Output:** `dist/program/hate_fun.so`

**Build Status:** ✅ Compiles cleanly with **zero warnings**

### Cargo Configuration
- Added `test-sbf` feature in Cargo.toml
- Configured allowed cfgs for Pinocchio's `target_os = "solana"`
- Fixed all compiler warnings

## Next Steps

### Phase 4: Testing Infrastructure ✅ COMPLETE
**Status:** Native testing infrastructure ready. Unit tests complete. Integration tests require client SDK.

#### ✅ Completed
- [x] Unit tests for threshold calculations
- [x] Unit tests for fee calculations
- [x] Unit tests for validation bounds checking
- [x] Testing documentation (TESTING.md)
- [x] Native testing infrastructure (NATIVE-TESTING.md)
- [x] Setup script with prerequisites checking
- [x] Build, deploy, and test scripts
- [x] Local validator management scripts
- [x] Removed Docker/Podman dependencies

#### 🔄 Native Testing Workflow Ready
```bash
./scripts/setup-native.sh      # Check prerequisites
./scripts/build-native.sh      # Build program
./scripts/start-validator.sh   # Start local validator
./scripts/deploy-native.sh     # Deploy to local validator
./scripts/test-native.sh       # Run tests
./scripts/stop-validator.sh    # Stop validator
```

#### ⚠️ Prerequisites Needed
- Install `cargo-build-sbf` (part of Agave toolchain)
  ```bash
  sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
  ```
- Ensure `~/.local/share/solana/install/active_release/bin` is in PATH

### Phase 5: Integration Testing (NEXT)
**Requires:** Client SDK to build and send transactions

#### Option 1: TypeScript Client (Recommended)
- [ ] Create TypeScript client using `@solana/web3.js`
- [ ] Implement instruction builders for all 5 instructions
- [ ] Write end-to-end tests for all scenarios
- [ ] Test validation edge cases

#### Option 2: Rust Client
- [ ] Add `solana-sdk` and `solana-client` dependencies
- [ ] Create integration test suite in `tests/`
- [ ] Implement all test scenarios

#### Test Scenarios to Implement
- [ ] Integration test: basic create → deposit → flush → claim flow
- [ ] Test: escalation scenario (multiple flips)
- [ ] Test: close_bucket before first flip
- [ ] Test: close_bucket fails after flip
- [ ] Test: flush fails if below threshold
- [ ] Test: 3 epoch timeout enforcement
- [ ] Test: fee calculations with real transactions
- [ ] Test: edge case - direct deposits to main_bucket

**Testing Approach:** Pinocchio's custom types are incompatible with `solana-program-test`. Integration testing must be done by deploying to local validator/devnet and using client SDK.

See `NATIVE-TESTING.md` for complete native testing guide.

### Phase 6: Deployment Prep
- [ ] Build program for devnet
- [ ] Deploy to devnet
- [ ] Manual end-to-end testing on devnet
- [ ] Create example client code (TypeScript/Rust)
- [ ] Execute all integration test scenarios
- [ ] Security audit (recommended)

### Phase 7: Documentation & Examples
- [x] README with usage instructions
- [x] NATIVE-TESTING.md guide
- [ ] Document all instruction parameters in detail
- [ ] Create example scenarios (Republican vs Democrat, etc.)
- [ ] Add inline code comments
- [ ] Create deployment guide for mainnet
- [ ] Update QUICKSTART.md for native workflow

## Recent Updates (October 2025)

### Build Improvements ✅
- Fixed all compiler warnings (was 4 warnings, now 0)
- Added `test-sbf` feature to Cargo.toml
- Configured allowed cfgs for Pinocchio's `target_os = "solana"`
- `cargo build` now runs completely clean

### Native Testing Infrastructure ✅
**Motivation:** Remove Docker/Podman dependencies for easier local development

**Created Scripts:**
- `scripts/setup-native.sh` - Prerequisites checker with installation instructions
- `scripts/build-native.sh` - Native build using cargo-build-sbf
- `scripts/start-validator.sh` - Start local test validator with health checks
- `scripts/stop-validator.sh` - Stop validator and cleanup
- `scripts/deploy-native.sh` - Deploy to local validator with verification
- `scripts/test-native.sh` - Run unit tests and verify deployment

**Created Documentation:**
- `NATIVE-TESTING.md` - Complete native testing guide
  - Prerequisites and installation
  - Native workflow walkthrough
  - Integration testing options (TypeScript/Rust)
  - Troubleshooting guide
  - Comparison with Docker approach

**Removed:**
- All Docker files (Dockerfile.builder, Dockerfile.validator, docker-compose.yml, .dockerignore)
- Docker scripts (scripts/docker/*)
- Podman scripts (scripts/podman/*)

**Updated Documentation:**
- `README.md` - Prioritizes native testing over Docker
- `.gitignore` - Added validator data directories

### Current Prerequisites Status
- ✅ Cargo installed
- ✅ Solana CLI installed
- ✅ agave-validator installed
- ⚠️ cargo-build-sbf needs installation (part of Agave toolchain)

**To complete setup:**
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
# Then add to PATH: ~/.local/share/solana/install/active_release/bin
```

## Important Context

- Project name "hate.fun" (formerly "Hate Bucket" / "Gate") refers to the psychological mechanism (tribal rivalry)
- This is intended as a fundraising tool that harnesses competitive psychology
- The "perpetual griefing" feature is intentional - forces pot to grow
- Use case example: Political fundraising, sports rivalries, cultural debates

## Questions Already Resolved

- ✅ Who can deposit to escrows? Anyone, to either escrow, anytime
- ✅ Can defender counter-swap? Yes, but must beat last_swap by min_increase%
- ✅ What happens after flush? Entire escrow → main bucket, target flips
- ✅ Minimum increase? 5% default (configurable per bucket, 1-50% allowed)
- ✅ Who triggers final payout? Anyone (incentivized by claimer fee)
- ✅ Can creator rug pull? No, close disabled after first flip
- ✅ Factory or single instance? Factory (anyone can create buckets)
- ✅ Creator participation? Must be distinct from addresses A and B
- ✅ How to use Pinocchio? Low-level library, manual CPI, raw syscalls
- ✅ PDA creation? Via System Program CPI with invoke_signed
- ✅ Lamports manipulation? Use borrow_mut_lamports_unchecked() in unsafe blocks

## Technical Decisions

1. **System Program Helper**: Created custom module rather than using external crate
2. **Seed Lifetime Management**: Store bump arrays in variables before creating Seed types
3. **Fee Calculation**: Use u128 intermediate for fee math to prevent overflow
4. **Account Closing**: Zero lamports in claim/close, runtime handles account cleanup
5. **Instruction Data**: Custom binary format with little-endian encoding
6. **Account Parsing**: Let-else pattern for clean error handling

Read spell.md for complete technical specification details.
