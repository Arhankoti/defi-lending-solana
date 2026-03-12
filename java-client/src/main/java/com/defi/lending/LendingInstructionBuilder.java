package com.defi.lending;

import com.defi.lending.model.LoanRequest;
import java.util.ArrayList;
import java.util.List;
import org.p2p.solanaj.core.AccountMeta;
import org.p2p.solanaj.core.PublicKey;
import org.p2p.solanaj.core.TransactionInstruction;

public final class LendingInstructionBuilder {
    private final PublicKey programId;

    public LendingInstructionBuilder(PublicKey programId) {
        this.programId = programId;
    }

    public TransactionInstruction buildInitializeMarket(
        PublicKey market,
        PublicKey admin,
        PublicKey principalMint,
        PublicKey collateralMint,
        PublicKey treasuryPrincipalAta,
        int protocolFeeBps,
        int minCollateralRatioBps,
        long gracePeriodSecs,
        int originationFeeBps,
        long supplyCap,
        long borrowCap
    ) {
        byte[] data = new BorshWriter()
            .writeBytes(AnchorDiscriminator.forInstruction("initialize_market"))
            .writeU16(protocolFeeBps)
            .writeU16(minCollateralRatioBps)
            .writeI64(gracePeriodSecs)
            .writeU16(originationFeeBps)
            .writeU64(supplyCap)
            .writeU64(borrowCap)
            .toByteArray();

        List<AccountMeta> metas = new ArrayList<>();
        metas.add(new AccountMeta(market, false, true));
        metas.add(new AccountMeta(admin, true, true));
        metas.add(new AccountMeta(principalMint, false, false));
        metas.add(new AccountMeta(collateralMint, false, false));
        metas.add(new AccountMeta(treasuryPrincipalAta, false, false));
        metas.add(new AccountMeta(ProtocolConstants.SYSTEM_PROGRAM_ID, false, false));

        return new TransactionInstruction(programId, metas, data);
    }

    public TransactionInstruction buildUpdateMarket(
        PublicKey market,
        PublicKey admin,
        int protocolFeeBps,
        int minCollateralRatioBps,
        long gracePeriodSecs,
        boolean paused,
        int originationFeeBps,
        long supplyCap,
        long borrowCap
    ) {
        byte[] data = new BorshWriter()
            .writeBytes(AnchorDiscriminator.forInstruction("update_market"))
            .writeU16(protocolFeeBps)
            .writeU16(minCollateralRatioBps)
            .writeI64(gracePeriodSecs)
            .writeBool(paused)
            .writeU16(originationFeeBps)
            .writeU64(supplyCap)
            .writeU64(borrowCap)
            .toByteArray();

        List<AccountMeta> metas = new ArrayList<>();
        metas.add(new AccountMeta(market, false, true));
        metas.add(new AccountMeta(admin, true, false));

        return new TransactionInstruction(programId, metas, data);
    }

    public TransactionInstruction buildCreateLoanRequest(
        PublicKey market,
        PublicKey borrower,
        PublicKey borrowerCollateralAta,
        PublicKey collateralMint,
        LoanAddresses addresses,
        LoanRequest request
    ) {
        byte[] data = new BorshWriter()
            .writeBytes(AnchorDiscriminator.forInstruction("create_loan_request"))
            .writeU64(request.loanSeed())
            .writeU64(request.principalAmount())
            .writeU64(request.collateralAmount())
            .writeU16(request.interestBps())
            .writeI64(request.durationSecs())
            .toByteArray();

        List<AccountMeta> metas = new ArrayList<>();
        metas.add(new AccountMeta(market, false, false));
        metas.add(new AccountMeta(addresses.loan(), false, true));
        metas.add(new AccountMeta(addresses.vaultAuthority(), false, false));
        metas.add(new AccountMeta(addresses.collateralVault(), false, true));
        metas.add(new AccountMeta(borrower, true, true));
        metas.add(new AccountMeta(borrowerCollateralAta, false, true));
        metas.add(new AccountMeta(collateralMint, false, false));
        metas.add(new AccountMeta(ProtocolConstants.TOKEN_PROGRAM_ID, false, false));
        metas.add(new AccountMeta(ProtocolConstants.SYSTEM_PROGRAM_ID, false, false));
        metas.add(new AccountMeta(ProtocolConstants.SYSVAR_RENT_PUBKEY, false, false));

        return new TransactionInstruction(programId, metas, data);
    }

    public TransactionInstruction buildCancelLoanRequest(
        PublicKey market,
        PublicKey borrower,
        PublicKey borrowerCollateralAta,
        LoanAddresses addresses
    ) {
        byte[] data = new BorshWriter()
            .writeBytes(AnchorDiscriminator.forInstruction("cancel_loan_request"))
            .toByteArray();

        List<AccountMeta> metas = new ArrayList<>();
        metas.add(new AccountMeta(market, false, false));
        metas.add(new AccountMeta(addresses.loan(), false, true));
        metas.add(new AccountMeta(addresses.collateralVault(), false, true));
        metas.add(new AccountMeta(addresses.vaultAuthority(), false, false));
        metas.add(new AccountMeta(borrower, true, false));
        metas.add(new AccountMeta(borrowerCollateralAta, false, true));
        metas.add(new AccountMeta(ProtocolConstants.TOKEN_PROGRAM_ID, false, false));

        return new TransactionInstruction(programId, metas, data);
    }

