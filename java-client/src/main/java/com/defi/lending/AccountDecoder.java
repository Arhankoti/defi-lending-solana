package com.defi.lending;

import com.defi.lending.model.LoanAccount;
import com.defi.lending.model.LoanStatus;
import com.defi.lending.model.MarketConfig;
import java.util.Arrays;

public final class AccountDecoder {
    private static final byte[] MARKET_DISCRIMINATOR = AnchorDiscriminator.forAccount("Market");
    private static final byte[] LOAN_DISCRIMINATOR = AnchorDiscriminator.forAccount("Loan");

    private AccountDecoder() {
    }

    public static MarketConfig decodeMarket(byte[] data) {
        if (!AnchorDiscriminator.matches(data, MARKET_DISCRIMINATOR)) {
            throw new IllegalArgumentException("Account discriminator does not match Market");
        }

        BorshReader reader = new BorshReader(Arrays.copyOfRange(data, 8, data.length));
        return new MarketConfig(
            reader.readPublicKeyString(),
            reader.readPublicKeyString(),
            reader.readPublicKeyString(),
            reader.readPublicKeyString(),
            reader.readU16(),
            reader.readU16(),
            reader.readI64(),
            reader.readBool(),
            reader.readU16(),
            reader.readU64(),
            reader.readU64(),
            reader.readU64(),
            reader.readU64()
        );
    }

    public static LoanAccount decodeLoan(byte[] data) {
        if (!AnchorDiscriminator.matches(data, LOAN_DISCRIMINATOR)) {
            throw new IllegalArgumentException("Account discriminator does not match Loan");
        }

        BorshReader reader = new BorshReader(Arrays.copyOfRange(data, 8, data.length));
        return new LoanAccount(
            reader.readPublicKeyString(),
            reader.readPublicKeyString(),
            reader.readPublicKeyString(),
            reader.readU64(),
            reader.readU64(),
            reader.readU16(),
            reader.readI64(),
            reader.readI64(),
            reader.readI64(),
            reader.readI64(),
            reader.readU64(),
            LoanStatus.fromOrdinal(reader.readU8()),
            reader.readU8(),
            reader.readU64(),
            reader.readU64(),
            reader.readI64(),
            reader.readU64(),
            reader.readU64()
        );
    }
}
