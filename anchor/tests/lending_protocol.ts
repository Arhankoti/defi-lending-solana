import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LendingProtocol } from "../target/types/lending_protocol";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";
import BN from "bn.js";

describe("lending_protocol", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.LendingProtocol as Program<LendingProtocol>;
  const admin = provider.wallet as anchor.Wallet;

  let principalMint: PublicKey;
  let collateralMint: PublicKey;
  let treasuryPrincipalAta: PublicKey;
  let marketPda: PublicKey;
  let marketBump: number;

  // Borrower
  const borrower = Keypair.generate();
  let borrowerPrincipalAta: PublicKey;
  let borrowerCollateralAta: PublicKey;

  // Lender
  const lender = Keypair.generate();
  let lenderPrincipalAta: PublicKey;
  let lenderCollateralAta: PublicKey;

  const PROTOCOL_FEE_BPS = 500; // 5%
  const MIN_COLLATERAL_RATIO_BPS = 15000; // 150%
  const GRACE_PERIOD_SECS = new BN(3600); // 1 hour
  const ORIGINATION_FEE_BPS = 100; // 1%
  const SUPPLY_CAP = new BN(0); // unlimited
  const BORROW_CAP = new BN(0); // unlimited

  const PRINCIPAL_AMOUNT = new BN(1_000_000);
  const COLLATERAL_AMOUNT = new BN(1_500_000);
  const INTEREST_BPS = 1000; // 10%
  const DURATION_SECS = new BN(86400); // 1 day
  const LOAN_SEED = new BN(1);

  before(async () => {
    // Airdrop SOL to borrower and lender
    const airdropBorrower = await provider.connection.requestAirdrop(
      borrower.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropBorrower);

    const airdropLender = await provider.connection.requestAirdrop(
      lender.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropLender);

    // Create mints
    principalMint = await createMint(
      provider.connection,
      (admin as any).payer,
      admin.publicKey,
      null,
      6
    );

    collateralMint = await createMint(
      provider.connection,
      (admin as any).payer,
      admin.publicKey,
      null,
      6
    );

    // Create token accounts
    treasuryPrincipalAta = await createAccount(
      provider.connection,
      (admin as any).payer,
      principalMint,
      admin.publicKey
    );

    borrowerPrincipalAta = await createAccount(
      provider.connection,
      (admin as any).payer,
      principalMint,
      borrower.publicKey
    );

    borrowerCollateralAta = await createAccount(
      provider.connection,
      (admin as any).payer,
      collateralMint,
      borrower.publicKey
    );

    lenderPrincipalAta = await createAccount(
      provider.connection,
      (admin as any).payer,
      principalMint,
      lender.publicKey
    );

    lenderCollateralAta = await createAccount(
      provider.connection,
      (admin as any).payer,
      collateralMint,
      lender.publicKey
    );

    // Mint tokens
    await mintTo(
      provider.connection,
      (admin as any).payer,
      collateralMint,
      borrowerCollateralAta,
      admin.publicKey,
      10_000_000
    );

    await mintTo(
      provider.connection,
      (admin as any).payer,
      principalMint,
      lenderPrincipalAta,
      admin.publicKey,
      10_000_000
    );

    await mintTo(
      provider.connection,
      (admin as any).payer,
      principalMint,
      borrowerPrincipalAta,
      admin.publicKey,
      10_000_000
    );

    // Derive market PDA
    [marketPda, marketBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("market"),
        admin.publicKey.toBuffer(),
        principalMint.toBuffer(),
        collateralMint.toBuffer(),
      ],
      program.programId
    );
  });

  // ---------- Market Tests ----------

  it("initializes a market", async () => {
    await program.methods
      .initializeMarket(
        PROTOCOL_FEE_BPS,
        MIN_COLLATERAL_RATIO_BPS,
        GRACE_PERIOD_SECS,
        ORIGINATION_FEE_BPS,
        SUPPLY_CAP,
        BORROW_CAP
      )
      .accounts({
        market: marketPda,
        admin: admin.publicKey,
        principalMint,
        collateralMint,
        treasuryPrincipalAta,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const market = await program.account.market.fetch(marketPda);
    assert.equal(market.admin.toBase58(), admin.publicKey.toBase58());
    assert.equal(market.principalMint.toBase58(), principalMint.toBase58());
    assert.equal(market.collateralMint.toBase58(), collateralMint.toBase58());
    assert.equal(market.protocolFeeBps, PROTOCOL_FEE_BPS);
    assert.equal(market.minCollateralRatioBps, MIN_COLLATERAL_RATIO_BPS);
    assert.equal(market.originationFeeBps, ORIGINATION_FEE_BPS);
    assert.isFalse(market.isPaused);
  });

  it("updates a market", async () => {
    await program.methods
      .updateMarket(
        PROTOCOL_FEE_BPS,
        MIN_COLLATERAL_RATIO_BPS,
        GRACE_PERIOD_SECS,
        false,
        200, // new origination fee
        new BN(5_000_000), // supply cap
        new BN(0)
      )
      .accounts({
        market: marketPda,
        admin: admin.publicKey,
      })
      .rpc();

    const market = await program.account.market.fetch(marketPda);
    assert.equal(market.originationFeeBps, 200);
    assert.equal(market.supplyCap.toNumber(), 5_000_000);

    // Reset origination fee for subsequent tests
    await program.methods
      .updateMarket(
        PROTOCOL_FEE_BPS,
        MIN_COLLATERAL_RATIO_BPS,
        GRACE_PERIOD_SECS,
        false,
        ORIGINATION_FEE_BPS,
        SUPPLY_CAP,
        BORROW_CAP
      )
      .accounts({
        market: marketPda,
        admin: admin.publicKey,
      })
      .rpc();
  });

  it("fails to initialize market with same mints", async () => {
    try {
      const [badMarket] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("market"),
          admin.publicKey.toBuffer(),
          principalMint.toBuffer(),
          principalMint.toBuffer(),
        ],
        program.programId
      );

      await program.methods
        .initializeMarket(
          PROTOCOL_FEE_BPS,
          MIN_COLLATERAL_RATIO_BPS,
          GRACE_PERIOD_SECS,
          ORIGINATION_FEE_BPS,
          SUPPLY_CAP,
          BORROW_CAP
        )
        .accounts({
          market: badMarket,
          admin: admin.publicKey,
          principalMint,
          collateralMint: principalMint, // same as principal
          treasuryPrincipalAta,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have thrown");
    } catch (err) {
      assert.include(err.toString(), "DistinctMintsRequired");
    }
  });

  // ---------- Loan Lifecycle Tests ----------

  let loanPda: PublicKey;
  let vaultAuthorityPda: PublicKey;
  let collateralVaultPda: PublicKey;

  it("creates a loan request", async () => {
    [loanPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("loan"),
        marketPda.toBuffer(),
        borrower.publicKey.toBuffer(),
        LOAN_SEED.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), loanPda.toBuffer()],
      program.programId
    );

    [collateralVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_vault"), loanPda.toBuffer()],
      program.programId
    );

    await program.methods
      .createLoanRequest(
        LOAN_SEED,
        PRINCIPAL_AMOUNT,
        COLLATERAL_AMOUNT,
        INTEREST_BPS,
        DURATION_SECS
      )
      .accounts({
        market: marketPda,
        loan: loanPda,
        vaultAuthority: vaultAuthorityPda,
        collateralVault: collateralVaultPda,
        borrower: borrower.publicKey,
        borrowerCollateralAta,
        collateralMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([borrower])
      .rpc();

    const loan = await program.account.loan.fetch(loanPda);
    assert.equal(loan.borrower.toBase58(), borrower.publicKey.toBase58());
    assert.equal(loan.principalAmount.toNumber(), PRINCIPAL_AMOUNT.toNumber());
    assert.equal(
      loan.collateralAmount.toNumber(),
      COLLATERAL_AMOUNT.toNumber()
    );
    assert.deepEqual(loan.status, { requested: {} });
    assert.equal(loan.principalRepaid.toNumber(), 0);
    assert.equal(loan.accruedInterest.toNumber(), 0);

    // Verify collateral was transferred to vault
    const vault = await getAccount(provider.connection, collateralVaultPda);
    assert.equal(Number(vault.amount), COLLATERAL_AMOUNT.toNumber());
  });

  it("fails to create loan with insufficient collateral ratio", async () => {
    const badSeed = new BN(999);
    const [badLoan] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("loan"),
        marketPda.toBuffer(),
        borrower.publicKey.toBuffer(),
        badSeed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    const [badVaultAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), badLoan.toBuffer()],
      program.programId
    );
    const [badVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_vault"), badLoan.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .createLoanRequest(
          badSeed,
          PRINCIPAL_AMOUNT,
          new BN(100), // way too little collateral
          INTEREST_BPS,
          DURATION_SECS
        )
        .accounts({
          market: marketPda,
          loan: badLoan,
          vaultAuthority: badVaultAuth,
          collateralVault: badVault,
          borrower: borrower.publicKey,
          borrowerCollateralAta,
          collateralMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([borrower])
        .rpc();
      assert.fail("Should have thrown");
    } catch (err) {
      assert.include(err.toString(), "CollateralRatioTooLow");
    }
  });

  it("funds a loan", async () => {
    const borrowerBalBefore = (
      await getAccount(provider.connection, borrowerPrincipalAta)
    ).amount;

    await program.methods
      .fundLoan()
      .accounts({
        market: marketPda,
        loan: loanPda,
        collateralVault: collateralVaultPda,
        borrower: borrower.publicKey,
        lender: lender.publicKey,
        lenderPrincipalAta,
        borrowerPrincipalAta,
        treasuryPrincipalAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([lender])
      .rpc();

    const loan = await program.account.loan.fetch(loanPda);
    assert.deepEqual(loan.status, { active: {} });
    assert.equal(loan.lender.toBase58(), lender.publicKey.toBase58());
    assert.isAbove(loan.fundedAt.toNumber(), 0);
    assert.isAbove(loan.dueAt.toNumber(), loan.fundedAt.toNumber());
    assert.equal(loan.lastInterestUpdate.toNumber(), loan.fundedAt.toNumber());

    // Verify origination fee: borrower gets principal minus fee
    const originationFee = Math.floor(
      (PRINCIPAL_AMOUNT.toNumber() * ORIGINATION_FEE_BPS) / 10000
    );
    const borrowerBalAfter = (
      await getAccount(provider.connection, borrowerPrincipalAta)
    ).amount;
    assert.equal(
      Number(borrowerBalAfter) - Number(borrowerBalBefore),
      PRINCIPAL_AMOUNT.toNumber() - originationFee
    );

    // Verify market counters
    const market = await program.account.market.fetch(marketPda);
    assert.equal(
      market.totalPrincipalOutstanding.toNumber(),
      PRINCIPAL_AMOUNT.toNumber()
    );
    assert.equal(
      market.totalCollateralLocked.toNumber(),
      COLLATERAL_AMOUNT.toNumber()
    );
  });

  it("fails self-lending", async () => {
    // Create a loan where borrower tries to fund their own loan
    const selfSeed = new BN(42);
    const [selfLoan] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("loan"),
        marketPda.toBuffer(),
        borrower.publicKey.toBuffer(),
        selfSeed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    const [selfVaultAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), selfLoan.toBuffer()],
      program.programId
    );
    const [selfVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_vault"), selfLoan.toBuffer()],
      program.programId
    );

    await program.methods
      .createLoanRequest(
        selfSeed,
        new BN(100_000),
        new BN(200_000),
        INTEREST_BPS,
        DURATION_SECS
      )
      .accounts({
        market: marketPda,
        loan: selfLoan,
        vaultAuthority: selfVaultAuth,
        collateralVault: selfVault,
        borrower: borrower.publicKey,
        borrowerCollateralAta,
        collateralMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([borrower])
      .rpc();

    try {
      await program.methods
        .fundLoan()
        .accounts({
          market: marketPda,
          loan: selfLoan,
          collateralVault: selfVault,
          borrower: borrower.publicKey,
          lender: borrower.publicKey, // same as borrower!
          lenderPrincipalAta: borrowerPrincipalAta,
          borrowerPrincipalAta,
          treasuryPrincipalAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([borrower])
        .rpc();
      assert.fail("Should have thrown");
    } catch (err) {
      assert.include(err.toString(), "SelfLendingNotAllowed");
    }
  });

  it("performs partial repayment", async () => {
    const repayAmount = new BN(200_000);

    await program.methods
      .partialRepay(repayAmount)
      .accounts({
        market: marketPda,
        loan: loanPda,
        borrower: borrower.publicKey,
        lender: lender.publicKey,
        borrowerPrincipalAta,
        lenderPrincipalAta,
        treasuryPrincipalAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([borrower])
      .rpc();

    const loan = await program.account.loan.fetch(loanPda);
    // Principal repaid should have increased (exact value depends on accrued interest)
    assert.isAbove(loan.principalRepaid.toNumber(), 0);
    assert.deepEqual(loan.status, { active: {} }); // still active
  });

  it("repays the remaining loan in full", async () => {
    await program.methods
      .repayLoan()
      .accounts({
        market: marketPda,
        loan: loanPda,
        collateralVault: collateralVaultPda,
        vaultAuthority: vaultAuthorityPda,
        borrower: borrower.publicKey,
        borrowerPrincipalAta,
        lender: lender.publicKey,
        lenderPrincipalAta,
        treasuryPrincipalAta,
        borrowerCollateralAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([borrower])
      .rpc();

    const loan = await program.account.loan.fetch(loanPda);
    assert.deepEqual(loan.status, { repaid: {} });
    assert.equal(loan.principalRepaid.toNumber(), PRINCIPAL_AMOUNT.toNumber());

    // Verify collateral returned
    const vault = await getAccount(provider.connection, collateralVaultPda);
    assert.equal(Number(vault.amount), 0);

    // Verify market counters decremented
    const market = await program.account.market.fetch(marketPda);
    assert.equal(market.totalPrincipalOutstanding.toNumber(), 0);
    assert.equal(market.totalCollateralLocked.toNumber(), 0);
  });

  it("closes a repaid loan", async () => {
    await program.methods
      .closeLoan()
      .accounts({
        loan: loanPda,
        collateralVault: collateralVaultPda,
        vaultAuthority: vaultAuthorityPda,
        closer: borrower.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([borrower])
      .rpc();

    // Loan account should be closed (fetch will fail)
    try {
      await program.account.loan.fetch(loanPda);
      assert.fail("Loan should be closed");
    } catch (err) {
      assert.include(err.toString(), "Account does not exist");
    }
  });

  // ---------- Cancel Loan Tests ----------

  it("cancels a loan request and returns collateral", async () => {
    const cancelSeed = new BN(100);
    const [cancelLoan] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("loan"),
        marketPda.toBuffer(),
        borrower.publicKey.toBuffer(),
        cancelSeed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    const [cancelVaultAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), cancelLoan.toBuffer()],
      program.programId
    );
    const [cancelVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_vault"), cancelLoan.toBuffer()],
      program.programId
    );

    const balBefore = (
      await getAccount(provider.connection, borrowerCollateralAta)
    ).amount;

    await program.methods
      .createLoanRequest(
        cancelSeed,
        new BN(500_000),
        new BN(750_000),
        INTEREST_BPS,
        DURATION_SECS
      )
      .accounts({
        market: marketPda,
        loan: cancelLoan,
        vaultAuthority: cancelVaultAuth,
        collateralVault: cancelVault,
        borrower: borrower.publicKey,
        borrowerCollateralAta,
        collateralMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([borrower])
      .rpc();

    await program.methods
      .cancelLoanRequest()
      .accounts({
        market: marketPda,
        loan: cancelLoan,
        collateralVault: cancelVault,
        vaultAuthority: cancelVaultAuth,
        borrower: borrower.publicKey,
        borrowerCollateralAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([borrower])
      .rpc();

    const loan = await program.account.loan.fetch(cancelLoan);
    assert.deepEqual(loan.status, { cancelled: {} });

    // Collateral returned
    const balAfter = (
      await getAccount(provider.connection, borrowerCollateralAta)
    ).amount;
    assert.equal(Number(balAfter), Number(balBefore));
  });

  // ---------- Liquidation Tests ----------

  it("liquidate_loan", async () => {
    // Create fresh borrower/lender keypairs so we don't interfere with other tests
    const liqBorrower = Keypair.generate();
    const liqLender = Keypair.generate();

    // Airdrop SOL
    const airdrop1 = await provider.connection.requestAirdrop(
      liqBorrower.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdrop1);
    const airdrop2 = await provider.connection.requestAirdrop(
      liqLender.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdrop2);

    // Create token accounts for liqBorrower and liqLender
    const liqBorrowerCollateralAta = await createAccount(
      provider.connection,
      (admin as any).payer,
      collateralMint,
      liqBorrower.publicKey
    );
    const liqBorrowerPrincipalAta = await createAccount(
      provider.connection,
      (admin as any).payer,
      principalMint,
      liqBorrower.publicKey
    );
    const liqLenderPrincipalAta = await createAccount(
      provider.connection,
      (admin as any).payer,
      principalMint,
      liqLender.publicKey
    );
    const liqLenderCollateralAta = await createAccount(
      provider.connection,
      (admin as any).payer,
      collateralMint,
      liqLender.publicKey
    );

    // Mint tokens
    await mintTo(
      provider.connection,
      (admin as any).payer,
      collateralMint,
      liqBorrowerCollateralAta,
      admin.publicKey,
      10_000_000
    );
    await mintTo(
      provider.connection,
      (admin as any).payer,
      principalMint,
      liqLenderPrincipalAta,
      admin.publicKey,
      10_000_000
    );

    // Update market: set grace_period to 0 so loan is immediately liquidatable after due_at
    await program.methods
      .updateMarket(
        PROTOCOL_FEE_BPS,
        MIN_COLLATERAL_RATIO_BPS,
        new BN(0), // grace_period_secs = 0
        false,
        ORIGINATION_FEE_BPS,
        SUPPLY_CAP,
        BORROW_CAP
      )
      .accounts({
        market: marketPda,
        admin: admin.publicKey,
      })
      .rpc();

    // Create loan request with minimum duration (60 seconds)
    const liqSeed = new BN(200);
    const liqDuration = new BN(60);
    const liqPrincipal = new BN(500_000);
    const liqCollateral = new BN(750_000);

    const [liqLoanPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("loan"),
        marketPda.toBuffer(),
        liqBorrower.publicKey.toBuffer(),
        liqSeed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    const [liqVaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), liqLoanPda.toBuffer()],
      program.programId
    );
    const [liqCollateralVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_vault"), liqLoanPda.toBuffer()],
      program.programId
    );

    await program.methods
      .createLoanRequest(
        liqSeed,
        liqPrincipal,
        liqCollateral,
        INTEREST_BPS,
        liqDuration
      )
      .accounts({
        market: marketPda,
        loan: liqLoanPda,
        vaultAuthority: liqVaultAuthority,
        collateralVault: liqCollateralVault,
        borrower: liqBorrower.publicKey,
        borrowerCollateralAta: liqBorrowerCollateralAta,
        collateralMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([liqBorrower])
      .rpc();

    // Fund the loan
    await program.methods
      .fundLoan()
      .accounts({
        market: marketPda,
        loan: liqLoanPda,
        collateralVault: liqCollateralVault,
        borrower: liqBorrower.publicKey,
        lender: liqLender.publicKey,
        lenderPrincipalAta: liqLenderPrincipalAta,
        borrowerPrincipalAta: liqBorrowerPrincipalAta,
        treasuryPrincipalAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([liqLender])
      .rpc();

    // Verify loan is active
    let loanData = await program.account.loan.fetch(liqLoanPda);
    assert.deepEqual(loanData.status, { active: {} });

    // Wait for the loan to expire (60s duration + small buffer)
    console.log("    Waiting 65 seconds for loan to expire...");
    await new Promise((resolve) => setTimeout(resolve, 65000));

    // Record lender collateral balance before liquidation
    const lenderCollateralBefore = (
      await getAccount(provider.connection, liqLenderCollateralAta)
    ).amount;

    // Liquidate the loan (anyone can call this - using admin as liquidator)
    await program.methods
      .liquidateLoan()
      .accounts({
        market: marketPda,
        loan: liqLoanPda,
        collateralVault: liqCollateralVault,
        vaultAuthority: liqVaultAuthority,
        liquidator: admin.publicKey,
        lender: liqLender.publicKey,
        lenderCollateralAta: liqLenderCollateralAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Verify loan status is Liquidated
    loanData = await program.account.loan.fetch(liqLoanPda);
    assert.deepEqual(loanData.status, { liquidated: {} });

    // Verify collateral was transferred to lender
    const lenderCollateralAfter = (
      await getAccount(provider.connection, liqLenderCollateralAta)
    ).amount;
    assert.equal(
      Number(lenderCollateralAfter) - Number(lenderCollateralBefore),
      liqCollateral.toNumber()
    );

    // Verify collateral vault is now empty
    const vault = await getAccount(provider.connection, liqCollateralVault);
    assert.equal(Number(vault.amount), 0);

    // Restore grace period for subsequent tests
    await program.methods
      .updateMarket(
        PROTOCOL_FEE_BPS,
        MIN_COLLATERAL_RATIO_BPS,
        GRACE_PERIOD_SECS,
        false,
        ORIGINATION_FEE_BPS,
        SUPPLY_CAP,
        BORROW_CAP
      )
      .accounts({
        market: marketPda,
        admin: admin.publicKey,
      })
      .rpc();
  });

  // ---------- Error Tests ----------

  it("cannot fund a paused market", async () => {
    // Create a new loan request to attempt funding
    const pauseSeed = new BN(300);
    const [pauseLoan] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("loan"),
        marketPda.toBuffer(),
        borrower.publicKey.toBuffer(),
        pauseSeed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    const [pauseVaultAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), pauseLoan.toBuffer()],
      program.programId
    );
    const [pauseVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_vault"), pauseLoan.toBuffer()],
      program.programId
    );

    await program.methods
      .createLoanRequest(
        pauseSeed,
        new BN(100_000),
        new BN(200_000),
        INTEREST_BPS,
        DURATION_SECS
      )
      .accounts({
        market: marketPda,
        loan: pauseLoan,
        vaultAuthority: pauseVaultAuth,
        collateralVault: pauseVault,
        borrower: borrower.publicKey,
        borrowerCollateralAta,
        collateralMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([borrower])
      .rpc();

    // Pause the market
    await program.methods
      .updateMarket(
        PROTOCOL_FEE_BPS,
        MIN_COLLATERAL_RATIO_BPS,
        GRACE_PERIOD_SECS,
        true, // is_paused = true
        ORIGINATION_FEE_BPS,
        SUPPLY_CAP,
        BORROW_CAP
      )
      .accounts({
        market: marketPda,
        admin: admin.publicKey,
      })
      .rpc();

    try {
      await program.methods
        .fundLoan()
        .accounts({
          market: marketPda,
          loan: pauseLoan,
          collateralVault: pauseVault,
          borrower: borrower.publicKey,
          lender: lender.publicKey,
          lenderPrincipalAta,
          borrowerPrincipalAta,
          treasuryPrincipalAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([lender])
        .rpc();
      assert.fail("Should have failed");
    } catch (err) {
      assert.include(err.toString(), "MarketPaused");
    }

    // Unpause the market
    await program.methods
      .updateMarket(
        PROTOCOL_FEE_BPS,
        MIN_COLLATERAL_RATIO_BPS,
        GRACE_PERIOD_SECS,
        false, // is_paused = false
        ORIGINATION_FEE_BPS,
        SUPPLY_CAP,
        BORROW_CAP
      )
      .accounts({
        market: marketPda,
        admin: admin.publicKey,
      })
      .rpc();
  });

  it("cannot liquidate before default", async () => {
    // Create and fund a loan with long duration
    const earlyLiqSeed = new BN(400);
    const [earlyLiqLoan] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("loan"),
        marketPda.toBuffer(),
        borrower.publicKey.toBuffer(),
        earlyLiqSeed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    const [earlyLiqVaultAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), earlyLiqLoan.toBuffer()],
      program.programId
    );
    const [earlyLiqVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_vault"), earlyLiqLoan.toBuffer()],
      program.programId
    );

    await program.methods
      .createLoanRequest(
        earlyLiqSeed,
        new BN(100_000),
        new BN(200_000),
        INTEREST_BPS,
        DURATION_SECS // 1 day - won't expire during test
      )
      .accounts({
        market: marketPda,
        loan: earlyLiqLoan,
        vaultAuthority: earlyLiqVaultAuth,
        collateralVault: earlyLiqVault,
        borrower: borrower.publicKey,
        borrowerCollateralAta,
        collateralMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([borrower])
      .rpc();

    await program.methods
      .fundLoan()
      .accounts({
        market: marketPda,
        loan: earlyLiqLoan,
        collateralVault: earlyLiqVault,
        borrower: borrower.publicKey,
        lender: lender.publicKey,
        lenderPrincipalAta,
        borrowerPrincipalAta,
        treasuryPrincipalAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([lender])
      .rpc();

    // Verify it's active
    const loanData = await program.account.loan.fetch(earlyLiqLoan);
    assert.deepEqual(loanData.status, { active: {} });

    try {
      await program.methods
        .liquidateLoan()
        .accounts({
          market: marketPda,
          loan: earlyLiqLoan,
          collateralVault: earlyLiqVault,
          vaultAuthority: earlyLiqVaultAuth,
          liquidator: admin.publicKey,
          lender: lender.publicKey,
          lenderCollateralAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      assert.fail("Should have failed");
    } catch (err) {
      assert.include(err.toString(), "LoanNotDefaulted");
    }
  });

  it("cannot cancel a funded loan", async () => {
    // Create and fund a loan, then try to cancel it
    const cancelFundedSeed = new BN(500);
    const [cancelFundedLoan] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("loan"),
        marketPda.toBuffer(),
        borrower.publicKey.toBuffer(),
        cancelFundedSeed.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    const [cancelFundedVaultAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), cancelFundedLoan.toBuffer()],
      program.programId
    );
    const [cancelFundedVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_vault"), cancelFundedLoan.toBuffer()],
      program.programId
    );

    await program.methods
      .createLoanRequest(
        cancelFundedSeed,
        new BN(100_000),
        new BN(200_000),
        INTEREST_BPS,
        DURATION_SECS
      )
      .accounts({
        market: marketPda,
        loan: cancelFundedLoan,
        vaultAuthority: cancelFundedVaultAuth,
        collateralVault: cancelFundedVault,
        borrower: borrower.publicKey,
        borrowerCollateralAta,
        collateralMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([borrower])
      .rpc();

    // Fund the loan so it becomes Active
    await program.methods
      .fundLoan()
      .accounts({
        market: marketPda,
        loan: cancelFundedLoan,
        collateralVault: cancelFundedVault,
        borrower: borrower.publicKey,
        lender: lender.publicKey,
        lenderPrincipalAta,
        borrowerPrincipalAta,
        treasuryPrincipalAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([lender])
      .rpc();

    // Verify it's active
    const loanData = await program.account.loan.fetch(cancelFundedLoan);
    assert.deepEqual(loanData.status, { active: {} });

    // Try to cancel the funded loan
    try {
      await program.methods
        .cancelLoanRequest()
        .accounts({
          market: marketPda,
          loan: cancelFundedLoan,
          collateralVault: cancelFundedVault,
          vaultAuthority: cancelFundedVaultAuth,
          borrower: borrower.publicKey,
          borrowerCollateralAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([borrower])
        .rpc();
      assert.fail("Should have failed");
    } catch (err) {
      assert.include(err.toString(), "InvalidLoanStatus");
    }
  });
});
