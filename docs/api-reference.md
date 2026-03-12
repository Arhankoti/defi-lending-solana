# API Reference

## Instructions

### initialize_market

Creates a new lending market for a token pair.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| protocol_fee_bps | u16 | Protocol fee on interest (0-10000) |
| min_collateral_ratio_bps | u16 | Minimum collateral ratio (>= 10000) |
| grace_period_secs | i64 | Grace period before liquidation (0-31536000) |
| origination_fee_bps | u16 | One-time funding fee (0-5000) |
| supply_cap | u64 | Max principal outstanding (0 = unlimited) |
| borrow_cap | u64 | Max collateral locked (0 = unlimited) |

**Accounts:**
| Name | Mutable | Signer | Description |
|------|---------|--------|-------------|
| market | Yes | No | PDA to initialize |
| admin | Yes | Yes | Market admin, pays rent |
| principal_mint | No | No | Token mint for loans |
| collateral_mint | No | No | Token mint for collateral |
| treasury_principal_ata | No | No | Protocol fee recipient |
| system_program | No | No | System program |

**Errors:** `DistinctMintsRequired`, `InvalidFeeBps`, `InvalidCollateralRatio`, `InvalidGracePeriod`, `InvalidOriginationFeeBps`

---

### update_market

Updates market parameters. Admin only.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| protocol_fee_bps | u16 | Updated protocol fee |
| min_collateral_ratio_bps | u16 | Updated collateral ratio |
| grace_period_secs | i64 | Updated grace period |
| is_paused | bool | Pause/unpause market |
| origination_fee_bps | u16 | Updated origination fee |
| supply_cap | u64 | Updated supply cap |
| borrow_cap | u64 | Updated borrow cap |

**Accounts:** `market` (mut), `admin` (signer)

**Errors:** `InvalidFeeBps`, `InvalidCollateralRatio`, `InvalidGracePeriod`, `InvalidOriginationFeeBps`, `UnauthorizedAdmin`

---

### create_loan_request

Borrower deposits collateral and creates a loan request.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| loan_seed | u64 | Unique seed per borrower per market |
| principal_amount | u64 | Amount to borrow (> 0) |
| collateral_amount | u64 | Collateral to deposit (> 0) |
| interest_bps | u16 | Annual interest rate in BPS |
| duration_secs | i64 | Loan duration (60-31536000) |

**Accounts:** `market`, `loan` (init), `vault_authority`, `collateral_vault` (init), `borrower` (signer, mut), `borrower_collateral_ata` (mut), `collateral_mint`, `token_program`, `system_program`, `rent`

**Errors:** `MarketPaused`, `InvalidAmount`, `InvalidDuration`, `CollateralRatioTooLow`, `SupplyCapExceeded`, `BorrowCapExceeded`

---

### cancel_loan_request

Cancels an unfunded loan request and returns collateral.

**Accounts:** `market`, `loan` (mut), `collateral_vault` (mut), `vault_authority`, `borrower` (signer), `borrower_collateral_ata` (mut), `token_program`

**Errors:** `InvalidLoanStatus` (must be Requested), `VaultUnderfunded`

---

### fund_loan

Lender funds a requested loan. Origination fee is deducted and sent to treasury.

**Accounts:** `market` (mut), `loan` (mut), `collateral_vault`, `borrower`, `lender` (signer), `lender_principal_ata` (mut), `borrower_principal_ata` (mut), `treasury_principal_ata` (mut), `token_program`

**Errors:** `MarketPaused`, `InvalidLoanStatus`, `SelfLendingNotAllowed`, `VaultUnderfunded`

---

### partial_repay

Borrower makes a partial payment. Interest is paid first, then principal.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| amount | u64 | Amount to repay (> 0, <= outstanding) |

**Accounts:** `market` (mut), `loan` (mut), `borrower` (signer, mut), `lender`, `borrower_principal_ata` (mut), `lender_principal_ata` (mut), `treasury_principal_ata` (mut), `token_program`

**Errors:** `InvalidLoanStatus`, `InvalidPartialRepayAmount`, `RepaymentExceedsOutstanding`

---

### repay_loan

Borrower repays the full remaining balance and reclaims collateral.

