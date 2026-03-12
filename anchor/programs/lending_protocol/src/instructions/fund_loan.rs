//! Instruction: Fund a pending loan request, transferring principal to the borrower.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::LendingError;
use crate::events::{LoanFunded, OriginationFeeCharged};
use crate::state::{Loan, LoanStatus, Market};

/// Accounts required to fund a loan.
#[derive(Accounts)]
pub struct FundLoan<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        has_one = market,
        has_one = borrower @ LendingError::InvalidBorrower
    )]
    pub loan: Account<'info, Loan>,
    #[account(
        seeds = [COLLATERAL_VAULT_SEED, loan.key().as_ref()],
        bump,
        constraint = collateral_vault.mint == market.collateral_mint
            @ LendingError::InvalidCollateralMint
    )]
    pub collateral_vault: Account<'info, TokenAccount>,
    /// CHECK: Address is pinned by loan state.
    #[account(address = loan.borrower @ LendingError::InvalidBorrower)]
    pub borrower: UncheckedAccount<'info>,
    pub lender: Signer<'info>,
    #[account(
        mut,
        constraint = lender_principal_ata.owner == lender.key()
            @ LendingError::InvalidTokenOwner,
        constraint = lender_principal_ata.mint == market.principal_mint
            @ LendingError::InvalidPrincipalMint
    )]
    pub lender_principal_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = borrower_principal_ata.owner == borrower.key()
            @ LendingError::InvalidTokenOwner,
        constraint = borrower_principal_ata.mint == market.principal_mint
            @ LendingError::InvalidPrincipalMint
    )]
    pub borrower_principal_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        address = market.treasury_principal_ata @ LendingError::InvalidTreasury,
        constraint = treasury_principal_ata.mint == market.principal_mint
            @ LendingError::InvalidTreasuryMint
    )]
    pub treasury_principal_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Validates the loan, transfers principal (minus origination fee) to the borrower,
/// sends the fee to the treasury, and activates the loan.
pub fn handle_fund_loan(ctx: Context<FundLoan>) -> Result<()> {
    require!(!ctx.accounts.market.is_paused, LendingError::MarketPaused);
    require!(
        ctx.accounts.loan.status == LoanStatus::Requested,
        LendingError::InvalidLoanStatus
    );
    require!(
        ctx.accounts.lender.key() != ctx.accounts.loan.borrower,
        LendingError::SelfLendingNotAllowed
    );
    require!(
        ctx.accounts.collateral_vault.amount >= ctx.accounts.loan.collateral_amount,
        LendingError::VaultUnderfunded
    );

    let principal_amount = ctx.accounts.loan.principal_amount;
    let collateral_amount = ctx.accounts.loan.collateral_amount;

    // Enforce supply/borrow caps at funding time
    let market = &ctx.accounts.market;
    if market.supply_cap > 0 {
        let new_principal = market.total_principal_outstanding
            .checked_add(principal_amount)
            .ok_or(error!(LendingError::MathOverflow))?;
        require!(
            new_principal <= market.supply_cap,
            LendingError::SupplyCapExceeded
        );
    }
    if market.borrow_cap > 0 {
        let new_collateral = market.total_collateral_locked
            .checked_add(collateral_amount)
            .ok_or(error!(LendingError::MathOverflow))?;
        require!(
            new_collateral <= market.borrow_cap,
            LendingError::BorrowCapExceeded
        );
    }

    let fee = ctx.accounts.loan.origination_fee_amount;
    let borrower_receives = principal_amount
        .checked_sub(fee)
        .ok_or(error!(LendingError::MathOverflow))?;

    msg!(
        "Funding loan: principal={}, origination_fee={}, borrower_receives={}",
        principal_amount,
        fee,
        borrower_receives
    );

    // Transfer principal minus fee to borrower
    let principal_cpi = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.lender_principal_ata.to_account_info(),
            to: ctx.accounts.borrower_principal_ata.to_account_info(),
            authority: ctx.accounts.lender.to_account_info(),
        },
    );
    token::transfer(principal_cpi, borrower_receives)?;

    // Transfer origination fee to treasury
    if fee > 0 {
        let fee_cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.lender_principal_ata.to_account_info(),
                to: ctx.accounts.treasury_principal_ata.to_account_info(),
                authority: ctx.accounts.lender.to_account_info(),
            },
        );
        token::transfer(fee_cpi, fee)?;
    }

    let funded_at = Clock::get()?.unix_timestamp;
    let loan = &mut ctx.accounts.loan;
    loan.lender = ctx.accounts.lender.key();
    loan.funded_at = funded_at;
    loan.due_at = funded_at
        .checked_add(loan.duration_secs)
        .ok_or(error!(LendingError::MathOverflow))?;
    loan.status = LoanStatus::Active;
    loan.last_interest_update = funded_at;

    let loan_key = loan.key();
    let loan_borrower = loan.borrower;
    let loan_lender = loan.lender;
    let loan_due_at = loan.due_at;

    let market = &mut ctx.accounts.market;
    market.total_principal_outstanding = market
        .total_principal_outstanding
        .checked_add(principal_amount)
        .ok_or(error!(LendingError::MathOverflow))?;
    market.total_collateral_locked = market
        .total_collateral_locked
        .checked_add(collateral_amount)
        .ok_or(error!(LendingError::MathOverflow))?;

    if fee > 0 {
        emit!(OriginationFeeCharged {
            loan: loan_key,
            fee_amount: fee,
        });
    }

    emit!(LoanFunded {
        loan: loan_key,
        borrower: loan_borrower,
        lender: loan_lender,
        funded_at,
        due_at: loan_due_at,
    });

    Ok(())
}
