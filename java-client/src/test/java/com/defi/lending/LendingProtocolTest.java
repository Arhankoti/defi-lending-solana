package com.defi.lending;

import static org.junit.jupiter.api.Assertions.*;

import com.defi.lending.model.LoanStatus;
import org.junit.jupiter.api.Test;

public class LendingProtocolTest {
    @Test
    void discriminatorLengthsAreEightBytes() {
        assertEquals(8, AnchorDiscriminator.forInstruction("initialize_market").length);
        assertEquals(8, AnchorDiscriminator.forInstruction("partial_repay").length);
        assertEquals(8, AnchorDiscriminator.forAccount("Loan").length);
    }

    @Test
    void base58RoundTripWorks() {
        byte[] original = new byte[] {0, 1, 2, 3, 4, 5, 6, 7};
        String encoded = Base58Util.encode(original);
        byte[] decoded = Base58Util.decode(encoded);
        assertArrayEquals(original, decoded);
    }

    @Test
    void loanStatusOrdinalMappingIsStable() {
        assertEquals(LoanStatus.REQUESTED, LoanStatus.fromOrdinal(0));
        assertEquals(LoanStatus.ACTIVE, LoanStatus.fromOrdinal(1));
        assertEquals(LoanStatus.REPAID, LoanStatus.fromOrdinal(2));
        assertEquals(LoanStatus.CANCELLED, LoanStatus.fromOrdinal(3));
        assertEquals(LoanStatus.LIQUIDATED, LoanStatus.fromOrdinal(4));
    }
}