    public TransactionInstruction buildFundLoan(
        PublicKey market,
        PublicKey borrower,
        PublicKey lender,
        PublicKey lenderPrincipalAta,
        PublicKey borrowerPrincipalAta,
        PublicKey treasuryPrincipalAta,
        LoanAddresses addresses
    ) {
        byte[] data = new BorshWriter()
            .writeBytes(AnchorDiscriminator.forInstruction("fund_loan"))
            .toByteArray();

        List<AccountMeta> metas = new ArrayList<>();
        metas.add(new AccountMeta(market, false, true));
        metas.add(new AccountMeta(addresses.loan(), false, true));
        metas.add(new AccountMeta(addresses.collateralVault(), false, false));
        metas.add(new AccountMeta(borrower, false, false));
        metas.add(new AccountMeta(lender, true, false));
        metas.add(new AccountMeta(lenderPrincipalAta, false, true));
        metas.add(new AccountMeta(borrowerPrincipalAta, false, true));
        metas.add(new AccountMeta(treasuryPrincipalAta, false, true));
        metas.add(new AccountMeta(ProtocolConstants.TOKEN_PROGRAM_ID, false, false));

        return new TransactionInstruction(programId, metas, data);
    }

    public TransactionInstruction buildRepayLoan(
        PublicKey market,
        PublicKey borrower,
        PublicKey borrowerPrincipalAta,
        PublicKey lender,
        PublicKey lenderPrincipalAta,
        PublicKey treasuryPrincipalAta,
        PublicKey borrowerCollateralAta,
        LoanAddresses addresses
    ) {
        byte[] data = new BorshWriter()
            .writeBytes(AnchorDiscriminator.forInstruction("repay_loan"))
            .toByteArray();

        List<AccountMeta> metas = new ArrayList<>();
        metas.add(new AccountMeta(market, false, false));
        metas.add(new AccountMeta(addresses.loan(), false, true));
        metas.add(new AccountMeta(addresses.collateralVault(), false, true));
        metas.add(new AccountMeta(addresses.vaultAuthority(), false, false));
        metas.add(new AccountMeta(borrower, true, false));
        metas.add(new AccountMeta(borrowerPrincipalAta, false, true));
        metas.add(new AccountMeta(lender, false, false));
        metas.add(new AccountMeta(lenderPrincipalAta, false, true));
        metas.add(new AccountMeta(treasuryPrincipalAta, false, true));
        metas.add(new AccountMeta(borrowerCollateralAta, false, true));
        metas.add(new AccountMeta(ProtocolConstants.TOKEN_PROGRAM_ID, false, false));

        return new TransactionInstruction(programId, metas, data);
    }

    public TransactionInstruction buildLiquidateLoan(
        PublicKey market,
        PublicKey liquidator,
        PublicKey lender,
        PublicKey lenderCollateralAta,
        LoanAddresses addresses
    ) {
        byte[] data = new BorshWriter()
            .writeBytes(AnchorDiscriminator.forInstruction("liquidate_loan"))
            .toByteArray();

        List<AccountMeta> metas = new ArrayList<>();
        metas.add(new AccountMeta(market, false, false));
        metas.add(new AccountMeta(addresses.loan(), false, true));
        metas.add(new AccountMeta(addresses.collateralVault(), false, true));
        metas.add(new AccountMeta(addresses.vaultAuthority(), false, false));
        metas.add(new AccountMeta(liquidator, true, false));
        metas.add(new AccountMeta(lender, false, false));
        metas.add(new AccountMeta(lenderCollateralAta, false, true));
        metas.add(new AccountMeta(ProtocolConstants.TOKEN_PROGRAM_ID, false, false));

        return new TransactionInstruction(programId, metas, data);
    }

    public TransactionInstruction buildPartialRepay(
        PublicKey market,
        PublicKey borrower,
        PublicKey lender,
        PublicKey borrowerPrincipalAta,
        PublicKey lenderPrincipalAta,
        PublicKey treasuryPrincipalAta,
        LoanAddresses addresses,
        long amount
    ) {
        byte[] data = new BorshWriter()
            .writeBytes(AnchorDiscriminator.forInstruction("partial_repay"))
            .writeU64(amount)
            .toByteArray();

        List<AccountMeta> metas = new ArrayList<>();
        metas.add(new AccountMeta(market, false, false));
        metas.add(new AccountMeta(addresses.loan(), false, true));
        metas.add(new AccountMeta(borrower, true, true));
        metas.add(new AccountMeta(lender, false, false));
        metas.add(new AccountMeta(borrowerPrincipalAta, false, true));
        metas.add(new AccountMeta(lenderPrincipalAta, false, true));
        metas.add(new AccountMeta(treasuryPrincipalAta, false, true));
        metas.add(new AccountMeta(ProtocolConstants.TOKEN_PROGRAM_ID, false, false));

        return new TransactionInstruction(programId, metas, data);
    }

    public TransactionInstruction buildCloseLoan(
        PublicKey closer,
        LoanAddresses addresses
    ) {
        byte[] data = new BorshWriter()
            .writeBytes(AnchorDiscriminator.forInstruction("close_loan"))
            .toByteArray();

        List<AccountMeta> metas = new ArrayList<>();
        metas.add(new AccountMeta(addresses.loan(), false, true));
        metas.add(new AccountMeta(addresses.collateralVault(), false, true));
        metas.add(new AccountMeta(addresses.vaultAuthority(), false, false));
        metas.add(new AccountMeta(closer, true, true));
        metas.add(new AccountMeta(ProtocolConstants.TOKEN_PROGRAM_ID, false, false));

        return new TransactionInstruction(programId, metas, data);
    }
}
