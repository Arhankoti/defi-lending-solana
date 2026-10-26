//! Instruction: Fully repay an active loan and release collateral.

use crate::constants::*;
use crate::errors::LendingError;
use crate::events::LoanRepaid;
use crate::math::{calculate_accrued_interest, protocol_fee_amount};
use crate::state::{Loan, LoanStatus, Market};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

/// Accounts required to fully repay an active loan.
#[derive(Accounts)]
pub struct RepayLoan<'info> {
    #[account(mut)]
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
        constraint = borrower_principal_ata.owner == borrower.key()
            @ LendingError::InvalidTokenOwner,
        constraint = borrower_principal_ata.mint == market.principal_mint
            @ LendingError::InvalidPrincipalMint
    )]
    pub borrower_principal_ata: Account<'info, TokenAccount>,
    /// CHECK: Address is pinned by loan state.
    #[account(address = loan.lender @ LendingError::InvalidLender)]
    pub lender: UncheckedAccount<'info>,
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
        address = market.treasury_principal_ata @ LendingError::InvalidTreasury,
        constraint = treasury_principal_ata.mint == market.principal_mint
            @ LendingError::InvalidTreasuryMint
    )]
    pub treasury_principal_ata: Account<'info, TokenAccount>,
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

/// Accrues final interest, transfers repayment to the lender and fee to the treasury,
/// releases collateral, and marks the loan as `Repaid`.
pub fn handle_repay_loan(ctx: Context<RepayLoan>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    let loan = &mut ctx.accounts.loan;

    require!(
        loan.status == LoanStatus::Active,
        LendingError::InvalidLoanStatus
    );

    let now = Clock::get()?.unix_timestamp;
    let elapsed = now
        .checked_sub(loan.last_interest_update)
        .ok_or(error!(LendingError::MathOverflow))?;
    let principal_outstanding = loan
        .principal_amount
        .checked_sub(loan.principal_repaid)
        .ok_or(error!(LendingError::MathOverflow))?;
    let new_interest =
        calculate_accrued_interest(principal_outstanding, loan.interest_bps, elapsed)?;
    loan.accrued_interest = loan
        .accrued_interest
        .checked_add(new_interest)
        .ok_or(error!(LendingError::MathOverflow))?;
    loan.last_interest_update = now;

    let total_interest_owed = loan
        .accrued_interest
        .checked_sub(loan.interest_repaid)
        .ok_or(error!(LendingError::MathOverflow))?;
    let principal_owed = loan
        .principal_amount
        .checked_sub(loan.principal_repaid)
        .ok_or(error!(LendingError::MathOverflow))?;
    let total_owed = principal_owed
        .checked_add(total_interest_owed)
        .ok_or(error!(LendingError::MathOverflow))?;

    let fee = protocol_fee_amount(total_interest_owed, market.protocol_fee_bps)?;
    let lender_payout = total_owed
        .checked_sub(fee)
        .ok_or(error!(LendingError::MathOverflow))?;

    msg!(
        "Repaying loan: principal_owed={}, interest_owed={}, fee={}",
        principal_owed,
        total_interest_owed,
        fee
    );

    let repay_lender_cpi = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.borrower_principal_ata.to_account_info(),
            to: ctx.accounts.lender_principal_ata.to_account_info(),
            authority: ctx.accounts.borrower.to_account_info(),
        },
    );
    token::transfer(repay_lender_cpi, lender_payout)?;

    if fee > 0 {
        let fee_cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.borrower_principal_ata.to_account_info(),
                to: ctx.accounts.treasury_principal_ata.to_account_info(),
                authority: ctx.accounts.borrower.to_account_info(),
            },
        );
        token::transfer(fee_cpi, fee)?;
    }

    let vault_balance = ctx.accounts.collateral_vault.amount;
    require!(
        vault_balance >= loan.collateral_amount,
        LendingError::VaultUnderfunded
    );

    let loan_key = loan.key();
    let signer_seeds: &[&[u8]] = &[
        VAULT_AUTHORITY_SEED,
        loan_key.as_ref(),
        &[loan.vault_authority_bump],
    ];
    let signer = &[signer_seeds];

    let release_collateral_cpi = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.collateral_vault.to_account_info(),
            to: ctx.accounts.borrower_collateral_ata.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
        signer,
    );
    token::transfer(release_collateral_cpi, vault_balance)?;

    loan.status = LoanStatus::Repaid;

    let remaining_principal = loan
        .principal_amount
        .checked_sub(loan.principal_repaid)
        .ok_or(error!(LendingError::MathOverflow))?;
    loan.principal_repaid = loan.principal_amount;
    loan.interest_repaid = loan.accrued_interest;

    market.total_principal_outstanding = market
        .total_principal_outstanding
        .saturating_sub(remaining_principal);
    market.total_collateral_locked = market
        .total_collateral_locked
        .saturating_sub(loan.collateral_amount);

    emit!(LoanRepaid {
        loan: loan.key(),
        borrower: loan.borrower,
        lender: loan.lender,
        principal_amount: loan.principal_amount,
        interest_amount: total_interest_owed,
        protocol_fee_amount: fee,
    });

    Ok(())
}
