//! Instruction: Create a new loan request, locking collateral in the vault.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::LendingError;
use crate::events::LoanRequested;
use crate::math::{collateral_ratio_satisfied, origination_fee_amount};
use crate::state::{Loan, LoanStatus, Market};

/// Accounts required to create a loan request.
#[derive(Accounts)]
#[instruction(loan_seed: u64)]
pub struct CreateLoanRequest<'info> {
    pub market: Account<'info, Market>,
    #[account(
        init,
        payer = borrower,
        space = 8 + Loan::LEN,
        seeds = [
            LOAN_SEED,
            market.key().as_ref(),
            borrower.key().as_ref(),
            &loan_seed.to_le_bytes(),
        ],
        bump
    )]
    pub loan: Account<'info, Loan>,
    /// CHECK: The PDA is verified via seeds and used only as a token authority.
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, loan.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = borrower,
        seeds = [COLLATERAL_VAULT_SEED, loan.key().as_ref()],
        bump,
        token::mint = collateral_mint,
        token::authority = vault_authority
    )]
    pub collateral_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(
        mut,
        constraint = borrower_collateral_ata.owner == borrower.key()
            @ LendingError::InvalidTokenOwner,
        constraint = borrower_collateral_ata.mint == collateral_mint.key()
            @ LendingError::InvalidCollateralMint
    )]
    pub borrower_collateral_ata: Account<'info, TokenAccount>,
    #[account(address = market.collateral_mint @ LendingError::InvalidCollateralMint)]
    pub collateral_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Validates inputs, locks collateral, and initialises the loan account in `Requested` status.
pub fn handle_create_loan_request(
    ctx: Context<CreateLoanRequest>,
    loan_seed: u64,
    principal_amount: u64,
    collateral_amount: u64,
    interest_bps: u16,
    duration_secs: i64,
) -> Result<()> {
    msg!(
        "Creating loan request: principal={}, collateral={}, interest={}bps, duration={}s",
        principal_amount,
        collateral_amount,
        interest_bps,
        duration_secs
    );

    let market = &ctx.accounts.market;
    require!(!market.is_paused, LendingError::MarketPaused);
    require!(principal_amount > 0, LendingError::InvalidAmount);
    require!(collateral_amount > 0, LendingError::InvalidAmount);
    require!(
        (MIN_DURATION_SECS..=MAX_DURATION_SECS).contains(&duration_secs),
        LendingError::InvalidDuration
    );
    require!(
        collateral_ratio_satisfied(
            collateral_amount,
            principal_amount,
            market.min_collateral_ratio_bps,
        )?,
        LendingError::CollateralRatioTooLow
    );

    if market.supply_cap > 0 {
        require!(
            market.total_principal_outstanding + principal_amount <= market.supply_cap,
            LendingError::SupplyCapExceeded
        );
    }
    if market.borrow_cap > 0 {
        require!(
            market.total_collateral_locked + collateral_amount <= market.borrow_cap,
            LendingError::BorrowCapExceeded
        );
    }

    let collateral_cpi = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.borrower_collateral_ata.to_account_info(),
            to: ctx.accounts.collateral_vault.to_account_info(),
            authority: ctx.accounts.borrower.to_account_info(),
        },
    );
    token::transfer(collateral_cpi, collateral_amount)?;

    let now = Clock::get()?.unix_timestamp;
    let loan = &mut ctx.accounts.loan;
    loan.market = market.key();
    loan.borrower = ctx.accounts.borrower.key();
    loan.lender = Pubkey::default();
    loan.principal_amount = principal_amount;
    loan.collateral_amount = collateral_amount;
    loan.interest_bps = interest_bps;
    loan.duration_secs = duration_secs;
    loan.created_at = now;
    loan.funded_at = 0;
    loan.due_at = 0;
    loan.loan_seed = loan_seed;
    loan.status = LoanStatus::Requested;
    loan.vault_authority_bump = ctx.bumps.vault_authority;
    loan.principal_repaid = 0;
    loan.interest_repaid = 0;
    loan.last_interest_update = 0;
    loan.accrued_interest = 0;
    loan.origination_fee_amount = origination_fee_amount(principal_amount, market.origination_fee_bps)?;

    emit!(LoanRequested {
        market: loan.market,
        loan: loan.key(),
        borrower: loan.borrower,
        principal_amount,
        collateral_amount,
        interest_bps,
        duration_secs,
        loan_seed,
    });

    Ok(())
}
