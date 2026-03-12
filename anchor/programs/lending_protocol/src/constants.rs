//! Protocol-wide constants used for PDA seeds, fee calculations, and loan duration limits.

/// Denominator for basis-point arithmetic (1 bps = 1/10 000).
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Minimum allowed loan duration in seconds (1 minute).
pub const MIN_DURATION_SECS: i64 = 60;

/// Maximum allowed loan duration in seconds (1 year).
pub const MAX_DURATION_SECS: i64 = 365 * 24 * 60 * 60;

/// PDA seed prefix for the [`Market`] account.
pub const MARKET_SEED: &[u8] = b"market";

/// PDA seed prefix for the [`Loan`] account.
pub const LOAN_SEED: &[u8] = b"loan";

/// PDA seed prefix for the collateral-vault authority PDA.
pub const VAULT_AUTHORITY_SEED: &[u8] = b"vault_authority";

/// PDA seed prefix for the collateral-vault token account.
pub const COLLATERAL_VAULT_SEED: &[u8] = b"collateral_vault";

/// Number of seconds in a 365-day year, used for annualised interest calculations.
pub const SECONDS_PER_YEAR: i64 = 365 * 24 * 60 * 60;
