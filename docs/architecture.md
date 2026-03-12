# Architecture

## Overview

The DeFi Lending Protocol is a peer-to-peer collateralized lending system on Solana. It uses Anchor framework for the on-chain program and provides a Java CLI client for interaction.

## On-Chain Components

### Program Structure

```
lending_protocol (Anchor program)
├── lib.rs          → Instruction dispatcher (routes to handlers)
├── constants.rs    → BPS_DENOMINATOR, seeds, durations, SECONDS_PER_YEAR
├── errors.rs       → 31 error codes (LendingError enum)
├── events.rs       → 10 event structs emitted on state changes
├── math.rs         → Pure math functions (interest, fees, ratios)
├── state/
│   ├── market.rs   → Market account (194 bytes)
│   └── loan.rs     → Loan account + LoanStatus enum (232 bytes)
└── instructions/   → 9 instruction handlers (one file each)
```

### Accounts

#### Market (PDA)

Seeds: `["market", admin, principal_mint, collateral_mint]`

The market defines the lending parameters for a specific token pair. One market per (admin, principal_mint, collateral_mint) combination.

| Field | Type | Description |
|-------|------|-------------|
| admin | Pubkey | Market admin who can update parameters |
| principal_mint | Pubkey | Token being lent (e.g., USDC) |
| collateral_mint | Pubkey | Token used as collateral (e.g., SOL) |
| treasury_principal_ata | Pubkey | Protocol fee destination |
| protocol_fee_bps | u16 | Fee on interest (basis points) |
| min_collateral_ratio_bps | u16 | Minimum collateral ratio (e.g., 15000 = 150%) |
| grace_period_secs | i64 | Time after due date before liquidation |
| is_paused | bool | Halts new loans and funding |
| origination_fee_bps | u16 | One-time fee at loan funding |
| supply_cap | u64 | Max total principal outstanding (0 = unlimited) |
| borrow_cap | u64 | Max total collateral locked (0 = unlimited) |
| total_principal_outstanding | u64 | Running counter of active principal |
| total_collateral_locked | u64 | Running counter of locked collateral |

#### Loan (PDA)

Seeds: `["loan", market, borrower, loan_seed_le_bytes]`

Each loan is a separate PDA account with its own collateral vault.

| Field | Type | Description |
|-------|------|-------------|
| market | Pubkey | Associated market |
| borrower | Pubkey | Loan requester |
| lender | Pubkey | Loan funder (set at funding) |
| principal_amount | u64 | Amount borrowed |
| collateral_amount | u64 | Collateral deposited |
| interest_bps | u16 | Annual interest rate in basis points |
| duration_secs | i64 | Loan duration |
| created_at | i64 | Request timestamp |
| funded_at | i64 | Funding timestamp |
| due_at | i64 | Repayment deadline |
| loan_seed | u64 | Unique seed per borrower |
| status | LoanStatus | Requested/Active/Repaid/Cancelled/Liquidated |
| vault_authority_bump | u8 | PDA bump for vault authority |
| principal_repaid | u64 | Cumulative principal paid back |
| interest_repaid | u64 | Cumulative interest paid |
| last_interest_update | i64 | Timestamp for accrual calculation |
| accrued_interest | u64 | Accumulated interest to date |
| origination_fee_amount | u64 | Fee charged at funding |

#### Collateral Vault (PDA Token Account)

Seeds: `["collateral_vault", loan]`

SPL Token account holding the borrower's collateral. Authority is the vault_authority PDA.

#### Vault Authority (PDA)

Seeds: `["vault_authority", loan]`

PDA that signs CPI transfers from the collateral vault.

### Loan Lifecycle

```
         ┌─────────┐
         │Requested │
         └────┬─────┘
        fund  │  cancel
    ┌─────────┼──────────┐
    ▼                     ▼
┌───────┐          ┌──────────┐
│Active │          │Cancelled │
└───┬───┘          └──────────┘
    │ partial_repay (loop)
    │ repay_loan / liquidate
    ├──────────┐
    ▼          ▼
┌──────┐  ┌───────────┐
│Repaid│  │Liquidated │
└──┬───┘  └─────┬─────┘
   │ close      │ close
   ▼            ▼
 [Account Closed]
```

### Interest Model

Simple interest, time-proportional:

```
interest = principal_outstanding * interest_bps * elapsed_secs / (BPS_DENOMINATOR * SECONDS_PER_YEAR)
```

Interest accrues lazily — calculated only when `repay_loan` or `partial_repay` is called. The `last_interest_update` timestamp tracks when interest was last computed.

### Fee Model

1. **Protocol fee** — Percentage of interest paid, sent to treasury on repayment
2. **Origination fee** — Percentage of principal, deducted at funding from lender's transfer

### Risk Controls

- **Minimum collateral ratio** — Enforced at loan creation
- **Supply cap** — Limits total principal outstanding across all loans
- **Borrow cap** — Limits total collateral locked across all loans
- **Grace period** — Buffer time after due date before liquidation is allowed
- **Market pause** — Admin can halt new loans and funding

## Off-Chain Components

### Java Client

The Java client provides:

1. **PDA derivation** — Compute Market and Loan addresses deterministically
2. **Instruction building** — Construct Solana transaction instructions with correct Borsh serialization
3. **Account decoding** — Deserialize on-chain account data into Java records
4. **CLI interface** — Command-line tool for all protocol operations
5. **RPC interaction** — Send transactions via solanaj

### Security Considerations

- All token transfers use CPI through Anchor's type-safe accounts
- Vault authority is a PDA — no private key exists
- Self-lending is explicitly blocked
- All math uses checked arithmetic with overflow protection
- Account constraints validate ownership, mints, and authorities
