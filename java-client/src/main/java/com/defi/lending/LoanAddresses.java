package com.defi.lending;

import org.p2p.solanaj.core.PublicKey;

public record LoanAddresses(
    PublicKey loan,
    PublicKey vaultAuthority,
    PublicKey collateralVault
) {
}