**Accounts:** `market` (mut), `loan` (mut), `collateral_vault` (mut), `vault_authority`, `borrower` (signer), `borrower_principal_ata` (mut), `lender`, `lender_principal_ata` (mut), `treasury_principal_ata` (mut), `borrower_collateral_ata` (mut), `token_program`

**Errors:** `InvalidLoanStatus`, `MathOverflow`

---

### liquidate_loan

Liquidates an overdue loan after the grace period. Collateral goes to lender.

**Accounts:** `market` (mut), `loan` (mut), `collateral_vault` (mut), `vault_authority`, `liquidator` (signer), `lender`, `lender_collateral_ata` (mut), `token_program`

**Errors:** `InvalidLoanStatus`, `LoanNotDefaulted`

---

### close_loan

Closes a settled loan account (Cancelled, Repaid, or Liquidated). Reclaims rent.

**Accounts:** `loan` (mut, close), `collateral_vault` (mut), `vault_authority`, `closer` (signer, mut), `token_program`

**Errors:** `LoanNotClosable`, `UnauthorizedCloser`, `VaultNotEmpty`

**Who can close:**
- Cancelled: borrower
- Repaid: borrower or lender
- Liquidated: lender

---

## Events

| Event | Fields | Emitted By |
|-------|--------|------------|
| MarketInitialized | market, admin, mints, treasury | initialize_market |
| MarketUpdated | market, protocol_fee_bps, min_collateral_ratio_bps, grace_period_secs, is_paused, origination_fee_bps, supply_cap, borrow_cap | update_market |
| LoanRequested | market, loan, borrower, amounts, terms | create_loan_request |
| LoanCancelled | loan, borrower | cancel_loan_request |
| LoanFunded | loan, borrower, lender, timestamps | fund_loan |
| OriginationFeeCharged | loan, fee_amount | fund_loan |
| PartialRepayment | loan, borrower, amount, remaining, interest | partial_repay |
| LoanRepaid | loan, borrower, lender, amounts, fee | repay_loan |
| LoanLiquidated | loan, lender, liquidator | liquidate_loan |
| LoanClosed | loan, closer, final_status | close_loan |

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 6000 | InvalidFeeBps | Protocol fee > 100% |
| 6001 | InvalidCollateralRatio | Ratio < 100% |
| 6002 | InvalidGracePeriod | Grace period out of range |
| 6003 | MarketPaused | Market is paused |
| 6004 | InvalidAmount | Zero principal or collateral |
| 6005 | InvalidDuration | Duration out of range |
| 6006 | CollateralRatioTooLow | Below market minimum |
| 6007 | InvalidLoanStatus | Wrong state for operation |
| 6008 | MathOverflow | Arithmetic overflow |
| 6009 | LoanNotDefaulted | Not past grace period |
| 6010 | LoanNotClosable | Loan still active/requested |
| 6011 | UnauthorizedCloser | Wrong closer for status |
| 6012 | UnauthorizedAdmin | Not market admin |
| 6013 | SelfLendingNotAllowed | Borrower == lender |
| 6014 | InvalidTreasury | Treasury token account is invalid |
| 6015 | InvalidTreasuryMint | Treasury mint does not match principal mint |
| 6016 | InvalidTokenOwner | Token account owner mismatch |
| 6017 | InvalidTokenMint | Token account mint is invalid |
| 6018 | InvalidCollateralMint | Collateral mint is invalid |
| 6019 | InvalidPrincipalMint | Principal mint is invalid |
| 6020 | InvalidBorrower | Borrower account is invalid |
| 6021 | InvalidLender | Lender account is invalid |
| 6022 | InvalidVaultAuthority | Vault authority PDA is invalid |
| 6023 | VaultUnderfunded | Collateral vault balance is insufficient |
| 6024 | VaultNotEmpty | Collateral vault must be empty before closing |
| 6025 | DistinctMintsRequired | Same mint for both tokens |
| 6026 | RepaymentExceedsOutstanding | Overpayment |
| 6027 | InvalidPartialRepayAmount | Zero partial repay |
| 6028 | SupplyCapExceeded | Over supply limit |
| 6029 | BorrowCapExceeded | Over borrow limit |
| 6030 | InvalidOriginationFeeBps | Origination fee > 50% |
