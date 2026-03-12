//! The [`Market`] account stores global configuration for a single lending market.

use anchor_lang::prelude::*;

/// Top-level market configuration account, one per (admin, principal_mint, collateral_mint) triple.
#[account]
pub struct Market {
    /// The admin authority who can update market parameters.
    pub admin: Pubkey,
    /// SPL mint of the principal (lent) token.
    pub principal_mint: Pubkey,
    /// SPL mint of the collateral token.
    pub collateral_mint: Pubkey,
    /// Associated token account that receives protocol fees (denominated in `principal_mint`).
    pub treasury_principal_ata: Pubkey,
    /// Protocol fee charged on interest, in basis points.
    pub protocol_fee_bps: u16,
    /// Minimum collateral-to-principal ratio required, in basis points (10 000 = 100 %).
    pub min_collateral_ratio_bps: u16,
    /// Extra seconds after `due_at` before a loan can be liquidated.
    pub grace_period_secs: i64,
    /// When `true`, no new loans can be requested or funded.
    pub is_paused: bool,
    /// One-time fee deducted from the principal at funding time, in basis points.
    pub origination_fee_bps: u16,
    /// Maximum total principal outstanding across all active loans (0 = unlimited).
    pub supply_cap: u64,
    /// Maximum total collateral locked across all active loans (0 = unlimited).
    pub borrow_cap: u64,
    /// Running sum of principal currently outstanding across active loans.
    pub total_principal_outstanding: u64,
    /// Running sum of collateral currently locked across active loans.
    pub total_collateral_locked: u64,
}

impl Market {
    /// Account size in bytes (excluding the 8-byte discriminator).
    pub const LEN: usize = 194;
}
