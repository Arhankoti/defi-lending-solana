package com.defi.lending;

import com.defi.lending.model.LoanRequest;
import org.p2p.solanaj.core.Account;
import org.p2p.solanaj.core.PublicKey;

public final class Main {
    private Main() {
    }

    public static void main(String[] args) {
        if (args.length == 0) {
            printUsage();
            return;
        }

        try {
            String command = args[0];
            switch (command) {
                case "derive-market" -> deriveMarket(args);
                case "derive-loan" -> deriveLoan(args);
                case "init-market" -> initMarket(args);
                case "create-loan" -> createLoan(args);
                case "fund-loan" -> fundLoan(args);
                case "repay-loan" -> repayLoan(args);
                case "cancel-loan" -> cancelLoan(args);
                case "liquidate-loan" -> liquidateLoan(args);
                case "partial-repay" -> partialRepay(args);
                case "close-loan" -> closeLoan(args);
                default -> {
                    System.err.println("Unknown command: " + command);
                    printUsage();
                    System.exit(1);
                }
            }
        } catch (Exception ex) {
            System.err.println("Command failed: " + ex.getMessage());
            ex.printStackTrace(System.err);
            System.exit(1);
        }
    }

    private static void deriveMarket(String[] args) {
        requireArgs(args, 5);
        PublicKey programId = new PublicKey(args[1]);
        PublicKey admin = new PublicKey(args[2]);
        PublicKey principalMint = new PublicKey(args[3]);
        PublicKey collateralMint = new PublicKey(args[4]);

        PublicKey market = PdaUtil.deriveMarketAddress(programId, admin, principalMint, collateralMint);
        System.out.println(market.toBase58());
    }

    private static void deriveLoan(String[] args) {
        requireArgs(args, 5);
        PublicKey programId = new PublicKey(args[1]);
        PublicKey market = new PublicKey(args[2]);
        PublicKey borrower = new PublicKey(args[3]);
        long loanSeed = Long.parseLong(args[4]);

        LoanAddresses addresses = PdaUtil.deriveLoanAddresses(programId, market, borrower, loanSeed);
        System.out.println("loan=" + addresses.loan().toBase58());
        System.out.println("vaultAuthority=" + addresses.vaultAuthority().toBase58());
        System.out.println("collateralVault=" + addresses.collateralVault().toBase58());
    }

