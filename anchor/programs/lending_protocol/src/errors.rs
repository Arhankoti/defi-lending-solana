//! Custom error codes for the lending protocol.

use anchor_lang::prelude::*;

/// Errors that can be returned by the lending protocol instructions.
#[error_code]
pub enum LendingError {
    /// The protocol fee basis points exceed the allowed maximum.
    #[msg("The protocol fee basis points are invalid.")]
    InvalidFeeBps,
    /// The minimum collateral ratio is below the 100 % floor.
    #[msg("The minimum collateral ratio basis points are invalid.")]
    InvalidCollateralRatio,
    /// The market grace period is outside the allowed range.
    #[msg("The market grace period is invalid.")]
    InvalidGracePeriod,
    /// The market is currently paused; no new activity is allowed.
    #[msg("The market is paused.")]
    MarketPaused,
    /// A zero or otherwise invalid token amount was supplied.
    #[msg("The provided amount is invalid.")]
    InvalidAmount,
    /// The requested loan duration is outside the allowed range.
    #[msg("The loan duration is invalid.")]
    InvalidDuration,
    /// The posted collateral does not meet the market's minimum ratio.
    #[msg("Collateral ratio is below the market minimum.")]
    CollateralRatioTooLow,
    /// The loan's current status does not permit this operation.
    #[msg("The loan is not in a valid status for this instruction.")]
    InvalidLoanStatus,
    /// An arithmetic overflow or underflow was detected.
    #[msg("Math overflow or underflow occurred.")]
    MathOverflow,
    /// The loan has not yet passed its due date plus grace period.
    #[msg("The loan is not defaulted yet.")]
    LoanNotDefaulted,
    /// The loan is not in a terminal state and cannot be closed.
    #[msg("The loan cannot be closed yet.")]
    LoanNotClosable,
    /// The signer is not authorized to close this loan.
    #[msg("The closer is not authorized to close this loan.")]
    UnauthorizedCloser,
    /// The signer is not the market admin.
    #[msg("The provided admin is not authorized for this market.")]
    UnauthorizedAdmin,
    /// A borrower attempted to fund their own loan request.
    #[msg("Borrower and lender cannot be the same wallet.")]
    SelfLendingNotAllowed,
    /// The treasury token account address does not match the market config.
    #[msg("The treasury token account is invalid.")]
    InvalidTreasury,
    /// The treasury token account's mint does not match the principal mint.
    #[msg("The treasury token account mint does not match the principal mint.")]
    InvalidTreasuryMint,
    /// The token account's owner does not match the expected signer.
    #[msg("The token account owner does not match the expected owner.")]
    InvalidTokenOwner,
    /// The token account's mint is incorrect for this operation.
    #[msg("The token account mint is invalid.")]
    InvalidTokenMint,
    /// The collateral mint does not match the market's configured collateral mint.
    #[msg("The collateral mint is invalid.")]
    InvalidCollateralMint,
    /// The principal mint does not match the market's configured principal mint.
    #[msg("The principal mint is invalid.")]
    InvalidPrincipalMint,
    /// The borrower account does not match the loan's recorded borrower.
    #[msg("The borrower account is invalid.")]
    InvalidBorrower,
    /// The lender account does not match the loan's recorded lender.
    #[msg("The lender account is invalid.")]
    InvalidLender,
    /// The vault authority PDA does not match the expected derivation.
    #[msg("The vault authority PDA is invalid.")]
    InvalidVaultAuthority,
    /// The collateral vault does not hold enough tokens.
    #[msg("The collateral vault balance is insufficient.")]
    VaultUnderfunded,
    /// The collateral vault must be drained before the loan account can be closed.
    #[msg("The collateral vault must be empty before closing.")]
    VaultNotEmpty,
    /// Principal and collateral mints must be different SPL tokens.
    #[msg("Principal mint and collateral mint must be different assets.")]
    DistinctMintsRequired,
    /// The partial-repay amount exceeds the remaining balance.
    #[msg("Repayment exceeds outstanding balance")]
    RepaymentExceedsOutstanding,
    /// Partial repay was called with a zero amount.
    #[msg("Partial repay amount must be greater than zero")]
    InvalidPartialRepayAmount,
    /// Funding or requesting a loan would exceed the market's supply cap.
    #[msg("Supply cap exceeded")]
    SupplyCapExceeded,
    /// Funding or requesting a loan would exceed the market's borrow cap.
    #[msg("Borrow cap exceeded")]
    BorrowCapExceeded,
    /// The origination fee basis points exceed the allowed maximum.
    #[msg("Invalid origination fee BPS")]
    InvalidOriginationFeeBps,
}
