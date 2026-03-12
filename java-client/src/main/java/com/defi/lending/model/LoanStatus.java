package com.defi.lending.model;

public enum LoanStatus {
    REQUESTED,
    ACTIVE,
    REPAID,
    CANCELLED,
    LIQUIDATED;

    public static LoanStatus fromOrdinal(int ordinal) {
        LoanStatus[] values = values();
        if (ordinal < 0 || ordinal >= values.length) {
            throw new IllegalArgumentException("Unknown loan status ordinal: " + ordinal);
        }
        return values[ordinal];
    }
}
