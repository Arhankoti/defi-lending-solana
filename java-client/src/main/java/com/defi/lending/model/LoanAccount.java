package com.defi.lending.model;

public record LoanAccount(
    String market,
    String borrower,
    String lender,
    long principalAmount,
    long collateralAmount,
    int interestBps,
    long durationSecs,
    long createdAt,
    long fundedAt,
    long dueAt,
    long loanSeed,
    LoanStatus status,
    int vaultAuthorityBump,
    long principalRepaid,
    long interestRepaid,
    long lastInterestUpdate,
    long accruedInterest,
    long originationFeeAmount
) {
}
