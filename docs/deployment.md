# Deployment Guide

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/v1.18.26/install)"

# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor --tag v0.30.1 anchor-cli --locked

# Verify
solana --version    # 1.18.26+
anchor --version    # 0.30.1
```

## Local Development

### 1. Build the program

```bash
cd anchor
anchor build
```

The program binary will be at `target/deploy/lending_protocol.so` and the IDL at `target/idl/lending_protocol.json`.

### 2. Start local validator

```bash
solana-test-validator
```

### 3. Deploy locally

```bash
solana config set --url localhost
anchor deploy
```

### 4. Run tests

```bash
cd anchor

# Rust unit tests (no validator needed)
cargo test

# Integration tests (starts its own validator)
npm install
anchor test
```

## Devnet Deployment

### 1. Configure Solana CLI

```bash
solana config set --url devnet
```

### 2. Create or use a deploy keypair

```bash
# Generate a new keypair (save this securely)
solana-keygen new -o deploy-keypair.json

# Fund it
solana airdrop 5 deploy-keypair.json --url devnet
```

### 3. Build and deploy

```bash
cd anchor
anchor build
anchor deploy --provider.cluster devnet
```

### 4. Note the program ID

After deployment, update `Anchor.toml` and `declare_id!()` in `lib.rs` with the deployed program ID.

## Testnet / Mainnet Deployment

### 1. Update program ID

```bash
# Generate program keypair
solana-keygen new -o target/deploy/lending_protocol-keypair.json

# Get the program ID
solana-keygen pubkey target/deploy/lending_protocol-keypair.json
```

Update in:
- `anchor/Anchor.toml` → `[programs.mainnet]`
- `anchor/programs/lending_protocol/src/lib.rs` → `declare_id!("...")`

### 2. Build with correct ID

```bash
anchor build
```

### 3. Deploy

```bash
# Testnet
anchor deploy --provider.cluster testnet

# Mainnet (use with caution)
anchor deploy --provider.cluster mainnet
```

## CI/CD Deployment

The project includes a GitHub Actions workflow for automated deployment:

1. Set the `DEPLOY_KEYPAIR` secret in your GitHub repository settings (JSON array of the keypair bytes)
2. Go to Actions > "Testnet Deploy" > Run workflow
3. Select cluster (devnet or testnet)

## Post-Deployment: Initialize a Market

Using the Java CLI:

```bash
cd java-client
mvn package -q

java -jar target/lending-client.jar init-market \
  https://api.devnet.solana.com \
  <PROGRAM_ID> \
  <ADMIN_SECRET_KEY_BASE58> \
  <USDC_MINT> \
  <SOL_MINT> \
  <TREASURY_ATA> \
  500   \  # 5% protocol fee
  15000 \  # 150% min collateral ratio
  3600  \  # 1 hour grace period
  100   \  # 1% origination fee
  0     \  # unlimited supply
  0        # unlimited borrow
```

## Upgrading

The program is deployed as upgradeable by default. To upgrade:

```bash
anchor build
anchor upgrade target/deploy/lending_protocol.so --program-id <PROGRAM_ID>
```

To make the program immutable (non-upgradeable):

```bash
solana program set-upgrade-authority <PROGRAM_ID> --final
```

## Monitoring

After deployment, monitor:

1. **Program logs** — `solana logs <PROGRAM_ID>`
2. **Account state** — Use the Java client's decode methods or Anchor's `fetch()`
3. **Events** — Subscribe to program logs and parse Anchor events
