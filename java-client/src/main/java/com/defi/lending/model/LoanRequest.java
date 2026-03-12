package com.defi.lending.model;

public record LoanRequest(
    long loanSeed,
    long principalAmount,
    long collateralAmount,
    int interestBps,
    long durationSecs
) {
}
