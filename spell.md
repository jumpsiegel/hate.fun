
# Project Description

## Overview

A hate-driven fundraising mechanism on Solana that creates competitive bidding wars between two opposing parties. The system uses escalating commitment and game theory to maximize capital locked in service of tribal competition.

## Concept

The intended use case is to point payout accounts at groups who hate each other (Republicans vs Democrats, pro-life vs pro-choice, sports rivals, etc.). Nothing motivates groups more than hate - this mechanism harnesses that energy for fundraising.

## Architecture

### Factory Pattern

The program is a factory that allows anyone to create individual "hate buckets". Each bucket is:
- Fully permissionless after creation
- Completely immutable (no parameter updates)
- Independent of other buckets

### Bucket Structure

Each bucket contains:
- **Two competing addresses** (A and B) - the rivals who could win the payout
- **Creator address** - receives fee on final payout
- **Main bucket** - accumulates funds from escrow flushes
- **Two escrow accounts** (A_escrow and B_escrow) - anyone can deposit to either
- **Current target** - which address (A or B) would receive payout
- **Last swap amount** - the lamports required to be exceeded for next flip
- **Last flip epoch** - when the target was last changed
- **Fee parameters** - creator and claimer fee percentages
- **Minimum increase** - percentage required to exceed last swap

All escrow accounts and main bucket are PDAs owned by the program.

## Game Mechanics

### Initialization
- Bucket starts pointing at address A
- Initial last_swap value is configurable (e.g., 1 SOL)
- Creator sets all parameters at creation time

### Deposits
- Anyone can deposit to either escrow (A_escrow or B_escrow) at any time
- Deposits to either escrow are allowed regardless of current target
- Direct deposits to main bucket PDA are ignored by logic but included in final payout

### Flushing (Flipping Control)
- If an escrow account balance >= last_swap * (1 + min_increase_bps/10000), it can be flushed
- Example: If last_swap = 1 SOL and min_increase = 5%, need >= 1.05 SOL in escrow
- When flushed:
  1. **Entire escrow balance** transfers to main bucket (not just minimum needed)
  2. Current target flips to the other address
  3. last_swap updates to the amount that was flushed
  4. last_flip_epoch updates to current epoch
  5. Escrow balance becomes 0

### Escalation Example
1. Init: Target → A, last_swap = 1 SOL
2. Someone deposits 1.5 SOL to B_escrow and flushes: Target → B, last_swap = 1.5 SOL
3. A supporter deposits 2 SOL to A_escrow and flushes: Target → A, last_swap = 2 SOL
4. B supporter deposits 3 SOL to B_escrow and flushes: Target → B, last_swap = 3 SOL
5. ... pot grows larger with each flip

### Perpetual Griefing Allowed
- Nothing stops someone from flipping every 2.9 epochs to prevent payout
- But the pot grows by at least min_increase_bps each time
- This is a feature, not a bug - maximizes total funds raised

### Final Payout (After 3 Epochs Idle)
- If no flip occurs for 3 consecutive Solana epochs (~6-9 days), bucket becomes claimable
- Anyone can trigger the payout transaction (incentivized by claimer fee)
- Total payout = main_bucket + A_escrow + B_escrow + any direct deposits
- Distribution:
  1. Creator receives: total * creator_fee_bps / 10000
  2. Transaction sender receives: total * claimer_fee_bps / 10000
  3. Current target (A or B) receives: remainder

## Instructions

### 1. create_bucket

Creates a new hate bucket with immutable parameters.

**Parameters:**
```rust
{
    address_a: Pubkey,          // First competing address
    address_b: Pubkey,          // Second competing address
    creator_address: Pubkey,    // Receives creator fee on final payout
    creator_fee_bps: u16,       // Creator fee in basis points (e.g., 500 = 5%)
    claimer_fee_bps: u16,       // Fee for triggering payout (e.g., 50 = 0.5%)
    initial_last_swap: u64,     // Starting threshold in lamports
    min_increase_bps: u16,      // Minimum increase percentage (e.g., 500 = 5%)
}
```

