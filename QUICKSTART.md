# hate.fun Quick Start Guide

Get up and running with hate.fun in 5 minutes using Docker.

## Prerequisites

- Docker & Docker Compose installed
- `jq` installed (for JSON parsing)

## Steps

### 1. Start Local Test Validator

```bash
./scripts/docker/start.sh
```

Wait for: `✅ Validator is ready!`

### 2. Build the Program

```bash
docker-compose run builder
```

Output: `dist/program/hate_fun.so` (~20KB)

### 3. Deploy to Validator

```bash
./scripts/docker/deploy.sh
```

Output: Program ID saved to `.program-id`

### 4. Verify Deployment

```bash
# Get program info
PROGRAM_ID=$(cat .program-id)
docker exec hate_fun_validator solana program show $PROGRAM_ID --url http://localhost:8899

# Check epoch
docker exec hate_fun_validator solana epoch-info --url http://localhost:8899
```

## What's Next?

### Run Example Tests

```bash
./scripts/docker/test-basic-flow.sh
```

### View Validator Logs

```bash
docker-compose logs -f validator
```

### Stop Everything

```bash
./scripts/docker/stop.sh
```

## Connection Info

- **RPC Endpoint**: `http://localhost:8899`
- **WebSocket**: `ws://localhost:8900`
- **Network**: Local test validator (not devnet/mainnet)

## Creating Your First Bucket

You'll need a client SDK to interact with the program. Here's the instruction format:

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
```javascript
{
  address_a: Keypair.generate().publicKey,
  address_b: Keypair.generate().publicKey,
  creator_address: wallet.publicKey,
  creator_fee_bps: 500,      // 5%
  claimer_fee_bps: 50,       // 0.5%
  initial_last_swap: 1_000_000_000,  // 1 SOL
  min_increase_bps: 500,     // 5%
  seed: randomBytes(32)
}
```

## Troubleshooting

### Validator won't start

```bash
docker-compose logs validator
# Look for port conflicts or permission issues
```

### Build fails

```bash
# Clean and retry
docker-compose down
docker volume rm hate_fun_cargo-cache
docker-compose run builder
```

### Deployment fails

```bash
# Check validator is running
docker-compose ps

# Check binary exists
ls -lh dist/program/hate_fun.so
```

## Full Documentation

- **[DOCKER.md](DOCKER.md)** - Complete Docker guide
- **[spell.md](spell.md)** - Technical specification
- **[README.md](README.md)** - Project overview

## Need Help?

1. Check [DOCKER.md](DOCKER.md) for detailed troubleshooting
2. Review logs: `docker-compose logs validator`
3. Verify validator health: `docker exec hate_fun_validator solana cluster-version --url http://localhost:8899`
