# hate.fun: Competitive Fundraising on Solana

A Solana program that creates competitive bidding wars between two opposing parties, harnessing tribal psychology and game theory to maximize fundraising through escalating commitment.

## 🎯 Shoutout
- I want to give Anatoly a shout out for naming this

## 🎯 Concept

hate.fun (formerly "Hate Bucket" / "Gate") enables anyone to create a fundraising competition between two rival groups. Think:
- **Republicans vs Democrats** - Political fundraising
- **Team A vs Team B** - Sports rivalries
- **Pro-Life vs Pro-Choice** - Social causes
- **Any rivalry you can imagine**

The mechanism is simple but powerful: **whoever raises more money controls where ALL the funds go.**

## 🎮 How It Works

### The Game

1. **Creator sets up a "bucket"** with two competing wallet addresses (A and B)
2. **Anyone can deposit** to either side's escrow at any time
3. **When one side exceeds the threshold** (current high + 5%), they "flip" control
4. **The pot grows** with every flip (minimum 5% increase required)
5. **After 3 epochs** (~6-9 days) with no flips, the winning side claims everything

### Example

```
Initial: A controls → threshold is 1 SOL

B supporters deposit 1.05 SOL → B takes control
A supporters deposit 1.1 SOL → A takes control back
B supporters deposit 1.2 SOL → B takes control
A supporters deposit 1.3 SOL → A takes control back

... pot keeps growing ...

Eventually: No one outbids for 3 epochs → A wins and receives all funds
```

## 🔑 Key Features

### Permissionless Factory
- **Anyone can create buckets** - No gatekeeping
- **Fully immutable** - Parameters locked at creation
- **Independent instances** - Each bucket operates separately

### Economic Design
- **Escalating commitment** - Each flip requires 5%+ more (configurable 1-50%)
- **Winner takes all** - Maximum motivation vs proportional splits
- **Creator fee** - Incentivizes creating viral matchups (up to 20%)
- **Claimer fee** - Anyone can trigger payout (gets small cut)
- **Perpetual griefing** - Allowed but expensive, makes pot bigger

### Safety Features
- **No rug pulls** - Creator cannot close bucket after first flip
- **Creator restrictions** - Must be different from both competing addresses
- **Fee caps** - Combined fees limited to 20%
- **Transparent** - All parameters visible on-chain

## 📦 Architecture

Each bucket contains:
- **Two competing addresses** (A and B) - The rivals
- **Two escrow accounts** - Anyone deposits here
- **Main bucket** - Accumulates funds from flips
- **Current target** - Which address would win
- **Threshold** - Amount needed to flip

All accounts are Program Derived Addresses (PDAs) owned by the program.

## 🔧 Technical Stack

