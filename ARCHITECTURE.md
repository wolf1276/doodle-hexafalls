# Escrow Program — Architecture

Status: **Production Ready**. Program: `programs/escrow` (Anchor, `declare_id!("FFJ8YAVGUJP4SeDZrQ3g1d9fdQFq9hutsU1m4f3o1UXS")`).

## 1. Protocol Overview

PayGig splits on-chain responsibility into independent programs instead of one monolith:

```
┌─────────────┐      ┌──────────────┐      ┌────────────────┐
│   Escrow     │      │  Reputation  │      │    Dispute      │
│  (payments)  │      │  (scoring)   │      │  (not yet live) │
└─────────────┘      └──────────────┘      └────────────────┘
```

The Escrow program is the only one live and audited today. It owns exactly one job: **hold client funds and release them to the freelancer according to a fixed set of rules.** It has no knowledge of ratings, disputes, or reputation.

## 2. Why Escrow Only Manages Payments

- **Smaller attack surface.** A program that only moves SPL tokens between a vault and two known parties is far easier to reason about, audit, and formally enumerate invariants for than a program that also handles voting, badge minting, or arbitrary off-chain rating input.
- **Independent upgrade paths.** Reputation scoring rules and dispute-resolution mechanics will change far more often (game-design tuning, jury economics) than payment/escrow rules should. Coupling them would force the security-critical vault logic to be re-reviewed every time an unrelated scoring tweak ships.
- **Failure isolation.** If the reputation or dispute program has a bug, funds already locked in an escrow vault are unaffected — the vault's authority is a PDA owned solely by the escrow program's own seeds, not by any cross-program state.
- **No unnecessary CPI surface.** The current implementation performs zero outbound CPIs to other custom programs — only inbound CPIs to the SPL Token program for `transfer_checked`. Every code path that moves value is visible in `programs/escrow/src/instructions/`.

## 3. Why Reputation and Disputes Are Separate Programs

The product vision (`docs/details.md`) describes future composition — escrow calling into reputation on approval, escrow handing off to a dispute program on conflict. Deliberately, the audited escrow program **does not implement those CPIs yet**. Reputation (`programs/reputation`) is a standalone, independently-deployed program today; the dispute program (`programs/dispute`) is not yet implemented (placeholder only).

This separation is intentional, not an oversight:

- A rating or a badge mint is a *side effect* of a client being happy, not a precondition of the client getting what they paid for. Escrow must succeed even if reputation-scoring logic is unavailable, buggy, or mid-upgrade.
- Cross-program calls widen the trust boundary of the escrow program to include the reputation program's account validation. Keeping them separate means an escrow security audit doesn't have to also audit reputation's account constraints.
- Future CPI wiring (escrow → reputation on `approve_milestone`) can be added later as an additive change without touching vault custody logic, once the reputation program has undergone its own audit.

## 4. Account Architecture

### 4.1 Gig Account (PDA)

```
Gig {
  id: u64
  client: Pubkey
  freelancer: Pubkey
  mint: Pubkey            // SPL mint every milestone is funded in (e.g. USDC)
  milestone_count: u32
  active_milestone: u32
  status: GigStatus        // Active | Completed | Cancelled
  created_at: i64
  bump: u8
}
```

One `Gig` account exists per engagement between a client and a freelancer. It is the root of trust for every other account in the tree — `has_one` constraints on `client`/`freelancer` throughout the program all resolve back to this account.

### 4.2 Milestone Account (PDA)

```
Milestone {
  gig: Pubkey
  index: u32
  amount: u64
  released: u64
  status: MilestoneStatus  // PendingFunding | Funded | Submitted | PartialReleased | Completed
  submitted_at: i64
  approved_at: i64
  bump: u8
}
```

Milestones are created sequentially (`index` == `gig.milestone_count` at creation time) and are independently funded, delivered, and released. `released` tracks cumulative payout so partial timeout releases and the final release never double-pay.

### 4.3 Vault (EscrowVault PDA + Vault Token Account)

```
EscrowVault {
  gig: Pubkey
  token_account: Pubkey
  mint: Pubkey
  total_locked: u64
  total_released: u64
  bump: u8
}
```

One vault per `Gig`, shared by every milestone under that gig. `EscrowVault` is a small bookkeeping account; the actual SPL tokens live in a separate **Vault Token Account** (an `spl-token` `TokenAccount` whose `authority` is the `EscrowVault` PDA itself). `total_locked`/`total_released` are running counters checked against actual token balances by the test suite (see `tests/vault_accounting.rs`).

### 4.4 Account Relationships

```
Gig (1) ──┬──< Milestone (N, seeded by gig + index)
          │
          └──< EscrowVault (1, seeded by gig)
                    │
                    └── Vault Token Account (1, seeded by gig + "token", authority = EscrowVault)
```

