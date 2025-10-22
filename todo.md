# hate.fun Implementation TODO

## Phase 1: Project Setup ✅ COMPLETE
- [x] Initialize Rust/Cargo project structure
- [x] Add Pinocchio dependencies to Cargo.toml (v0.9)
- [x] Set up build configuration for Solana program (cdylib, release opts)
- [x] Create basic project directory structure (src/, tests/, examples/, scripts/)
- [x] Create build script (scripts/build.sh)

## Phase 2: Core Program Structure ✅ COMPLETE
- [x] Define Bucket account structure with all fields (159 bytes)
- [x] Implement PDA derivation functions (bucket, main_bucket, escrow_a, escrow_b)
- [x] Set up instruction enum with all 5 instructions (discriminators 0-4)
- [x] Create instruction data structures for each instruction
- [x] Create error types (11 custom errors)
- [x] Create system_program helper module for CPI

## Phase 3: Instruction Implementations ✅ COMPLETE
- [x] Implement `create_bucket` instruction
  - [x] Validation logic (fees, addresses, parameters)
  - [x] PDA creation via System Program CPI (bucket, main_bucket, escrow_a, escrow_b)
  - [x] Initialize bucket state with proper epoch tracking
- [x] Implement `deposit_to_escrow` instruction
  - [x] Transfer SOL from signer to escrow PDA via System Program
  - [x] Validate escrow belongs to program
- [x] Implement `flush_escrow` instruction
  - [x] Validate escrow balance meets threshold (last_swap * 1.05)
  - [x] Transfer entire escrow balance → main_bucket
  - [x] Flip current_target to opposite address
  - [x] Update last_swap and last_flip_epoch
- [x] Implement `claim_payout` instruction
  - [x] Validate 3 epoch timeout
  - [x] Calculate fee distributions (u128 math to prevent overflow)
  - [x] Transfer to creator, claimer, winner
  - [x] Close all PDAs by zeroing lamports
- [x] Implement `close_bucket` instruction
  - [x] Validate creator signature, no flips, empty escrows
  - [x] Recover all funds + rent
  - [x] Close all PDAs

**Build Status:** ✅ Compiles successfully with only cfg warnings (expected)

---

## Phase 4: Testing ⚠️ PARTIALLY COMPLETE

**Important Discovery:** Pinocchio uses custom types (`[u8; 32]` for Pubkey, custom `AccountInfo`) that are incompatible with `solana-program-test`. Therefore:
- ✅ Unit tests implemented and passing
- ⚠️ Integration tests require deployment to localnet/devnet

See `TESTING.md` for complete testing guide.

### Unit Tests ✅ COMPLETE
- [x] Test validation logic in create_bucket
  - [x] Fee validation (creator + claimer <= 20%)
  - [x] Creator must differ from A and B
  - [x] Min increase bounds (1-50%)
  - [x] Initial swap minimum (0.0001 SOL)
- [x] Test threshold calculations
- [x] Test fee calculations
- [ ] Test PDA derivation consistency (N/A - relies on solana-program)
- [ ] Test instruction data parsing (N/A - requires integration test)
- [ ] Test error conversions (N/A - requires integration test)

### Integration Tests ⚠️ REQUIRES DEPLOYMENT

**Note:** These tests must be performed manually by deploying to localnet/devnet due to Pinocchio/solana-program-test incompatibility.

#### Basic Flow Tests
- [ ] **Happy path**: create → deposit → flush → claim
  - [ ] Create bucket with standard parameters
  - [ ] Deposit to escrow A (below threshold)
  - [ ] Deposit more to reach threshold
  - [ ] Flush escrow A (should flip to B)
  - [ ] Wait 3 epochs (or configure local validator)
  - [ ] Claim payout and verify distributions

#### Escalation Scenarios
- [ ] **Multiple flips**: A flips to B, B counter-flips to A, repeat
  - [ ] Verify pot grows by min_increase each time
  - [ ] Verify last_swap updates correctly
  - [ ] Verify current_target alternates
- [ ] **Griefing scenario**: Flip every 2.9 epochs to prevent payout
  - [ ] Verify pot continues growing
  - [ ] Verify payout still unavailable

#### Close Bucket Tests
- [ ] **Successful close**: Creator closes before first flip
  - [ ] Create bucket
  - [ ] Close immediately (should succeed)
  - [ ] Verify creator receives all lamports
- [ ] **Failed close after flip**: Cannot close after first flip
  - [ ] Create bucket
  - [ ] Deposit and flush once
  - [ ] Attempt close (should fail with BucketHasFlips)
- [ ] **Failed close with escrows**: Cannot close with non-empty escrows
  - [ ] Create bucket
  - [ ] Deposit to escrow (no flush)
  - [ ] Attempt close (should fail with EscrowsNotEmpty)

#### Flush Validation Tests
- [ ] **Insufficient balance**: Flush fails below threshold
  - [ ] Deposit 99% of required amount
  - [ ] Attempt flush (should fail with InsufficientEscrowBalance)
- [ ] **Exact threshold**: Flush succeeds at exact threshold
  - [ ] Deposit exactly last_swap * 1.05
  - [ ] Flush (should succeed)
- [ ] **Above threshold**: Flush with excess transfers entire balance
  - [ ] Deposit 150% of required amount
  - [ ] Flush and verify entire balance transferred

