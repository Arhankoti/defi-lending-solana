//! Instruction: Initialize a new lending market.

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use crate::constants::*;
use crate::errors::LendingError;
use crate::events::MarketInitialized;
use crate::state::Market;

/// Accounts required to initialize a new lending market.
#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Market::LEN,
        seeds = [
            MARKET_SEED,
            admin.key().as_ref(),
            principal_mint.key().as_ref(),
            collateral_mint.key().as_ref(),
        ],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub principal_mint: Account<'info, Mint>,
    pub collateral_mint: Account<'info, Mint>,
    #[account(
        constraint = treasury_principal_ata.mint == principal_mint.key()
            @ LendingError::InvalidTreasuryMint
    )]
    pub treasury_principal_ata: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

/// Creates a new lending market with the given parameters.
pub fn handle_initialize_market(
    ctx: Context<InitializeMarket>,
    protocol_fee_bps: u16,
    min_collateral_ratio_bps: u16,
    grace_period_secs: i64,
    origination_fee_bps: u16,
    supply_cap: u64,
    borrow_cap: u64,
) -> Result<()> {
    msg!(
        "Initializing market: fee={}bps, ratio={}bps, grace={}s, orig_fee={}bps, supply_cap={}, borrow_cap={}",
        protocol_fee_bps,
        min_collateral_ratio_bps,
        grace_period_secs,
        origination_fee_bps,
        supply_cap,
        borrow_cap
    );

    require!(
        ctx.accounts.principal_mint.key() != ctx.accounts.collateral_mint.key(),
        LendingError::DistinctMintsRequired
    );
    require!(
        protocol_fee_bps <= BPS_DENOMINATOR as u16,
        LendingError::InvalidFeeBps
    );
    require!(
        min_collateral_ratio_bps >= BPS_DENOMINATOR as u16,
        LendingError::InvalidCollateralRatio
    );
    require!(
        (0..=MAX_DURATION_SECS).contains(&grace_period_secs),
        LendingError::InvalidGracePeriod
    );
    require!(
        origination_fee_bps <= 5000,
        LendingError::InvalidOriginationFeeBps
    );

    let market = &mut ctx.accounts.market;
    market.admin = ctx.accounts.admin.key();
    market.principal_mint = ctx.accounts.principal_mint.key();
    market.collateral_mint = ctx.accounts.collateral_mint.key();
    market.treasury_principal_ata = ctx.accounts.treasury_principal_ata.key();
    market.protocol_fee_bps = protocol_fee_bps;
    market.min_collateral_ratio_bps = min_collateral_ratio_bps;
    market.grace_period_secs = grace_period_secs;
    market.is_paused = false;
    market.origination_fee_bps = origination_fee_bps;
    market.supply_cap = supply_cap;
    market.borrow_cap = borrow_cap;
    market.total_principal_outstanding = 0;
    market.total_collateral_locked = 0;

    emit!(MarketInitialized {
        market: market.key(),
        admin: market.admin,
        principal_mint: market.principal_mint,
        collateral_mint: market.collateral_mint,
        treasury_principal_ata: market.treasury_principal_ata,
    });

    Ok(())
}
