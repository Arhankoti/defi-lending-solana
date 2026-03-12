//! Pure arithmetic helpers for interest, fee, and collateral-ratio calculations.
//!
//! All functions use `u128` intermediate precision and return checked results.

use anchor_lang::prelude::*;
use crate::constants::{BPS_DENOMINATOR, SECONDS_PER_YEAR};
use crate::errors::LendingError;

/// Returns `true` when `collateral_amount / principal_amount >= min_ratio_bps / 10 000`.
///
/// The comparison is cross-multiplied to avoid division:
/// `collateral * 10 000 >= principal * min_ratio_bps`.
pub fn collateral_ratio_satisfied(
    collateral_amount: u64,
    principal_amount: u64,
    min_ratio_bps: u16,
) -> Result<bool> {
    let lhs = (collateral_amount as u128)
        .checked_mul(BPS_DENOMINATOR as u128)
        .ok_or(error!(LendingError::MathOverflow))?;
    let rhs = (principal_amount as u128)
        .checked_mul(min_ratio_bps as u128)
        .ok_or(error!(LendingError::MathOverflow))?;
    Ok(lhs >= rhs)
}

/// Computes simple (non-compounding) interest over `elapsed_secs`:
///
/// `interest = principal_outstanding * interest_bps * elapsed_secs / (10 000 * SECONDS_PER_YEAR)`
pub fn calculate_accrued_interest(
    principal_outstanding: u64,
    interest_bps: u16,
    elapsed_secs: i64,
) -> Result<u64> {
    if elapsed_secs <= 0 || principal_outstanding == 0 {
        return Ok(0);
    }
    let numerator = (principal_outstanding as u128)
        .checked_mul(interest_bps as u128)
        .ok_or(error!(LendingError::MathOverflow))?
        .checked_mul(elapsed_secs as u128)
        .ok_or(error!(LendingError::MathOverflow))?;
    let denominator = (BPS_DENOMINATOR as u128)
        .checked_mul(SECONDS_PER_YEAR as u128)
        .ok_or(error!(LendingError::MathOverflow))?;
    Ok((numerator / denominator) as u64)
}

/// Computes the one-time origination fee: `principal * fee_bps / 10 000`.
pub fn origination_fee_amount(principal: u64, fee_bps: u16) -> Result<u64> {
    let fee = (principal as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(error!(LendingError::MathOverflow))?
        / (BPS_DENOMINATOR as u128);
    Ok(fee as u64)
}

/// Computes the protocol's share of an interest payment:
/// `interest_amount * protocol_fee_bps / 10 000`.
pub fn protocol_fee_amount(interest_amount: u64, protocol_fee_bps: u16) -> Result<u64> {
    let numerator = (interest_amount as u128)
        .checked_mul(protocol_fee_bps as u128)
        .ok_or(error!(LendingError::MathOverflow))?;
    let fee = numerator
        .checked_div(BPS_DENOMINATOR as u128)
        .ok_or(error!(LendingError::MathOverflow))?;
    u64::try_from(fee).map_err(|_| error!(LendingError::MathOverflow))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collateral_ratio_satisfied_exact() {
        // 150% ratio: collateral=150, principal=100, min_ratio=15000 bps
        assert!(collateral_ratio_satisfied(150, 100, 15_000).unwrap());
    }

    #[test]
    fn test_collateral_ratio_below_minimum() {
        // 100% collateral but 150% required
        assert!(!collateral_ratio_satisfied(100, 100, 15_000).unwrap());
    }

    #[test]
    fn test_collateral_ratio_100_percent() {
        // Exactly 100% = 10_000 bps
        assert!(collateral_ratio_satisfied(100, 100, 10_000).unwrap());
    }

    #[test]
    fn test_accrued_interest_one_year() {
        // 1_000_000 principal, 1000 bps (10%), 1 full year
        let interest = calculate_accrued_interest(1_000_000, 1000, SECONDS_PER_YEAR).unwrap();
        assert_eq!(interest, 100_000); // 10% of 1M
    }

    #[test]
    fn test_accrued_interest_half_year() {
        let interest = calculate_accrued_interest(1_000_000, 1000, SECONDS_PER_YEAR / 2).unwrap();
        assert_eq!(interest, 50_000); // 5% of 1M
    }

    #[test]
    fn test_accrued_interest_zero_elapsed() {
        let interest = calculate_accrued_interest(1_000_000, 1000, 0).unwrap();
        assert_eq!(interest, 0);
    }

    #[test]
    fn test_accrued_interest_negative_elapsed() {
        let interest = calculate_accrued_interest(1_000_000, 1000, -100).unwrap();
        assert_eq!(interest, 0);
    }

    #[test]
    fn test_accrued_interest_zero_principal() {
        let interest = calculate_accrued_interest(0, 1000, SECONDS_PER_YEAR).unwrap();
        assert_eq!(interest, 0);
    }

    #[test]
    fn test_origination_fee() {
        // 1_000_000 principal, 100 bps (1%)
        let fee = origination_fee_amount(1_000_000, 100).unwrap();
        assert_eq!(fee, 10_000);
    }

    #[test]
    fn test_origination_fee_zero_bps() {
        let fee = origination_fee_amount(1_000_000, 0).unwrap();
        assert_eq!(fee, 0);
    }

    #[test]
    fn test_protocol_fee() {
        // 100_000 interest, 500 bps (5%)
        let fee = protocol_fee_amount(100_000, 500).unwrap();
        assert_eq!(fee, 5_000);
    }

    #[test]
    fn test_protocol_fee_zero() {
        let fee = protocol_fee_amount(100_000, 0).unwrap();
        assert_eq!(fee, 0);
    }

    #[test]
    fn test_large_values_no_overflow() {
        // Max u64 principal, reasonable bps, 1 year
        let interest = calculate_accrued_interest(u64::MAX / 10, 10_000, SECONDS_PER_YEAR);
        assert!(interest.is_ok());
    }
}