    private static void initMarket(String[] args) {
        requireArgs(args, 12);
        LendingProtocolClient client = new LendingProtocolClient(args[1], new PublicKey(args[2]));
        Account adminSigner = signerFromBase58(args[3]);
        PublicKey principalMint = new PublicKey(args[4]);
        PublicKey collateralMint = new PublicKey(args[5]);
        PublicKey treasuryPrincipalAta = new PublicKey(args[6]);
        int protocolFeeBps = Integer.parseInt(args[7]);
        int minCollateralRatioBps = Integer.parseInt(args[8]);
        long gracePeriodSecs = Long.parseLong(args[9]);
        int originationFeeBps = Integer.parseInt(args[10]);
        long supplyCap = Long.parseLong(args[11]);
        long borrowCap = Long.parseLong(args[12]);

        String signature = client.initializeMarket(
            adminSigner,
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
        System.out.println(signature);
    }

    private static void createLoan(String[] args) {
        requireArgs(args, 11);
        LendingProtocolClient client = new LendingProtocolClient(args[1], new PublicKey(args[2]));
        Account borrowerSigner = signerFromBase58(args[3]);
        PublicKey market = new PublicKey(args[4]);
        PublicKey borrowerCollateralAta = new PublicKey(args[5]);
        PublicKey collateralMint = new PublicKey(args[6]);
        long loanSeed = Long.parseLong(args[7]);
        long principalAmount = Long.parseLong(args[8]);
        long collateralAmount = Long.parseLong(args[9]);
        int interestBps = Integer.parseInt(args[10]);
        long durationSecs = Long.parseLong(args[11]);

        LoanRequest request = new LoanRequest(
            loanSeed,
            principalAmount,
            collateralAmount,
            interestBps,
            durationSecs
        );
        LoanAddresses addresses = client.createLoanRequest(
            borrowerSigner,
            market,
            borrowerCollateralAta,
            collateralMint,
            request
        );
        System.out.println("loan=" + addresses.loan().toBase58());
        System.out.println("vaultAuthority=" + addresses.vaultAuthority().toBase58());
        System.out.println("collateralVault=" + addresses.collateralVault().toBase58());
    }

    private static void fundLoan(String[] args) {
        requireArgs(args, 10);
        LendingProtocolClient client = new LendingProtocolClient(args[1], new PublicKey(args[2]));
        Account lenderSigner = signerFromBase58(args[3]);
        PublicKey market = new PublicKey(args[4]);
        PublicKey borrower = new PublicKey(args[5]);
        PublicKey lenderPrincipalAta = new PublicKey(args[6]);
        PublicKey borrowerPrincipalAta = new PublicKey(args[7]);
        PublicKey treasuryPrincipalAta = new PublicKey(args[8]);
        long loanSeed = Long.parseLong(args[9]);

        LoanAddresses addresses = client.deriveLoanAddresses(market, borrower, loanSeed);
        String signature = client.fundLoan(
            lenderSigner,
            market,
            borrower,
            lenderPrincipalAta,
            borrowerPrincipalAta,
            treasuryPrincipalAta,
            addresses
        );
        System.out.println(signature);
    }

    private static void repayLoan(String[] args) {
        requireArgs(args, 11);
        LendingProtocolClient client = new LendingProtocolClient(args[1], new PublicKey(args[2]));
        Account borrowerSigner = signerFromBase58(args[3]);
        PublicKey market = new PublicKey(args[4]);
        PublicKey borrowerPrincipalAta = new PublicKey(args[5]);
        PublicKey lender = new PublicKey(args[6]);
        PublicKey lenderPrincipalAta = new PublicKey(args[7]);
        PublicKey treasuryPrincipalAta = new PublicKey(args[8]);
        PublicKey borrowerCollateralAta = new PublicKey(args[9]);
        long loanSeed = Long.parseLong(args[10]);

        LoanAddresses addresses = client.deriveLoanAddresses(
            market,
            borrowerSigner.getPublicKey(),
            loanSeed
        );
        String signature = client.repayLoan(
            borrowerSigner,
            market,
            borrowerPrincipalAta,
            lender,
            lenderPrincipalAta,
            treasuryPrincipalAta,
            borrowerCollateralAta,
            addresses
        );
        System.out.println(signature);
    }

    private static void cancelLoan(String[] args) {
        requireArgs(args, 7);
        LendingProtocolClient client = new LendingProtocolClient(args[1], new PublicKey(args[2]));
        Account borrowerSigner = signerFromBase58(args[3]);
        PublicKey market = new PublicKey(args[4]);
        PublicKey borrowerCollateralAta = new PublicKey(args[5]);
        long loanSeed = Long.parseLong(args[6]);

        LoanAddresses addresses = client.deriveLoanAddresses(
            market,
            borrowerSigner.getPublicKey(),
            loanSeed
        );
        String signature = client.cancelLoanRequest(
            borrowerSigner,
            market,
            borrowerCollateralAta,
            addresses
        );
        System.out.println(signature);
    }

    private static void liquidateLoan(String[] args) {
        requireArgs(args, 9);
        LendingProtocolClient client = new LendingProtocolClient(args[1], new PublicKey(args[2]));
        Account liquidatorSigner = signerFromBase58(args[3]);
        PublicKey market = new PublicKey(args[4]);
        PublicKey borrower = new PublicKey(args[5]);
        PublicKey lender = new PublicKey(args[6]);
        PublicKey lenderCollateralAta = new PublicKey(args[7]);
        long loanSeed = Long.parseLong(args[8]);

        LoanAddresses addresses = client.deriveLoanAddresses(market, borrower, loanSeed);
        String signature = client.liquidateLoan(
            liquidatorSigner,
            market,
            lender,
            lenderCollateralAta,
            addresses
        );
        System.out.println(signature);
    }

    private static void partialRepay(String[] args) {
        requireArgs(args, 11);
        LendingProtocolClient client = new LendingProtocolClient(args[1], new PublicKey(args[2]));
        Account borrowerSigner = signerFromBase58(args[3]);
        PublicKey market = new PublicKey(args[4]);
        PublicKey lender = new PublicKey(args[5]);
        PublicKey borrowerPrincipalAta = new PublicKey(args[6]);
        PublicKey lenderPrincipalAta = new PublicKey(args[7]);
        PublicKey treasuryPrincipalAta = new PublicKey(args[8]);
        long loanSeed = Long.parseLong(args[9]);
        long amount = Long.parseLong(args[10]);

        LoanAddresses addresses = client.deriveLoanAddresses(
            market,
            borrowerSigner.getPublicKey(),
            loanSeed
        );
        String signature = client.partialRepay(
            borrowerSigner,
            market,
            lender,
            borrowerPrincipalAta,
            lenderPrincipalAta,
            treasuryPrincipalAta,
            addresses,
            amount
        );
        System.out.println(signature);
    }

    private static void closeLoan(String[] args) {
        requireArgs(args, 7);
        LendingProtocolClient client = new LendingProtocolClient(args[1], new PublicKey(args[2]));
        Account closerSigner = signerFromBase58(args[3]);
        PublicKey market = new PublicKey(args[4]);
        PublicKey borrower = new PublicKey(args[5]);
        long loanSeed = Long.parseLong(args[6]);

        LoanAddresses addresses = client.deriveLoanAddresses(market, borrower, loanSeed);
        String signature = client.closeLoan(closerSigner, addresses);
        System.out.println(signature);
    }

    private static Account signerFromBase58(String base58SecretKey) {
        return new Account(Base58Util.decode(base58SecretKey));
    }

    private static void requireArgs(String[] args, int maxIndex) {
        if (args.length <= maxIndex) {
            printUsage();
            throw new IllegalArgumentException("Not enough arguments");
        }
    }

    private static void printUsage() {
        System.out.println("""
            Usage:
              derive-market <programId> <adminPubkey> <principalMint> <collateralMint>
              derive-loan <programId> <marketPubkey> <borrowerPubkey> <loanSeed>

              init-market <rpcUrl> <programId> <adminSecretKeyBase58> <principalMint> <collateralMint> <treasuryPrincipalAta> <protocolFeeBps> <minCollateralRatioBps> <gracePeriodSecs> <originationFeeBps> <supplyCap> <borrowCap>
              create-loan <rpcUrl> <programId> <borrowerSecretKeyBase58> <market> <borrowerCollateralAta> <collateralMint> <loanSeed> <principalAmount> <collateralAmount> <interestBps> <durationSecs>
              fund-loan <rpcUrl> <programId> <lenderSecretKeyBase58> <market> <borrowerPubkey> <lenderPrincipalAta> <borrowerPrincipalAta> <treasuryPrincipalAta> <loanSeed>
              repay-loan <rpcUrl> <programId> <borrowerSecretKeyBase58> <market> <borrowerPrincipalAta> <lenderPubkey> <lenderPrincipalAta> <treasuryPrincipalAta> <borrowerCollateralAta> <loanSeed>
              cancel-loan <rpcUrl> <programId> <borrowerSecretKeyBase58> <market> <borrowerCollateralAta> <loanSeed>
              liquidate-loan <rpcUrl> <programId> <liquidatorSecretKeyBase58> <market> <borrowerPubkey> <lenderPubkey> <lenderCollateralAta> <loanSeed>
              partial-repay <rpcUrl> <programId> <borrowerSecretKeyBase58> <market> <lenderPubkey> <borrowerPrincipalAta> <lenderPrincipalAta> <treasuryPrincipalAta> <loanSeed> <amount>
              close-loan <rpcUrl> <programId> <closerSecretKeyBase58> <market> <borrowerPubkey> <loanSeed>
            """);
    }
}