## 5. Instruction Flow / State Machine

```
initialize_gig
      │
      ▼
create_milestone ──► Milestone::PendingFunding
      │
      ▼ fund_milestone (client transfer_checked → vault)
Milestone::Funded
      │
      ▼ submit_delivery (freelancer)
Milestone::Submitted ──► submitted_at = now
      │
      ├── approve_milestone (client, any time) ─────────► Milestone::Completed  (100% remaining released)
      │
      ├── partial_timeout_release (permissionless, now ≥ submitted_at + 72h)
      │         └──► Milestone::PartialReleased (20% released)
      │                     │
      │                     └── full_timeout_release (permissionless, now ≥ submitted_at + 7d)
      │                               └──► Milestone::Completed (remaining 80% released)
```

`cancel_before_funding` is the only exit from `PendingFunding`: it closes the milestone (rent refunded to client) and marks the gig `Cancelled`. It is unavailable once a milestone has been funded — funds can only leave the vault through `approve_milestone`, `partial_timeout_release`, or `full_timeout_release`.

When the last milestone under a gig reaches `Completed` (via approval or full timeout), `Gig.status` flips to `Completed` in the same instruction — there is no separate "close gig" step.

## 6. Event Architecture

Every state-changing instruction emits a typed Anchor event (`programs/escrow/src/events.rs`), giving off-chain indexers (e.g. `services/reputation-indexer`) a complete, replayable log without needing to poll account state:

| Event | Emitted by |
|---|---|
| `GigCreated` | `initialize_gig` |
| `MilestoneCreated` | `create_milestone` |
| `MilestoneFunded` | `fund_milestone` |
| `DeliverySubmitted` | `submit_delivery` |
| `MilestoneApproved` | `approve_milestone` |
| `PartialReleaseExecuted` | `partial_timeout_release` |
| `FullReleaseExecuted` | `full_timeout_release` |
| `GigCancelled` | `cancel_before_funding` |

Coverage is verified directly (`tests/events.rs`, 10 tests) — every instruction's event fields are asserted against the instruction's actual effects.

## 7. Token Flow / SPL Token CPI Architecture

The program never holds token authority itself — it always CPIs into the SPL Token program using `transfer_checked` (mint-aware, decimal-checked transfers, not the legacy `transfer`):

```
fund_milestone:
  client_token_account ──transfer_checked (client signs)──► vault_token_account

approve_milestone / partial_timeout_release / full_timeout_release:
  vault_token_account ──transfer_checked (EscrowVault PDA signs via seeds)──► freelancer_token_account
```

- **Inbound (funding):** the client is the transaction signer and the token-account authority; the program does not need PDA signing to pull funds in.
- **Outbound (release):** only the `EscrowVault` PDA can authorize a debit from the vault token account. The program supplies `signer_seeds` derived from `[VAULT_SEED, gig, bump]` to `CpiContext::new_with_signer`, so only this exact program, for this exact gig, can move vault funds.
- `mint.decimals` is passed to every `transfer_checked` call and the mint is constrained (`has_one = mint`, `token::mint = mint`) at every account boundary that touches a token account, so a caller cannot substitute a token account belonging to a different mint.

## 8. PDA Architecture

### 8.1 Why PDAs

Every stateful account in the program — `Gig`, `Milestone`, `EscrowVault`, and the vault's SPL token account — is a Program Derived Address. A PDA is computed deterministically from a set of seed bytes plus the program ID; it has **no corresponding private key**, so nothing can produce a valid `Ed25519` signature for it. The only way to authorize an action "as" a PDA is for the *owning program* to invoke `invoke_signed` with the exact seeds that derive it. This is what lets the escrow program act as a custodian: it can prove control of vault funds without ever holding a private key that could leak, be phished, or be reused elsewhere.

### 8.2 Deterministic Address Generation

Every PDA in this program is derived from `Pubkey::find_program_address(seeds, program_id)`, where the seeds tie the account unambiguously to the data it represents:

| PDA | Seeds | Rationale |
|---|---|---|
| **Gig PDA** | `[GIG_SEED, id.to_le_bytes()]` | `id` is caller-supplied and unique per gig; deriving from it means the client and freelancer never need to pass around a generated address out-of-band — anyone can recompute the gig's address from its `id` alone |
| **Milestone PDA** | `[MILESTONE_SEED, gig.key(), index.to_le_bytes()]` | Binds every milestone to its exact parent gig and its sequential position; two different gigs can never collide on a milestone address, and milestone `N` for a gig is always at the same deterministic address |
| **Vault PDA (`EscrowVault`)** | `[VAULT_SEED, gig.key()]` | One vault per gig; deriving from the gig alone (not from a milestone) is what lets multiple milestones share a single vault and its running `total_locked`/`total_released` counters |
| **Vault Token Account** | `[VAULT_SEED, gig.key(), b"token"]` | A distinct sub-seed from the `EscrowVault` bookkeeping PDA, so the SPL token account and the bookkeeping struct are two separately-addressable accounts even though they represent the same vault |

