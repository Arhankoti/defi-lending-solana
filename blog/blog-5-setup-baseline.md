# 5. Scaling Blockchains: Layer 2s, and Why Solana Took a Different Road

*Building a DeFi lending protocol on Solana · October 2026*

Welcome back! In post 4 I promised to look at scalability: why blockchains get slow and expensive, and what people do
about it. This post does that, and it also starts a new phase of the series. Up to now I was reading and writing about
blockchains. From here on I'm building a lending protocol on Solana, one small step per post, and every claim will
trace back to a number my code produced.

## Where we left off

Post 4 ended with smart contracts and a teaser about Layer 2s. I also have a first version of the lending protocol
(I'll call it **v0**) that I wrote before starting this series. Before I change anything, I want to know what v0
really does and what using the chain costs. A baseline.

## The problem: blockchains are shared computers

A blockchain is one computer that thousands of machines must all agree on. Every transaction competes for limited
space in the next block. When demand goes up, either fees go up or the chain slows down. Two broad answers exist:

- **Layer 2s (L2s)** keep the original chain (the "Layer 1") as a settlement court, and move most of the work onto
  a second system that periodically posts a compact summary back to Layer 1. You trade some extra complexity (and
  extra trust assumptions) for cheaper transactions.
- **A faster Layer 1**, which is the road Solana took: make the base chain itself handle more, by demanding more
  from the machines that run it (fast hardware and networks) and by letting transactions that touch different
  accounts run in parallel.

I'm not going to claim here which is "better." I haven't measured any L2, so I have no numbers for one. What I can do is
measure the Solana side of the trade on my own machine.

## How Solana's accounts show up in my code

On Solana, a program doesn't keep its own storage. All data lives in **accounts**, and every transaction lists the
accounts it will touch up front. That is what makes parallel execution possible: two transactions that touch
different accounts can't interfere. It also means I pay **rent** for storing data. An account must hold a minimum
balance (the **rent-exempt minimum**) that depends on its size in bytes, and as long as it holds that balance it
stays alive.

```
 transaction ──lists──> [account A] [account B] [account C]
 transaction ──lists──> [account D] [account E]            <- touches different accounts, can run in parallel
```

## Results: what v0 does and what the chain charges

All numbers below are from my machine, run on 25 Oct 2026 with Solana CLI 1.18.26 and Anchor 0.30.1, against a local
test validator (a private chain running on my laptop, so the fee schedule is Solana's default but nobody else is on it).

| Measurement | Result |
|---|---|
| v0 Rust unit tests (`cargo test`, tag `v0-reference`) | 14 passed |
| v0 `anchor build` as committed | **failed**: "`overflow-checks` is not enabled" |
| Rust unit tests after my build fixes | 14 passed |
| TypeScript integration tests (`anchor test`) after fixes | 15 passing (one test waits 65 s for a loan to expire; the run took about a minute) |
| `cargo clippy -- -D warnings` after fixes | clean |
| Fee for a simple transfer of 0.1 SOL | 5,000 lamports (0.000005 SOL) |

Rent-exempt minimum on the local validator:

| Account size | Minimum balance |
|---|---|
| 0 bytes | 0.00089088 SOL |
| 200 bytes | 0.00228288 SOL |
| 240 bytes | 0.00256128 SOL |

A **lamport** is the smallest unit of SOL (1 SOL = 1,000,000,000 lamports). The rent numbers fit a simple formula: every
account costs a fixed 128-byte overhead plus its own size, at 6,960 lamports per byte. For example (200 + 128) × 6,960 =
2,282,880 lamports = 0.00228288 SOL. I checked this arithmetic against the table, not against the docs.

## The catch

The honest headline of this part is that **v0 did not build**. `anchor build` refuses to compile unless
`overflow-checks` (Rust's check that crashes on arithmetic overflow instead of silently wrapping) is turned on for
release builds, and the project was missing that and a few other things. The fixes I needed:

- `[profile.release] overflow-checks = true` in the workspace `Cargo.toml`
- an `idl-build` feature so Anchor can generate the program's interface file
- `declare_id!` (the program's address inside the code) changed to match the program's keypair
- pinned dependency versions that Solana's older Rust toolchain can compile
- formatting and a handful of clippy warnings

None of these change what the program does, which is why `cargo test` passes on v0 and after the fixes alike (14 and 14).
I also found that the old README promised GitHub Actions CI that never existed. I removed that claim; real CI arrives
in Part 8. And a `target/` build folder had been committed to git; it's now untracked and ignored.

What I did *not* do: I didn't measure any Layer 2, and I haven't tested the protocol on devnet yet (Part 9). I have
no keypair-backed deployment numbers in this post.

## What I built

- A complete `.gitignore` (build output, `node_modules`, ledgers, keypairs, `.env`)
- `blog/blog-index.md`, listing every post
- A tag, `v0-reference`, on the original commit, so every later Part can compare against v0
- A building, linting, passing baseline (14 Rust tests, 15 integration tests)

Code: tag `part-5` in the repo.

## Where this goes next

The program stores everything in two accounts, a **Market** and a **Loan**, and I just learned that the size of an account
sets the rent it needs. In Part 6 I look at how Market is laid out, replace a hand-counted size constant with a
derived one, and fix the places where the market's limits can be set to nonsense values.

---

Previous: [4. Smart Contracts](blog-4-smart-contracts.md) · [Blog index](blog-index.md) · Next: 6. Accounts, PDAs, and the Market (coming soon)
