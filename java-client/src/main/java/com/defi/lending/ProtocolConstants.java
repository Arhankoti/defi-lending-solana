package com.defi.lending;

import java.nio.charset.StandardCharsets;
import org.p2p.solanaj.core.PublicKey;

public final class ProtocolConstants {
    public static final PublicKey SYSTEM_PROGRAM_ID =
        new PublicKey("11111111111111111111111111111111");
    public static final PublicKey TOKEN_PROGRAM_ID =
        new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    public static final PublicKey SYSVAR_RENT_PUBKEY =
        new PublicKey("SysvarRent111111111111111111111111111111111");

    public static final byte[] MARKET_SEED = "market".getBytes(StandardCharsets.UTF_8);
    public static final byte[] LOAN_SEED = "loan".getBytes(StandardCharsets.UTF_8);
    public static final byte[] VAULT_AUTHORITY_SEED = "vault_authority".getBytes(StandardCharsets.UTF_8);
    public static final byte[] COLLATERAL_VAULT_SEED = "collateral_vault".getBytes(StandardCharsets.UTF_8);

    private ProtocolConstants() {
    }
}
