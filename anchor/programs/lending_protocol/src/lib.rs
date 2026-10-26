//! Solana on-chain lending protocol built with Anchor.
//!
//! Borrowers post collateral and request fixed-rate loans; lenders fund them.
//! The protocol charges an origination fee and a share of the interest.

use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;

declare_id!("HUau3frCh5KJ4WbcMvsPwtNnb5a8puYbFEzubpYL4pAZ");

#[program]
pub mod lending_protocol {
    use super::*;

    /// Creates a new lending market with the given configuration.
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        protocol_fee_bps: u16,
        min_collateral_ratio_bps: u16,
        grace_period_secs: i64,
        origination_fee_bps: u16,
        supply_cap: u64,
        borrow_cap: u64,
    ) -> Result<()> {
        instructions::handle_initialize_market(
            ctx,
            protocol_fee_bps,
            min_collateral_ratio_bps,
            grace_period_secs,
            origination_fee_bps,
            supply_cap,
            borrow_cap,
        )
    }

    /// Updates an existing market's configuration (admin only).
    #[allow(clippy::too_many_arguments)]
    pub fn update_market(
        ctx: Context<UpdateMarket>,
        protocol_fee_bps: u16,
        min_collateral_ratio_bps: u16,
        grace_period_secs: i64,
        is_paused: bool,
        origination_fee_bps: u16,
        supply_cap: u64,
        borrow_cap: u64,
    ) -> Result<()> {
        instructions::handle_update_market(
            ctx,
            protocol_fee_bps,
            min_collateral_ratio_bps,
            grace_period_secs,
            is_paused,
            origination_fee_bps,
            supply_cap,
            borrow_cap,
        )
    }

    /// Creates a new loan request, locking the borrower's collateral.
    pub fn create_loan_request(
        ctx: Context<CreateLoanRequest>,
        loan_seed: u64,
        principal_amount: u64,
        collateral_amount: u64,
        interest_bps: u16,
        duration_secs: i64,
    ) -> Result<()> {
        instructions::handle_create_loan_request(
            ctx,
            loan_seed,
            principal_amount,
            collateral_amount,
            interest_bps,
            duration_secs,
        )
    }

    /// Cancels a pending loan request and returns collateral to the borrower.
    pub fn cancel_loan_request(ctx: Context<CancelLoanRequest>) -> Result<()> {
        instructions::handle_cancel_loan_request(ctx)
    }

    /// Funds a pending loan request, transferring principal to the borrower.
    pub fn fund_loan(ctx: Context<FundLoan>) -> Result<()> {
        instructions::handle_fund_loan(ctx)
    }

    /// Fully repays an active loan, returning collateral to the borrower.
    pub fn repay_loan(ctx: Context<RepayLoan>) -> Result<()> {
        instructions::handle_repay_loan(ctx)
    }

    /// Liquidates a defaulted loan, seizing collateral for the lender.
    pub fn liquidate_loan(ctx: Context<LiquidateLoan>) -> Result<()> {
        instructions::handle_liquidate_loan(ctx)
    }

    /// Closes a terminal loan account and its collateral vault.
    pub fn close_loan(ctx: Context<CloseLoan>) -> Result<()> {
        instructions::handle_close_loan(ctx)
    }

    /// Makes a partial repayment on an active loan (interest first, then principal).
    pub fn partial_repay(ctx: Context<PartialRepay>, amount: u64) -> Result<()> {
        instructions::handle_partial_repay(ctx, amount)
    }
}
