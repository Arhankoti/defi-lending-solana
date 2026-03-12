package com.defi.lending;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.Arrays;
import java.util.List;
import org.p2p.solanaj.core.PublicKey;

public final class PdaUtil {
    private PdaUtil() {
    }

    public static PublicKey deriveMarketAddress(
        PublicKey programId,
        PublicKey admin,
        PublicKey principalMint,
        PublicKey collateralMint
    ) {
        return PublicKey.findProgramAddress(
            Arrays.asList(
                ProtocolConstants.MARKET_SEED,
                admin.toByteArray(),
                principalMint.toByteArray(),
                collateralMint.toByteArray()
            ),
            programId
        ).getAddress();
    }

    public static LoanAddresses deriveLoanAddresses(
        PublicKey programId,
        PublicKey market,
        PublicKey borrower,
        long loanSeed
    ) {
        byte[] seedBytes = ByteBuffer.allocate(Long.BYTES)
            .order(ByteOrder.LITTLE_ENDIAN)
            .putLong(loanSeed)
            .array();

        PublicKey loan = PublicKey.findProgramAddress(
            List.of(
                ProtocolConstants.LOAN_SEED,
                market.toByteArray(),
                borrower.toByteArray(),
                seedBytes
            ),
            programId
        ).getAddress();

        PublicKey vaultAuthority = PublicKey.findProgramAddress(
            List.of(
                ProtocolConstants.VAULT_AUTHORITY_SEED,
                loan.toByteArray()
            ),
            programId
        ).getAddress();

        PublicKey collateralVault = PublicKey.findProgramAddress(
            List.of(
                ProtocolConstants.COLLATERAL_VAULT_SEED,
                loan.toByteArray()
            ),
            programId
        ).getAddress();

        return new LoanAddresses(loan, vaultAuthority, collateralVault);
    }
}
