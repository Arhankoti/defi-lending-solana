//! The [`Loan`] account tracks a single borrower-lender relationship through its lifecycle.

use anchor_lang::prelude::*;

/// On-chain state for a single loan, from request through repayment or liquidation.
#[account]
pub struct Loan {
    /// The market this loan belongs to.
    pub market: Pubkey,
    /// The wallet that posted collateral and receives principal.
    pub borrower: Pubkey,
    /// The wallet that supplied principal (set on funding).
    pub lender: Pubkey,
    /// Original principal amount in the principal mint's smallest unit.
    pub principal_amount: u64,
    /// Collateral amount locked in the vault.
    pub collateral_amount: u64,
    /// Annual interest rate in basis points.
    pub interest_bps: u16,
    /// Requested loan duration in seconds.
    pub duration_secs: i64,
    /// Unix timestamp when the loan request was created.
    pub created_at: i64,
    /// Unix timestamp when the loan was funded (0 if not yet funded).
    pub funded_at: i64,
    /// Unix timestamp when the loan is due (0 if not yet funded).
    pub due_at: i64,
    /// User-chosen seed to allow multiple loans per borrower-market pair.
    pub loan_seed: u64,
    /// Current lifecycle status of the loan.
    pub status: LoanStatus,
    /// Bump seed for the vault authority PDA.
    pub vault_authority_bump: u8,
    /// Cumulative principal already repaid via partial or full repayments.
    pub principal_repaid: u64,
    /// Cumulative interest already repaid.
    pub interest_repaid: u64,
    /// Unix timestamp of the last interest accrual update.
    pub last_interest_update: i64,
    /// Total interest accrued to date (updated lazily on repay/partial-repay).
    pub accrued_interest: u64,
    /// One-time origination fee computed at loan creation.
    pub origination_fee_amount: u64,
}

impl Loan {
    /// Account size in bytes (excluding the 8-byte discriminator).
    pub const LEN: usize = 232;
}

/// Lifecycle status of a loan.
#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum LoanStatus {
    /// Loan has been requested by the borrower but not yet funded.
    Requested = 0,
    /// Loan has been funded and is actively accruing interest.
    Active = 1,
    /// Loan has been fully repaid by the borrower.
    Repaid = 2,
    /// Loan request was cancelled by the borrower before funding.
    Cancelled = 3,
    /// Loan was liquidated after defaulting past the grace period.
    Liquidated = 4,
}
