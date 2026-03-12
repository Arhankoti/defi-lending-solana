package com.defi.lending;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Arrays;

public final class AnchorDiscriminator {
    private AnchorDiscriminator() {
    }

    public static byte[] forInstruction(String name) {
        return derive("global:" + name);
    }

    public static byte[] forAccount(String name) {
        return derive("account:" + name);
    }

    public static boolean matches(byte[] data, byte[] discriminator) {
        if (data == null || discriminator == null || data.length < discriminator.length) {
            return false;
        }
        return Arrays.equals(Arrays.copyOfRange(data, 0, discriminator.length), discriminator);
    }

    private static byte[] derive(String value) {
        try {
            MessageDigest sha = MessageDigest.getInstance("SHA-256");
            byte[] hash = sha.digest(value.getBytes(StandardCharsets.UTF_8));
            return Arrays.copyOf(hash, 8);
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException("SHA-256 not available", e);
        }
    }
}
