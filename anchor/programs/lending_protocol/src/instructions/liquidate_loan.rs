//! Instruction: Liquidate a defaulted loan, seizing collateral for the lender.

use crate::constants::*;
use crate::errors::LendingError;
use crate::events::LoanLiquidated;
use crate::state::{Loan, LoanStatus, Market};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

/// Accounts required to liquidate a defaulted loan.
#[derive(Accounts)]
pub struct LiquidateLoan<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut, has_one = market)]
    pub loan: Account<'info, Loan>,
    #[account(
        mut,
        seeds = [COLLATERAL_VAULT_SEED, loan.key().as_ref()],
        bump,
        constraint = collateral_vault.owner == vault_authority.key()
            @ LendingError::InvalidVaultAuthority,
        constraint = collateral_vault.mint == market.collateral_mint
            @ LendingError::InvalidCollateralMint
    )]
    pub collateral_vault: Account<'info, TokenAccount>,
    /// CHECK: The PDA is verified via seeds and used only as a token authority.
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, loan.key().as_ref()],
        bump = loan.vault_authority_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub liquidator: Signer<'info>,
    /// CHECK: Address is pinned by loan state.
    #[account(address = loan.lender @ LendingError::InvalidLender)]
    pub lender: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = lender_collateral_ata.owner == lender.key()
            @ LendingError::InvalidTokenOwner,
        constraint = lender_collateral_ata.mint == market.collateral_mint
            @ LendingError::InvalidCollateralMint
    )]
    pub lender_collateral_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Verifies the loan is past due + grace period, seizes collateral for the lender,
/// and marks the loan as `Liquidated`.
pub fn handle_liquidate_loan(ctx: Context<LiquidateLoan>) -> Result<()> {
    let market = &ctx.accounts.market;
    let loan = &mut ctx.accounts.loan;

    require!(
        loan.status == LoanStatus::Active,
        LendingError::InvalidLoanStatus
    );

    let now = Clock::get()?.unix_timestamp;
    let liquidation_ts = loan
        .due_at
        .checked_add(market.grace_period_secs)
        .ok_or(error!(LendingError::MathOverflow))?;
    require!(now > liquidation_ts, LendingError::LoanNotDefaulted);

    let vault_balance = ctx.accounts.collateral_vault.amount;
    require!(
        vault_balance >= loan.collateral_amount,
        LendingError::VaultUnderfunded
    );

    msg!(
        "Liquidating loan: collateral={}, past_due_by={}s",
        loan.collateral_amount,
        now.saturating_sub(liquidation_ts)
    );

    let loan_key = loan.key();
    let signer_seeds: &[&[u8]] = &[
        VAULT_AUTHORITY_SEED,
        loan_key.as_ref(),
        &[loan.vault_authority_bump],
    ];
    let signer = &[signer_seeds];

    let seize_collateral_cpi = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.collateral_vault.to_account_info(),
            to: ctx.accounts.lender_collateral_ata.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
        signer,
    );
    token::transfer(seize_collateral_cpi, vault_balance)?;

    loan.status = LoanStatus::Liquidated;

    let remaining_principal = loan
        .principal_amount
        .checked_sub(loan.principal_repaid)
        .ok_or(error!(LendingError::MathOverflow))?;
    ctx.accounts.market.total_principal_outstanding = ctx
        .accounts
        .market
        .total_principal_outstanding
        .saturating_sub(remaining_principal);
    ctx.accounts.market.total_collateral_locked = ctx
        .accounts
        .market
        .total_collateral_locked
        .saturating_sub(loan.collateral_amount);

    emit!(LoanLiquidated {
        loan: loan.key(),
        lender: loan.lender,
        liquidator: ctx.accounts.liquidator.key(),
    });

    Ok(())
}
