//! Instruction: Update an existing market's configuration.

use crate::constants::*;
use crate::errors::LendingError;
use crate::events::MarketUpdated;
use crate::state::Market;
use anchor_lang::prelude::*;

/// Accounts required to update a market (admin only).
#[derive(Accounts)]
pub struct UpdateMarket<'info> {
    #[account(mut, has_one = admin @ LendingError::UnauthorizedAdmin)]
    pub market: Account<'info, Market>,
    pub admin: Signer<'info>,
}

/// Updates the market's fee, collateral ratio, grace period, pause state, and caps.
#[allow(clippy::too_many_arguments)]
pub fn handle_update_market(
    ctx: Context<UpdateMarket>,
    protocol_fee_bps: u16,
    min_collateral_ratio_bps: u16,
    grace_period_secs: i64,
    is_paused: bool,
    origination_fee_bps: u16,
    supply_cap: u64,
    borrow_cap: u64,
) -> Result<()> {
    msg!(
        "Updating market: fee={}bps, ratio={}bps, paused={}, orig_fee={}bps",
        protocol_fee_bps,
        min_collateral_ratio_bps,
        is_paused,
        origination_fee_bps
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
    market.protocol_fee_bps = protocol_fee_bps;
    market.min_collateral_ratio_bps = min_collateral_ratio_bps;
    market.grace_period_secs = grace_period_secs;
    market.is_paused = is_paused;
    market.origination_fee_bps = origination_fee_bps;
    market.supply_cap = supply_cap;
    market.borrow_cap = borrow_cap;

    emit!(MarketUpdated {
        market: market.key(),
        protocol_fee_bps,
        min_collateral_ratio_bps,
        grace_period_secs,
        is_paused,
        origination_fee_bps,
        supply_cap,
        borrow_cap,
    });

    Ok(())
}
