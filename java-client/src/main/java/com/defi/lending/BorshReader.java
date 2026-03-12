package com.defi.lending;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;

public final class BorshReader {
    private final ByteBuffer buffer;

    public BorshReader(byte[] data) {
        this.buffer = ByteBuffer.wrap(data).order(ByteOrder.LITTLE_ENDIAN);
    }

    public int readU8() {
        return Byte.toUnsignedInt(buffer.get());
    }

    public boolean readBool() {
        return readU8() != 0;
    }

    public int readU16() {
        return Short.toUnsignedInt(buffer.getShort());
    }

    public int readU32() {
        return buffer.getInt();
    }

    public long readU64() {
        return buffer.getLong();
    }

    public long readI64() {
        return buffer.getLong();
    }

    public byte[] readBytes(int length) {
        byte[] bytes = new byte[length];
        buffer.get(bytes);
        return bytes;
    }

    public String readString() {
        int length = readU32();
        return new String(readBytes(length), StandardCharsets.UTF_8);
    }

    public String readPublicKeyString() {
        return Base58Util.encode(readBytes(32));
    }

    public byte[] readRemainingBytes() {
        byte[] bytes = new byte[buffer.remaining()];
        buffer.get(bytes);
        return bytes;
    }

    public int position() {
        return buffer.position();
    }

    public void skip(int length) {
        buffer.position(buffer.position() + length);
    }

    public byte[] peekBytes(int length) {
        byte[] bytes = Arrays.copyOfRange(buffer.array(), buffer.position(), buffer.position() + length);
        return bytes;
    }
}
