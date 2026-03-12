//! Anchor events emitted by the lending protocol for off-chain indexing.

use anchor_lang::prelude::*;

/// Emitted when a new lending market is created.
#[event]
pub struct MarketInitialized {
    pub market: Pubkey,
    pub admin: Pubkey,
    pub principal_mint: Pubkey,
    pub collateral_mint: Pubkey,
    pub treasury_principal_ata: Pubkey,
}

/// Emitted when a market's configuration is updated by the admin.
#[event]
pub struct MarketUpdated {
    pub market: Pubkey,
    pub protocol_fee_bps: u16,
    pub min_collateral_ratio_bps: u16,
    pub grace_period_secs: i64,
    pub is_paused: bool,
    pub origination_fee_bps: u16,
    pub supply_cap: u64,
    pub borrow_cap: u64,
}

/// Emitted when a borrower creates a new loan request.
#[event]
pub struct LoanRequested {
    pub market: Pubkey,
    pub loan: Pubkey,
    pub borrower: Pubkey,
    pub principal_amount: u64,
    pub collateral_amount: u64,
    pub interest_bps: u16,
    pub duration_secs: i64,
    pub loan_seed: u64,
}

/// Emitted when a borrower cancels their pending loan request.
#[event]
pub struct LoanCancelled {
    pub loan: Pubkey,
    pub borrower: Pubkey,
}

/// Emitted when a lender funds a loan request.
#[event]
pub struct LoanFunded {
    pub loan: Pubkey,
    pub borrower: Pubkey,
    pub lender: Pubkey,
    pub funded_at: i64,
    pub due_at: i64,
}

/// Emitted when a borrower fully repays an active loan.
#[event]
pub struct LoanRepaid {
    pub loan: Pubkey,
    pub borrower: Pubkey,
    pub lender: Pubkey,
    pub principal_amount: u64,
    pub interest_amount: u64,
    pub protocol_fee_amount: u64,
}

/// Emitted when a defaulted loan is liquidated.
#[event]
pub struct LoanLiquidated {
    pub loan: Pubkey,
    pub lender: Pubkey,
    pub liquidator: Pubkey,
}

/// Emitted when a terminal loan account and its vault are closed.
#[event]
pub struct LoanClosed {
    pub loan: Pubkey,
    pub closer: Pubkey,
    pub final_status: u8,
}

/// Emitted when an origination fee is charged during loan funding.
#[event]
pub struct OriginationFeeCharged {
    pub loan: Pubkey,
    pub fee_amount: u64,
}

/// Emitted when a borrower makes a partial repayment on an active loan.
#[event]
pub struct PartialRepayment {
    pub loan: Pubkey,
    pub borrower: Pubkey,
    pub amount: u64,
    pub principal_remaining: u64,
    pub interest_paid: u64,
}