**Validation:**
- `creator_fee_bps + claimer_fee_bps <= 2000` (max 20% total fees)
- `creator_address != address_a && creator_address != address_b` (creator must be distinct)
- `min_increase_bps >= 100 && min_increase_bps <= 5000` (1% to 50%)
- `initial_last_swap >= 100_000` (minimum 0.0001 SOL)

**Initial State:**
- current_target = address_a
- last_swap = initial_last_swap
- last_flip_epoch = current_epoch
- Creates PDAs: bucket account, main_bucket, a_escrow, b_escrow

### 2. deposit_to_escrow

Anyone can deposit SOL to either escrow account.

**Parameters:**
```rust
{
    bucket: Pubkey,
    target_escrow: Pubkey,  // Either a_escrow or b_escrow PDA
    amount: u64,
}
```

**Effect:**
- Transfers lamports from signer to target_escrow PDA
- No state changes to bucket

### 3. flush_escrow

Attempts to flip control by flushing an escrow that meets the threshold.

**Parameters:**
```rust
{
    bucket: Pubkey,
    escrow_to_flush: Pubkey,  // Either a_escrow or b_escrow
}
```

**Validation:**
- `escrow_balance >= last_swap * (10000 + min_increase_bps) / 10000`

**Effect:**
1. Transfer entire escrow balance to main_bucket
2. Update current_target to opposite address
3. Set last_swap = flushed amount
4. Set last_flip_epoch = current epoch
5. Zero out flushed escrow

### 4. claim_payout

After 3 epochs of inactivity, anyone can trigger final payout.

**Parameters:**
```rust
{
    bucket: Pubkey,
}
```

**Validation:**
- `current_epoch >= last_flip_epoch + 3`

**Effect:**
```rust
total = main_bucket + a_escrow + b_escrow
creator_cut = total * creator_fee_bps / 10000
claimer_cut = total * claimer_fee_bps / 10000
winner_cut = total - creator_cut - claimer_cut

Transfer creator_cut → creator_address
Transfer claimer_cut → tx.signer
Transfer winner_cut → current_target
Close all PDAs
```

### 5. close_bucket

Creator can close bucket before first flip if both escrows are empty.

**Parameters:**
```rust
{
    bucket: Pubkey,
}
```

**Validation:**
- `signer == creator_address`
- `last_flip_epoch == creation_epoch` (no flips have occurred)
- `a_escrow.balance == 0 && b_escrow.balance == 0`

**Effect:**
```rust
total = main_bucket + all_pda_rent_exemptions
Transfer total → creator_address
Close all PDAs (bucket, main_bucket, a_escrow, b_escrow)
```

## Account Structure

### Bucket Account (PDA)
```rust
pub struct Bucket {
    address_a: Pubkey,           // 32 bytes
    address_b: Pubkey,           // 32 bytes
    creator_address: Pubkey,     // 32 bytes
    current_target: Pubkey,      // 32 bytes (points to A or B)
    last_swap: u64,              // 8 bytes
    creation_epoch: u64,         // 8 bytes
    last_flip_epoch: u64,        // 8 bytes
    creator_fee_bps: u16,        // 2 bytes
    claimer_fee_bps: u16,        // 2 bytes
    min_increase_bps: u16,       // 2 bytes
    bump: u8,                    // 1 byte
}
// Total: ~159 bytes + discriminator
```

### PDA Derivations
- Bucket: `["bucket", creator.key, seed_bytes]`
- Main bucket: `["main", bucket.key]`
- A escrow: `["escrow_a", bucket.key]`
- B escrow: `["escrow_b", bucket.key]`

## Economic Design

### Incentive Alignment
- **Tribal hate**: Maximizes participation and capital commitment
- **Sunk cost fallacy**: Each side keeps investing because they've already invested
- **Winner-takes-all**: Maximizes motivation vs proportional splits
- **Creator fee**: Incentivizes creation of viral/interesting matchups
- **Claimer fee**: Ensures timely payout execution
- **Escalating commitment**: Each flip requires more capital

### Attack Resistance
- **Griefing**: Allowed but expensive (must pay min_increase each time)
- **Direct deposits**: Ignored by logic, simply add to final payout
- **Rug pull**: Prevented after first flip (close_bucket disabled)
- **Race conditions**: Atomic transactions prevent double-spends

# Tools

- https://github.com/anza-xyz/pinocchio
