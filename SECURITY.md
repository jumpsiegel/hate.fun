# Security

## Upgrade Authority

### Current Status

**Upgrade Authority:** [To be determined on mainnet deployment]

The hate.fun program is currently upgradeable to allow for bug fixes during the initial launch period.

### Roadmap to Immutability

We are committed to making the program immutable (non-upgradeable) after a thorough testing period:

**Phase 1: Active Development (Devnet)**
- Duration: Until all tests pass and audit complete
- Upgrade Authority: Development team
- Purpose: Bug fixes and improvements

**Phase 2: Mainnet Testing**
- Duration: 30-60 days
- Upgrade Authority: Multi-sig (3-of-5)
- Purpose: Community testing, bug bounty, final security audit

**Phase 3: Immutable**
- Timeline: After successful testing period
- Upgrade Authority: None (set to --final)
- This is **irreversible** - provides maximum security

### Verification

Users can verify the current upgrade authority at any time:

```bash
solana program show <PROGRAM_ID>
```

Look for the "Upgrade Authority" field:
- If it's an address: Program is upgradeable by that address
- If it's "None": Program is immutable (maximum security)

### Bug Bounty

During the testing phase, we welcome security researchers to review the code and report vulnerabilities.

**Scope:** All program instructions and state management
**Rewards:** To be determined based on severity
**Contact:** [Add contact method]

## Security Features

### Bucket-Level Protections

Each bucket has built-in protections that **cannot be bypassed even if the program is upgraded**:

1. **Immutable Parameters** - Once created, bucket parameters are fixed
2. **No Admin Key** - No special addresses can override bucket logic
3. **Creator-Only Close** - Only the creator can close, and only before the first flip
4. **Transparent On-Chain** - All state is verifiable on-chain

### Program Architecture

- **No Global Admin** - The program has no concept of an "owner" or "admin"
- **PDA-Based** - All accounts are PDAs deterministically derived
- **Factory Pattern** - Anyone can create buckets, all are equal
- **Open Source** - All code is available for review

## Verifying the Program

### 1. Check Upgrade Authority

```bash
# View current upgrade authority
solana program show <PROGRAM_ID>
```

### 2. Verify Source Code

```bash
# Clone repository
git clone https://github.com/jumpsiegel/hate.fun
cd hate.fun

# Checkout specific version
git checkout <TAG>

# Build
./scripts/build-native.sh

# Download on-chain program
solana program dump <PROGRAM_ID> onchain.so

# Compare hashes (should match)
sha256sum dist/program/hate_fun.so
sha256sum onchain.so
```

### 3. Review Source Code

The source code is fully open and auditable:
- **Repository:** https://github.com/jumpsiegel/hate.fun
- **License:** [To be added]
- **Test Coverage:** 10/10 tests passing

## Known Limitations

### Before Immutability

While the program is upgradeable:
- The upgrade authority COULD deploy a malicious version
- Users should trust the upgrade authority (individual or multisig)
- Monitor for upgrade announcements
- Verify source code of new versions

### After Immutability

Once set to immutable:
- ✅ No rug pull possible
- ✅ Maximum security
- ❌ Cannot fix bugs
- ❌ Cannot add features

## Responsible Disclosure

If you discover a security vulnerability, please report it responsibly:

1. **DO NOT** disclose publicly
2. **DO NOT** exploit it
3. Contact: [Add secure contact method]
4. Include: Description, steps to reproduce, potential impact

We will:
- Respond within 24 hours
- Provide updates on fix progress
- Credit you (if desired) after fix is deployed
- Consider bug bounty rewards

## Audit Status

- [ ] Code review by community
- [ ] Formal security audit (planned)
- [ ] Bug bounty program (planned)
- [ ] Immutability (after testing period)

## Questions?

For security-related questions:
- GitHub Issues: https://github.com/jumpsiegel/hate.fun/issues
- [Add other contact methods]

---

**Last Updated:** October 2025
**Program Version:** 0.1.0
**Audit Status:** Pending
