package com.defi.lending;

import com.defi.lending.model.LoanAccount;
import com.defi.lending.model.LoanRequest;
import com.defi.lending.model.MarketConfig;
import org.p2p.solanaj.core.Account;
import org.p2p.solanaj.core.PublicKey;
import org.p2p.solanaj.core.Transaction;
import org.p2p.solanaj.core.TransactionInstruction;
import org.p2p.solanaj.rpc.RpcClient;

public final class LendingProtocolClient {
    private final RpcClient rpcClient;
    private final PublicKey programId;
    private final LendingInstructionBuilder instructionBuilder;

    public LendingProtocolClient(String rpcUrl, PublicKey programId) {
        this.rpcClient = new RpcClient(rpcUrl);
        this.programId = programId;
        this.instructionBuilder = new LendingInstructionBuilder(programId);
    }

    public PublicKey getProgramId() {
        return programId;
    }

    public PublicKey deriveMarketAddress(
        PublicKey admin,
        PublicKey principalMint,
        PublicKey collateralMint
    ) {
        return PdaUtil.deriveMarketAddress(programId, admin, principalMint, collateralMint);
    }

    public LoanAddresses deriveLoanAddresses(
        PublicKey market,
        PublicKey borrower,
        long loanSeed
    ) {
        return PdaUtil.deriveLoanAddresses(programId, market, borrower, loanSeed);
    }

    public String initializeMarket(
        Account adminSigner,
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
        PublicKey market = deriveMarketAddress(
            adminSigner.getPublicKey(),
            principalMint,
            collateralMint
        );
        TransactionInstruction instruction = instructionBuilder.buildInitializeMarket(
            market,
            adminSigner.getPublicKey(),
            principalMint,
            collateralMint,
            treasuryPrincipalAta,
            protocolFeeBps,
            minCollateralRatioBps,
            gracePeriodSecs,
            originationFeeBps,
            supplyCap,
            borrowCap
        );
        return sendSingleInstruction(adminSigner, instruction);
    }

    public String updateMarket(
        Account adminSigner,
        PublicKey market,
        int protocolFeeBps,
        int minCollateralRatioBps,
        long gracePeriodSecs,
        boolean paused,
        int originationFeeBps,
        long supplyCap,
        long borrowCap
    ) {
        return sendSingleInstruction(
            adminSigner,
            instructionBuilder.buildUpdateMarket(
                market,
                adminSigner.getPublicKey(),
                protocolFeeBps,
                minCollateralRatioBps,
                gracePeriodSecs,
                paused,
                originationFeeBps,
                supplyCap,
                borrowCap
            )
        );
    }

    public LoanAddresses createLoanRequest(
        Account borrowerSigner,
        PublicKey market,
        PublicKey borrowerCollateralAta,
        PublicKey collateralMint,
        LoanRequest request
    ) {
        LoanAddresses addresses = deriveLoanAddresses(
            market,
            borrowerSigner.getPublicKey(),
            request.loanSeed()
        );
        sendSingleInstruction(
            borrowerSigner,
            instructionBuilder.buildCreateLoanRequest(
                market,
                borrowerSigner.getPublicKey(),
                borrowerCollateralAta,
                collateralMint,
                addresses,
                request
            )
        );
        return addresses;
    }

    public String cancelLoanRequest(
        Account borrowerSigner,
        PublicKey market,
        PublicKey borrowerCollateralAta,
        LoanAddresses addresses
    ) {
        return sendSingleInstruction(
            borrowerSigner,
            instructionBuilder.buildCancelLoanRequest(
                market,
                borrowerSigner.getPublicKey(),
                borrowerCollateralAta,
                addresses
            )
        );
    }

    public String fundLoan(
        Account lenderSigner,
        PublicKey market,
        PublicKey borrower,
        PublicKey lenderPrincipalAta,
        PublicKey borrowerPrincipalAta,
        PublicKey treasuryPrincipalAta,
        LoanAddresses addresses
    ) {
        return sendSingleInstruction(
            lenderSigner,
            instructionBuilder.buildFundLoan(
                market,
                borrower,
                lenderSigner.getPublicKey(),
                lenderPrincipalAta,
                borrowerPrincipalAta,
                treasuryPrincipalAta,
                addresses
            )
        );
    }

    public String repayLoan(
        Account borrowerSigner,
        PublicKey market,
        PublicKey borrowerPrincipalAta,
        PublicKey lender,
        PublicKey lenderPrincipalAta,
        PublicKey treasuryPrincipalAta,
        PublicKey borrowerCollateralAta,
        LoanAddresses addresses
    ) {
        return sendSingleInstruction(
            borrowerSigner,
            instructionBuilder.buildRepayLoan(
                market,
                borrowerSigner.getPublicKey(),
                borrowerPrincipalAta,
                lender,
                lenderPrincipalAta,
                treasuryPrincipalAta,
                borrowerCollateralAta,
                addresses
            )
        );
    }

    public String liquidateLoan(
        Account liquidatorSigner,
        PublicKey market,
        PublicKey lender,
        PublicKey lenderCollateralAta,
        LoanAddresses addresses
    ) {
        return sendSingleInstruction(
            liquidatorSigner,
            instructionBuilder.buildLiquidateLoan(
                market,
                liquidatorSigner.getPublicKey(),
                lender,
                lenderCollateralAta,
                addresses
            )
        );
    }

    public String partialRepay(
        Account borrowerSigner,
        PublicKey market,
        PublicKey lender,
        PublicKey borrowerPrincipalAta,
        PublicKey lenderPrincipalAta,
        PublicKey treasuryPrincipalAta,
        LoanAddresses addresses,
        long amount
    ) {
        return sendSingleInstruction(
            borrowerSigner,
            instructionBuilder.buildPartialRepay(
                market,
                borrowerSigner.getPublicKey(),
                lender,
                borrowerPrincipalAta,
                lenderPrincipalAta,
                treasuryPrincipalAta,
                addresses,
                amount
            )
        );
    }

    public String closeLoan(
        Account closerSigner,
        LoanAddresses addresses
    ) {
        return sendSingleInstruction(
            closerSigner,
            instructionBuilder.buildCloseLoan(closerSigner.getPublicKey(), addresses)
        );
    }

    public MarketConfig decodeMarket(byte[] accountData) {
        return AccountDecoder.decodeMarket(accountData);
    }

    public LoanAccount decodeLoan(byte[] accountData) {
        return AccountDecoder.decodeLoan(accountData);
    }

    private String sendSingleInstruction(Account signer, TransactionInstruction instruction) {
        try {
            Transaction transaction = new Transaction();
            transaction.addInstruction(instruction);
            return rpcClient.getApi().sendTransaction(transaction, signer);
        } catch (Exception e) {
            throw new RuntimeException("Failed to send transaction", e);
        }
    }
}
