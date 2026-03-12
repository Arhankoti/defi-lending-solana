package com.defi.lending;

import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;
import java.util.Objects;
import org.p2p.solanaj.core.PublicKey;

public final class BorshWriter {
    private final ByteArrayOutputStream out = new ByteArrayOutputStream();

    public BorshWriter writeU8(int value) {
        out.write(value & 0xFF);
        return this;
    }

    public BorshWriter writeBool(boolean value) {
        return writeU8(value ? 1 : 0);
    }

    public BorshWriter writeU16(int value) {
        out.write(value & 0xFF);
        out.write((value >>> 8) & 0xFF);
        return this;
    }

    public BorshWriter writeU64(long value) {
        for (int i = 0; i < Long.BYTES; i++) {
            out.write((int) ((value >>> (8 * i)) & 0xFF));
        }
        return this;
    }

    public BorshWriter writeI64(long value) {
        return writeU64(value);
    }

    public BorshWriter writeBytes(byte[] bytes) {
        Objects.requireNonNull(bytes, "bytes");
        out.writeBytes(bytes);
        return this;
    }

    public BorshWriter writeFixedBytes(byte[] bytes, int expectedLength) {
        Objects.requireNonNull(bytes, "bytes");
        if (bytes.length != expectedLength) {
            throw new IllegalArgumentException(
                "Expected " + expectedLength + " bytes but received " + bytes.length
            );
        }
        return writeBytes(bytes);
    }

    public BorshWriter writeString(String value) {
        byte[] bytes = value.getBytes(StandardCharsets.UTF_8);
        writeU32(bytes.length);
        return writeBytes(bytes);
    }

    public BorshWriter writePublicKey(PublicKey publicKey) {
        Objects.requireNonNull(publicKey, "publicKey");
        return writeFixedBytes(publicKey.toByteArray(), 32);
    }

    public BorshWriter writeU32(int value) {
        out.write(value & 0xFF);
        out.write((value >>> 8) & 0xFF);
        out.write((value >>> 16) & 0xFF);
        out.write((value >>> 24) & 0xFF);
        return this;
    }

    public byte[] toByteArray() {
        return out.toByteArray();
    }
}
