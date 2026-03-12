# DeFi Lending Protocol — Solana

A collateralized peer-to-peer lending protocol built on Solana using Anchor, with a Java CLI client.

## Architecture

```
┌──────────────┐      ┌─────────────────┐      ┌──────────────┐
│   Borrower   │      │  Solana Program  │      │    Lender    │
│              │─────>│  (Anchor/Rust)   │<─────│              │
└──────────────┘      └────────┬────────┘      └──────────────┘
                               │
                      ┌────────┴────────┐
                      │   Java Client   │
                      │   (CLI / SDK)   │
                      └─────────────────┘
```

**Borrower** deposits collateral, requests a loan. **Lender** funds it. Interest accrues over time. Borrower can partially or fully repay. Overdue loans can be liquidated.

## Features

- **Market management** — Admin-configurable protocol fees, collateral ratios, caps
- **Collateralized loans** — PDA-based collateral vaults with minimum ratio enforcement
- **Time-based interest** — Simple interest accruing proportionally over time (replaces flat BPS)
- **Partial repayments** — Pay down loans incrementally (interest-first allocation)
- **Supply/borrow caps** — Risk management with configurable limits (0 = unlimited)
- **Origination fees** — One-time fee charged at loan funding, sent to treasury
- **Loan lifecycle** — Request, fund, repay, cancel, liquidate, close

## Project Structure

```
.
├── anchor/                # Solana program (Anchor)
│   ├── programs/lending_protocol/src/
│   │   ├── lib.rs              # Thin instruction dispatcher
│   │   ├── constants.rs        # Protocol constants
│   │   ├── errors.rs           # Error codes
│   │   ├── events.rs           # On-chain events
│   │   ├── math.rs             # Interest, fees, collateral ratio math
│   │   ├── state/              # Account structures (Market, Loan)
│   │   └── instructions/       # 9 instruction handlers
│   ├── tests/                  # TypeScript integration tests
│   └── Anchor.toml
├── java-client/            # Java CLI client
│   ├── src/main/java/com/defi/lending/
│   │   ├── LendingProtocolClient.java
│   │   ├── LendingInstructionBuilder.java
│   │   ├── AccountDecoder.java
│   │   ├── Main.java           # CLI entry point
│   │   └── model/              # Data models
│   └── pom.xml
├── docs/                   # Documentation
└── .github/workflows/      # CI/CD
```

## Quick Start

### Prerequisites

- Rust + Cargo
- Solana CLI (v1.18.26+)
- Anchor CLI (v0.30.1)
- Node.js (for integration tests)
- Java 17 + Maven (for Java client)

### Build

```bash
cd anchor
anchor build
```

### Test

```bash
cd anchor

# Rust unit tests
cargo test

# Integration tests (requires local validator)
anchor test

# Java client tests
cd ../java-client
mvn test
```

### Deploy

```bash
# Local
solana-test-validator &
anchor deploy

# Devnet
solana config set --url devnet
anchor deploy --provider.cluster devnet
```

## Instructions

| Instruction | Description |
|-------------|-------------|
| `initialize_market` | Create a new lending market with fees, ratios, caps |
| `update_market` | Update market parameters (admin only) |
| `create_loan_request` | Borrower deposits collateral, requests a loan |
| `cancel_loan_request` | Borrower cancels unfunded request, reclaims collateral |
| `fund_loan` | Lender funds a requested loan (origination fee charged) |
| `partial_repay` | Borrower makes a partial payment (interest first) |
| `repay_loan` | Borrower repays in full, reclaims collateral |
| `liquidate_loan` | Anyone liquidates an overdue loan after grace period |
| `close_loan` | Close a settled loan account to reclaim rent |

## Java CLI Usage

```bash
cd java-client
mvn package -q

# Derive addresses
java -jar target/lending-client.jar derive-market <programId> <admin> <principalMint> <collateralMint>

# Initialize market
java -jar target/lending-client.jar init-market <rpcUrl> <programId> <adminKey> \
  <principalMint> <collateralMint> <treasuryAta> \
  <protocolFeeBps> <minCollateralRatioBps> <gracePeriodSecs> \
  <originationFeeBps> <supplyCap> <borrowCap>

# Create loan
java -jar target/lending-client.jar create-loan <rpcUrl> <programId> <borrowerKey> \
  <market> <borrowerCollateralAta> <collateralMint> \
  <loanSeed> <principalAmount> <collateralAmount> <interestBps> <durationSecs>
```

See `java -jar target/lending-client.jar` for all commands.

## Account Sizes

| Account | Size | Fields |
|---------|------|--------|
| Market | 8 + 194 bytes | 13 fields (admin, mints, fees, caps, counters) |
| Loan | 8 + 232 bytes | 18 fields (parties, amounts, timestamps, repayment tracking) |

## CI/CD

GitHub Actions workflows in `.github/workflows/`:

- **ci.yml** — Runs on push/PR: Rust check + clippy + fmt, Anchor build, integration tests, Java build + test
- **testnet-deploy.yml** — Manual trigger: deploys to devnet or testnet

## Documentation

See [`docs/`](./docs/) for detailed documentation:

- [Architecture](./docs/architecture.md)
- [API Reference](./docs/api-reference.md)
- [Deployment Guide](./docs/deployment.md)

## License

MIT