#### Timeout Tests
- [ ] **Claim too early**: Cannot claim before 3 epochs
  - [ ] Create and flush bucket
  - [ ] Attempt claim after 2 epochs (should fail with ClaimTooEarly)
- [ ] **Claim at exactly 3 epochs**: Should succeed
  - [ ] Wait exactly 3 epochs
  - [ ] Claim (should succeed)

#### Fee Calculation Tests
- [ ] **Standard fees**: 5% creator, 0.5% claimer
  - [ ] Verify exact lamport amounts
  - [ ] Verify winner receives remainder
- [ ] **Maximum fees**: 15% creator, 5% claimer (20% total)
  - [ ] Verify calculations don't overflow
- [ ] **Zero fees**: 0% creator, 0% claimer
  - [ ] Verify winner receives everything

#### Edge Cases
- [ ] **Direct deposits to main_bucket**: Should be included in payout
  - [ ] Send lamports directly to main_bucket PDA
  - [ ] Claim payout
  - [ ] Verify direct deposit included in total
- [ ] **Competing deposits**: Multiple deposits to same escrow
  - [ ] Verify all deposits accumulate
- [ ] **Wrong escrow PDA**: Flush with invalid escrow
  - [ ] Attempt flush with random account (should fail with InvalidEscrow)

### Test Infrastructure
- [ ] Set up test fixtures and helpers
- [ ] Create test accounts and keypairs
- [ ] Mock clock/epoch advancement
- [ ] Helper for creating buckets with default params
- [ ] Helper for calculating expected fees

---

## Phase 5: Deployment Prep
- [ ] Build program for devnet
  - [ ] Run `cargo build-sbf` with devnet config
  - [ ] Verify .so file size and accounts
- [ ] Deploy to devnet
  - [ ] Deploy using Solana CLI
  - [ ] Verify program ID
  - [ ] Test program upgrade
- [ ] End-to-end testing on devnet
  - [ ] Create real bucket on devnet
  - [ ] Execute full lifecycle (deposit → flush → claim)
  - [ ] Monitor transactions and logs
- [ ] Write deployment scripts
  - [ ] Script for deploying to devnet
  - [ ] Script for deploying to mainnet
  - [ ] Script for program upgrades
- [ ] Create example client code
  - [ ] TypeScript/Anchor client SDK
  - [ ] Rust client using solana-sdk
  - [ ] Example: Create bucket
  - [ ] Example: Participate in bucket
  - [ ] Example: Monitor bucket state

---

## Phase 6: Documentation & Examples
- [ ] Write comprehensive README
  - [ ] Project overview and concept
  - [ ] Installation instructions
  - [ ] Quick start guide
  - [ ] API reference
- [ ] Document all instruction parameters
  - [ ] Create bucket parameters
  - [ ] Account ordering for each instruction
  - [ ] Error codes and meanings
- [ ] Create example scenarios
  - [ ] Political fundraising (Republican vs Democrat)
  - [ ] Sports rivalry (Team A vs Team B)
  - [ ] Social causes (Pro vs Con)
- [ ] Add inline code comments
  - [ ] Document safety invariants
  - [ ] Explain complex calculations
  - [ ] Note security considerations
- [ ] Create deployment guide
  - [ ] Devnet deployment walkthrough
  - [ ] Mainnet deployment checklist
  - [ ] Security audit recommendations
  - [ ] Upgrade procedures

---

## Optional Enhancements
- [ ] Add events/logging for indexing
  - [ ] BucketCreated event
  - [ ] EscrowFlushed event
  - [ ] PayoutClaimed event
- [ ] Build simple CLI tool
  - [ ] `gate create` - Create new bucket
  - [ ] `gate deposit` - Deposit to escrow
  - [ ] `gate flush` - Trigger flush
  - [ ] `gate claim` - Claim payout
  - [ ] `gate list` - List all buckets
- [ ] Create web frontend
  - [ ] Bucket visualization dashboard
  - [ ] Real-time flip tracking
  - [ ] Leaderboard of largest pots
  - [ ] Wallet integration
- [ ] Add metadata storage
  - [ ] Bucket names and descriptions
  - [ ] Side labels (e.g., "Democrats", "Republicans")
  - [ ] Optional images/logos
- [ ] Build indexer/API
  - [ ] Index all bucket creations
  - [ ] Track active buckets
  - [ ] Calculate statistics (total raised, flip count, etc.)
  - [ ] REST API for bucket discovery

---

## Implementation Notes

### Completed Work
- **Total Files Created**: 11 Rust source files
- **Total Lines of Code**: ~1,000 LOC
- **Build Time**: <2 seconds
- **Dependencies**: Only Pinocchio v0.9.2

### Key Technical Achievements
1. Successfully implemented System Program CPI without external helper crates
2. Proper PDA seed lifetime management with Pinocchio's Seed type
3. Safe lamports manipulation using unsafe blocks correctly
4. Fee calculation using u128 to prevent overflow
5. Clean error handling with custom error types

### Known Limitations
- No event emission (optional enhancement)
- No metadata storage (optional enhancement)
- Minimal inline documentation (Phase 6)
- No client SDK yet (Phase 5)

### Next Immediate Steps
1. Set up test infrastructure with Solana test framework
2. Implement basic happy path test
3. Add validation tests for all edge cases
4. Run tests and fix any issues discovered
