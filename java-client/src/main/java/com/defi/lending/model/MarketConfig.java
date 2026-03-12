package com.defi.lending.model;

public record MarketConfig(
    String admin,
    String principalMint,
    String collateralMint,
    String treasuryPrincipalAta,
    int protocolFeeBps,
    int minCollateralRatioBps,
    long gracePeriodSecs,
    boolean paused,
    int originationFeeBps,
    long supplyCap,
    long borrowCap,
    long totalPrincipalOutstanding,
    long totalCollateralLocked
) {
}
