package com.defi.lending;

import java.math.BigInteger;
import java.util.Arrays;

public final class Base58Util {
    private static final char[] ALPHABET =
        "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".toCharArray();
    private static final int[] INDEXES = new int[128];

    static {
        Arrays.fill(INDEXES, -1);
        for (int i = 0; i < ALPHABET.length; i++) {
            INDEXES[ALPHABET[i]] = i;
        }
    }

    private Base58Util() {
    }

    public static String encode(byte[] input) {
        if (input.length == 0) {
            return "";
        }

        BigInteger value = new BigInteger(1, input);
        StringBuilder encoded = new StringBuilder();
        BigInteger base = BigInteger.valueOf(58);

        while (value.compareTo(BigInteger.ZERO) > 0) {
            BigInteger[] divRem = value.divideAndRemainder(base);
            encoded.append(ALPHABET[divRem[1].intValue()]);
            value = divRem[0];
        }

        for (byte b : input) {
            if (b == 0) {
                encoded.append(ALPHABET[0]);
            } else {
                break;
            }
        }

        return encoded.reverse().toString();
    }

    public static byte[] decode(String input) {
        if (input == null || input.isEmpty()) {
            return new byte[0];
        }

        BigInteger value = BigInteger.ZERO;
        BigInteger base = BigInteger.valueOf(58);

        for (char c : input.toCharArray()) {
            if (c >= INDEXES.length || INDEXES[c] < 0) {
                throw new IllegalArgumentException("Invalid Base58 character: " + c);
            }
            value = value.multiply(base).add(BigInteger.valueOf(INDEXES[c]));
        }

        byte[] decoded = value.toByteArray();
        if (decoded.length > 0 && decoded[0] == 0) {
            decoded = Arrays.copyOfRange(decoded, 1, decoded.length);
        }

        int leadingZeros = 0;
        while (leadingZeros < input.length() && input.charAt(leadingZeros) == ALPHABET[0]) {
            leadingZeros++;
        }

        byte[] output = new byte[leadingZeros + decoded.length];
        System.arraycopy(decoded, 0, output, leadingZeros, decoded.length);
        return output;
    }
}