`GIG_SEED`, `MILESTONE_SEED`, and `VAULT_SEED` are `#[constant]` byte strings (`programs/escrow/src/constants.rs`), so they are also embedded in the program's IDL for client-side address derivation.

### 8.3 Bump Seeds and Program Signing

Each PDA account stores its own `bump: u8`, captured at creation time (`ctx.bumps.<account>`) via Anchor's `seeds`/`bump` account constraints. Storing the bump (rather than recomputing it on every instruction) is both a gas/compute optimization and a security property: instructions that need to *sign* on behalf of a PDA (`approve_milestone`, `partial_timeout_release`, `full_timeout_release`) build `signer_seeds` from the *stored* bump —

```rust
let signer_seeds: &[&[&[u8]]] = &[&[VAULT_SEED, gig_key.as_ref(), &[vault_bump]]];
```

— and pass them to `CpiContext::new_with_signer`. The runtime independently re-derives the PDA from these seeds and confirms it matches the account being signed for; a caller cannot forge a signature for the vault by supplying a different bump, because Anchor's `bump = vault.bump` constraint on the account already validated that the stored bump reproduces the vault's actual address before the instruction body ever runs.

### 8.4 Why PDAs Have No Private Keys — and Why That Matters Here

Because a PDA sits deliberately off the Ed25519 curve, no keypair exists that could sign for it directly. This is the entire basis of the escrow's custody guarantee:

- **Only the Escrow Program controls escrow funds.** The vault token account's SPL `authority` is set to the `EscrowVault` PDA (`token::authority = vault` in `fund_milestone`). Since nothing can sign for that PDA except an `invoke_signed` call issued by this exact program with these exact seeds, no other program, wallet, or validator operator can move vault funds — not even the client who funded it, and not even the deployer.
- **Clients and freelancers cannot directly move funds.** A client cannot call `spl-token transfer` against the vault token account, because a `TokenAccount`'s SPL-level authority is the PDA, not any user wallet. The only paths that debit the vault are the three release instructions, each of which is gated by its own status/timing/signer checks before the CPI is even reached.
- **PDA validation prevents spoofing.** Every instruction that touches a PDA re-derives and checks it via Anchor's `seeds`/`bump` (on init) or `seeds`/`bump = stored_bump` (on subsequent use) constraints, plus explicit `has_one`/`constraint`/`address` checks tying the PDA back to the expected `Gig`/`Milestone`/mint. An attacker cannot substitute an attacker-controlled account claiming to be "the vault" — Anchor rejects any account whose address doesn't match the expected derivation, and `tests/pda_security.rs` (8 tests) exercises exactly this: wrong gig PDA, wrong milestone PDA, wrong vault PDA, wrong bump, milestone-from-a-different-gig, vault-token mismatch, cross-gig vault substitution, and uninitialized spoofed PDAs are all asserted to fail.

## 9. Security Model (summary — full detail in SECURITY.md)

The vault token account's SPL `authority` field is the `EscrowVault` PDA, which has no private key. The only way to debit it is a CPI signed with that PDA's exact seeds — meaning the *only* code path capable of moving vault funds is the escrow program's own `approve_milestone` / `partial_timeout_release` / `full_timeout_release` handlers, each of which independently re-derives the vault from `[VAULT_SEED, gig]` and checks `vault.bump` and `vault.mint`. See [SECURITY.md](./SECURITY.md).

## 10. Design Rationale Summary

| Decision | Why |
|---|---|
| Milestones are separate PDAs, not a `Vec<Milestone>` inside `Gig` | Fixed account size, no realloc/resize logic, no per-gig cap on milestone count, cheaper writes (only the touched milestone is written) |
| One vault per gig, shared across milestones | Avoids re-creating a token account (and its rent) per milestone; simplifies accounting to one `total_locked`/`total_released` pair per engagement |
| `released` tracked per-milestone, not inferred from balance | Vault token balance alone cannot distinguish "not yet funded" from "already fully paid out" once multiple milestones share a vault |
| Timeout releases are permissionless | A silent client cannot hold a freelancer's payment hostage indefinitely by simply not signing anything — anyone (including automation) can trigger the timeout instructions once the deadline passes |
| Partial (20%) before full (7d) timeout | Gives the freelancer partial relief quickly (72h) while still preserving the client's ability to dispute or object within a longer window before the remainder auto-releases |
| No CPI to reputation/dispute programs | See §3 — isolates the audited payment path from higher-churn, less-audited logic |
