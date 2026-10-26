//! Instruction: Cancel a pending loan request and return collateral.

use crate::constants::*;
use crate::errors::LendingError;
use crate::events::LoanCancelled;
use crate::state::{Loan, LoanStatus, Market};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

/// Accounts required to cancel a pending loan request.
#[derive(Accounts)]
pub struct CancelLoanRequest<'info> {
    pub market: Account<'info, Market>,
    #[account(
        mut,
        has_one = market,
        has_one = borrower @ LendingError::InvalidBorrower
    )]
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
    pub borrower: Signer<'info>,
    #[account(
        mut,
        constraint = borrower_collateral_ata.owner == borrower.key()
            @ LendingError::InvalidTokenOwner,
        constraint = borrower_collateral_ata.mint == market.collateral_mint
            @ LendingError::InvalidCollateralMint
    )]
    pub borrower_collateral_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Cancels a `Requested` loan and returns collateral to the borrower.
pub fn handle_cancel_loan_request(ctx: Context<CancelLoanRequest>) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    require!(
        loan.status == LoanStatus::Requested,
        LendingError::InvalidLoanStatus
    );

    let vault_balance = ctx.accounts.collateral_vault.amount;
    require!(
        vault_balance >= loan.collateral_amount,
        LendingError::VaultUnderfunded
    );

    msg!(
        "Cancelling loan request, returning {} collateral",
        vault_balance
    );

    let loan_key = loan.key();
    let signer_seeds: &[&[u8]] = &[
        VAULT_AUTHORITY_SEED,
        loan_key.as_ref(),
        &[loan.vault_authority_bump],
    ];
    let signer = &[signer_seeds];

    let collateral_cpi = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.collateral_vault.to_account_info(),
            to: ctx.accounts.borrower_collateral_ata.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
        signer,
    );
    token::transfer(collateral_cpi, vault_balance)?;

    loan.status = LoanStatus::Cancelled;

    emit!(LoanCancelled {
        loan: loan.key(),
        borrower: loan.borrower,
    });

    Ok(())
}
