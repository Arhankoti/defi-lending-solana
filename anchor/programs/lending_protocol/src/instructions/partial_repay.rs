//! Instruction: Make a partial repayment on an active loan.

use crate::errors::LendingError;
use crate::events::PartialRepayment;
use crate::math::{calculate_accrued_interest, protocol_fee_amount};
use crate::state::{Loan, LoanStatus, Market};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

/// Accounts required for a partial loan repayment.
#[derive(Accounts)]
pub struct PartialRepay<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        has_one = market,
        has_one = borrower @ LendingError::InvalidBorrower
    )]
    pub loan: Account<'info, Loan>,
    #[account(mut)]
    pub borrower: Signer<'info>,
    /// CHECK: Address is pinned by loan state.
    #[account(address = loan.lender @ LendingError::InvalidLender)]
    pub lender: UncheckedAccount<'info>,
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
    pub token_program: Program<'info, Token>,
}

/// Accrues interest, allocates payment to interest first then principal,
/// and transfers funds to the lender and treasury.
pub fn handle_partial_repay(ctx: Context<PartialRepay>, amount: u64) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    let market = &ctx.accounts.market;

    require!(
        loan.status == LoanStatus::Active,
        LendingError::InvalidLoanStatus
    );
    require!(amount > 0, LendingError::InvalidPartialRepayAmount);

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
    let total_outstanding = principal_owed
        .checked_add(total_interest_owed)
        .ok_or(error!(LendingError::MathOverflow))?;

    require!(
        amount <= total_outstanding,
        LendingError::RepaymentExceedsOutstanding
    );

    // Allocate: interest first, then principal
    let interest_payment = if amount <= total_interest_owed {
        amount
    } else {
        total_interest_owed
    };
    let principal_payment = amount
        .checked_sub(interest_payment)
        .ok_or(error!(LendingError::MathOverflow))?;

    let fee = protocol_fee_amount(interest_payment, market.protocol_fee_bps)?;
    let lender_amount = amount
        .checked_sub(fee)
        .ok_or(error!(LendingError::MathOverflow))?;

    msg!(
        "Partial repay: amount={}, interest_part={}, principal_part={}, fee={}",
        amount,
        interest_payment,
        principal_payment,
        fee
    );

    // Transfer to lender
    let lender_cpi = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.borrower_principal_ata.to_account_info(),
            to: ctx.accounts.lender_principal_ata.to_account_info(),
            authority: ctx.accounts.borrower.to_account_info(),
        },
    );
    token::transfer(lender_cpi, lender_amount)?;

    // Transfer fee to treasury
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

    loan.principal_repaid = loan
        .principal_repaid
        .checked_add(principal_payment)
        .ok_or(error!(LendingError::MathOverflow))?;
    loan.interest_repaid = loan
        .interest_repaid
        .checked_add(interest_payment)
        .ok_or(error!(LendingError::MathOverflow))?;

    if principal_payment > 0 {
        ctx.accounts.market.total_principal_outstanding = ctx
            .accounts
            .market
            .total_principal_outstanding
            .saturating_sub(principal_payment);
    }

    let principal_remaining = loan
        .principal_amount
        .checked_sub(loan.principal_repaid)
        .ok_or(error!(LendingError::MathOverflow))?;

    emit!(PartialRepayment {
        loan: loan.key(),
        borrower: loan.borrower,
        amount,
        principal_remaining,
        interest_paid: interest_payment,
    });

    Ok(())
}