- **Framework**: [Pinocchio](https://github.com/anza-xyz/pinocchio) (v0.9)
- **Platform**: Solana
- **Language**: Rust
- **Size**: ~1,000 LOC

## 📖 Instructions

The program has 5 instructions:

### 1. Create Bucket
Initialize a new competitive bucket with addresses, fees, and parameters.

**Parameters:**
- `address_a`, `address_b` - The two competing addresses
- `creator_address` - Receives creator fee
- `creator_fee_bps` - Creator fee (0-2000 = 0-20%)
- `claimer_fee_bps` - Claimer fee (0-2000 = 0-20%)
- `initial_last_swap` - Starting threshold (min 0.0001 SOL)
- `min_increase_bps` - Minimum increase percentage (100-5000 = 1-50%)

### 2. Deposit to Escrow
Anyone can deposit SOL to either side's escrow.

**Parameters:**
- `amount` - Lamports to deposit

### 3. Flush Escrow
If escrow meets threshold, flip control and transfer funds to main bucket.

**Requirements:**
- Escrow balance ≥ `last_swap × (1 + min_increase%)`

**Effects:**
- Transfers **entire escrow** to main bucket
- Flips `current_target` to opposite address
- Updates `last_swap` and `last_flip_epoch`

### 4. Claim Payout
After 3 epochs of no flips, distribute all funds.

**Requirements:**
- At least 3 epochs since last flip

**Distribution:**
1. Creator receives their fee %
2. Claimer (transaction signer) receives their fee %
3. Winner (current target) receives remainder

### 5. Close Bucket
Creator can close bucket BEFORE first flip if escrows are empty.

**Requirements:**
- Must be creator
- No flips have occurred yet
- Both escrows must be empty

**Effect:**
- Returns all rent + lamports to creator

## 🏗️ Building

```bash
# Build the program
cargo build-sbf --manifest-path=Cargo.toml --sbf-out-dir=dist/program

# Or use the build script
./scripts/build.sh

# Run unit tests
cargo test
```

## 🧪 Testing

**Unit tests:** `cargo test` ✅ (5/5 passing)
**Integration tests:** `cargo test --test integration_client -- --ignored` ✅ (5/5 passing)
**Formal verification:** `./scripts/verify-kani.sh` ✅ (8/8 proofs passing)

All core functionality tested and verified, including:
- Traditional unit and integration tests
- Kani formal verification of arithmetic operations and security properties
- Recent bug fix for close_bucket rent-exempt handling

### Native Testing (Recommended)

Test locally without Docker or Podman:

```bash
# Check prerequisites
./scripts/setup-native.sh

# Build program
./scripts/build-native.sh

# Start local validator
./scripts/start-validator.sh

# Deploy to validator
./scripts/deploy-native.sh

# Run tests
./scripts/test-native.sh

# Stop validator when done
./scripts/stop-validator.sh
```

See [NATIVE-TESTING.md](NATIVE-TESTING.md) for complete native testing guide and [INTEGRATION_TESTS.md](INTEGRATION_TESTS.md) for integration test documentation.

## 🚀 Deployment

### Local Validator

```bash
# Start validator
solana-test-validator

# Deploy program
solana program deploy dist/program/hate_fun.so
```

### Devnet

```bash
# Configure to devnet
solana config set --url devnet

# Deploy program
solana program deploy dist/program/hate_fun.so
```

## 💡 Use Cases

### Political Fundraising
Create bucket with DNC wallet vs RNC wallet. Supporters compete to control where funds go. Both sides are incentivized to keep contributing to flip control.

### Sports Rivalries
Yankees vs Red Sox, Lakers vs Celtics - fans compete with their wallets. Winner's charity/organization receives the pot.

### Social Causes
Pro-life vs pro-choice organizations compete. The debate becomes a fundraising mechanism.

### Community Governance
Two competing proposals, whoever raises more controls implementation budget.

## ⚠️ Important Notes

### This is Experimental Software

- Not audited (audit recommended before mainnet)
- Test thoroughly on devnet first
- Understand the game theory implications
- Consider the psychological/social impact

### The "Hate" Mechanism

The name refers to tribal psychology and competitive dynamics. This tool harnesses rivalry for fundraising but can be used for any competitive scenario, not necessarily adversarial ones.

### Griefing is a Feature

Yes, someone can flip control every 2.9 epochs to prevent payout. But this is intentional - it forces the pot to grow by at least 5% each time, maximizing total funds raised.

## 📚 Documentation

- **[spell.md](spell.md)** - Complete technical specification
- **[KANI.md](KANI.md)** - Formal verification with Kani (complete guide)
- **[NATIVE-TESTING.md](NATIVE-TESTING.md)** - Native testing setup guide
- **[INTEGRATION_TESTS.md](INTEGRATION_TESTS.md)** - Integration test documentation
- **[TESTING.md](TESTING.md)** - General testing guide
- **[QUICKSTART.md](QUICKSTART.md)** - Quick start guide
- **[claude.md](claude.md)** - Development session context

## 🛠️ Development

### Project Structure

```
hate.fun/
├── src/
│   ├── lib.rs                 # Entrypoint
│   ├── state.rs               # Bucket account structure
│   ├── error.rs               # Custom errors
│   ├── system_program.rs      # CPI helpers
│   └── instructions/          # All 5 instructions
├── tests/
│   └── integration_client.rs  # Integration tests (5/5 passing)
├── scripts/
│   ├── build-native.sh        # Build script
│   ├── setup-native.sh        # Prerequisites checker
│   ├── start-validator.sh     # Start local validator
│   ├── stop-validator.sh      # Stop validator
│   ├── deploy-native.sh       # Deploy script
│   └── test-native.sh         # Test runner
└── dist/
    └── program/              # Compiled .so output
```

### Contributing

This is a proof-of-concept. Contributions welcome:
- Additional safety checks
- Client SDK (TypeScript/Rust)
- CLI tool
- Web frontend
- Improved testing infrastructure

## 📄 License

[Add your license here]

## 🙏 Acknowledgments

Built with [Pinocchio](https://github.com/anza-xyz/pinocchio) by Anza.

---

**Disclaimer:** This software is provided as-is. Users are responsible for understanding the implications and risks of using this program. Not financial advice.
