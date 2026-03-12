//! Instruction: Close a terminal loan account and its collateral vault.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Token, TokenAccount};
use crate::constants::*;
use crate::errors::LendingError;
use crate::events::LoanClosed;
use crate::state::{Loan, LoanStatus};

/// Accounts required to close a loan account and reclaim rent.
#[derive(Accounts)]
pub struct CloseLoan<'info> {
    #[account(mut, close = closer)]
    pub loan: Account<'info, Loan>,
    #[account(
        mut,
        seeds = [COLLATERAL_VAULT_SEED, loan.key().as_ref()],
        bump,
        constraint = collateral_vault.owner == vault_authority.key()
            @ LendingError::InvalidVaultAuthority
    )]
    pub collateral_vault: Account<'info, TokenAccount>,
    /// CHECK: The PDA is verified via seeds and used only as a token authority.
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, loan.key().as_ref()],
        bump = loan.vault_authority_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub closer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

/// Closes the collateral vault token account and the loan account, returning rent to the closer.
pub fn handle_close_loan(ctx: Context<CloseLoan>) -> Result<()> {
    let loan = &ctx.accounts.loan;

    match loan.status {
        LoanStatus::Cancelled => require!(
            ctx.accounts.closer.key() == loan.borrower,
            LendingError::UnauthorizedCloser
        ),
        LoanStatus::Repaid => require!(
            ctx.accounts.closer.key() == loan.borrower
                || ctx.accounts.closer.key() == loan.lender,
            LendingError::UnauthorizedCloser
        ),
        LoanStatus::Liquidated => require!(
            ctx.accounts.closer.key() == loan.lender,
            LendingError::UnauthorizedCloser
        ),
        _ => return err!(LendingError::LoanNotClosable),
    };

    require!(
        ctx.accounts.collateral_vault.amount == 0,
        LendingError::VaultNotEmpty
    );

    msg!("Closing loan account, status={}", loan.status as u8);

    let loan_key = loan.key();
    let signer_seeds: &[&[u8]] = &[
        VAULT_AUTHORITY_SEED,
        loan_key.as_ref(),
        &[loan.vault_authority_bump],
    ];
    let signer = &[signer_seeds];

    let close_vault_cpi = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.collateral_vault.to_account_info(),
            destination: ctx.accounts.closer.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
        signer,
    );
    token::close_account(close_vault_cpi)?;

    emit!(LoanClosed {
        loan: loan.key(),
        closer: ctx.accounts.closer.key(),
        final_status: loan.status as u8,
    });

    Ok(())
}
