# PayGig

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Solana](https://img.shields.io/badge/Solana-Anchor%201.1.2-14F195.svg)](https://www.anchor-lang.com/)
[![Rust](https://img.shields.io/badge/Rust-2021%20edition-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-351%20passing-brightgreen.svg)](./TESTING.md)
[![Status](https://img.shields.io/badge/status-pre--deployment-lightgrey.svg)](#roadmap)

**A non-custodial, milestone-based freelance payment protocol on Solana.** Client funds are held by program-derived vaults with no private key; payouts follow fixed on-chain rules; reputation is a deterministic function of on-chain history that anyone can recompute.

> **Maturity:** four on-chain programs (`gig`, `escrow`, `reputation`, `achievement`) are implemented and covered by 359 tests; `gig`/`escrow`/`reputation` are internally audited, `achievement` has not yet had a full audit pass. Nothing is deployed to devnet or mainnet yet, the dispute program is an empty scaffold, and the frontend is a landing-page shell. See [Implementation Matrix](#implementation-matrix) for exact per-component status.

---

## Table of Contents

- [Executive Summary](#executive-summary)
- [Problem Statement](#problem-statement)
- [Design Principles](#design-principles)
- [Implementation Matrix](#implementation-matrix)
- [System Architecture](#system-architecture)
- [Gig Program](#gig-program)
- [Escrow Program](#escrow-program)
- [Reputation Program](#reputation-program)
- [Achievement Program](#achievement-program)
- [Account Model](#account-model)
- [PDA Architecture](#pda-architecture)
- [Instruction Reference](#instruction-reference)
- [Complete Protocol Flow](#complete-protocol-flow)
- [Security Model](#security-model)
- [Testing](#testing)
- [Tech Stack](#tech-stack)
- [Repository Structure](#repository-structure)
- [Developer Guide](#developer-guide)
- [Documentation Index](#documentation-index)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Executive Summary

**Problem.** On a centralized freelance platform, one operator custodies the client's money, arbitrates whether the freelancer gets paid, and owns the freelancer's rating history. None of those three guarantees is independently verifiable by the people who depend on them.

**Solution.** PayGig moves all three onto Solana as small, separately auditable Anchor programs:

- The **gig program** owns the job. A client creates, updates, publishes, assigns, and (for gigs with no escrow activity) completes, archives, or cancels a `Gig` PDA. This used to live inside the escrow program; it was split out because gig metadata churn (title edits, category taxonomy, draft/publish workflow) has nothing to do with the custody trust boundary, and putting both in one program meant every metadata tweak sat inside the code that also moves money.
- The **escrow program** owns the money. A client funds a milestone into a vault whose SPL token authority is a PDA — an address with no private key, signable only by the escrow program itself via `invoke_signed`. Funds leave that vault through exactly three code paths: client approval, a 20% release 72 hours after delivery, or a full release 7 days after delivery. There is no admin withdrawal instruction. Escrow still needs to move the gig forward when money moves (funding starts work, a final release finishes it), so it does that over CPI into the gig program, signed by escrow's own `escrow_authority` PDA — the gig program only accepts that signature from the real escrow program, never from an arbitrary caller.
- The **reputation program** owns the record. Job counts, earnings, ratings, and badges are stored per-authority in PDAs, and `reputation_score` is recomputed from those stored counters on every mutation by a pure function — never accepted as instruction input. Anyone can re-derive the score from public account data and check it.
- The **achievement program** owns NFT credentials. It reads (never writes) a `Badge` PDA the reputation program already created and mints a Metaplex Core NFT into a user's wallet on request — never automatically, never during settlement.
- A **dispute program** is scaffolded but not implemented. Today the timeout releases, not a jury, are the freelancer's protection against a silent client.

**Core principles.** Non-custodial by construction, deterministic and recomputable, one responsibility per program, no unchecked arithmetic, no trusted mapping an admin can rewrite.

**Architecture.** Five Anchor programs. Gig and Escrow are connected by one narrow CPI edge — escrow calls into gig to advance `GigStatus` when money moves — but share no accounts and gig never calls back into escrow. Escrow's other outbound CPI is to the SPL Token program. Achievement reads Reputation's `Badge`/`UserProfile` PDAs directly (no CPI) and calls out to Metaplex Core. Reputation and Dispute perform no CPI at all and are untouched by the split.

**Maturity.** Program logic: complete and internally audited. Deployment: none. Client integration: none. See the [Implementation Matrix](#implementation-matrix).

**Target users.** Clients and freelancers who want escrowed milestone payments without a custodial intermediary; external platforms that want to read a portable reputation record; auditors and integrators who need every rule to be visible in program code.

---

## Problem Statement

Traditional freelance platforms concentrate five distinct powers in one operator.

**Centralized custody.** The platform holds the client's deposit in its own bank or wallet. Users have a claim against a company, not control of an asset. Operator insolvency, commingling, or a frozen account all reach the user's money.

**Payment disputes.** Whether the freelancer gets paid is a support-ticket decision, made under rules that are not published, not consistent, and not appealable to anything the user can inspect.

**Trust assumptions that cannot be checked.** "Funds are held in escrow" is a claim. Nothing lets a freelancer verify that the money for their milestone exists and is earmarked for them.

**Platform lock-in.** Leaving costs a freelancer their entire track record, because the record lives in the platform's database.

**Opaque reputation.** Scores are computed by undisclosed formulas over data the platform can edit. A freelancer cannot prove their own history, and a client cannot verify the number they are trusting.

### How PayGig addresses each

| Problem | Mechanism | Where enforced |
|---|---|---|
| Centralized custody | Vault token account's SPL authority is the `EscrowVault` PDA. No keypair exists for it. | `fund_milestone.rs` (`token::authority = vault`) |
| Payment disputes | Release paths are fixed instructions with explicit status and time preconditions. No discretionary release exists. | `approve_milestone.rs`, `partial_timeout_release.rs`, `full_timeout_release.rs` |
| Unverifiable escrow | `EscrowVault` records `total_locked` / `total_released`; the vault address is derivable from the gig by anyone. | `state/vault.rs`, `VAULT_SEED` derivation |
| Client goes silent | 72h → 20% released permissionlessly; 7d → remainder released permissionlessly. Neither requires the client. | `PARTIAL_TIMEOUT`, `FULL_TIMEOUT` in `constants.rs` |
| Platform lock-in | `UserProfile` is a PDA seeded by the user's own authority, readable by any program or indexer. | `initialize_profile.rs` |
| Opaque reputation | `compute_reputation_score` is a pure function of stored counters, with published weights. | `reputation/src/utils.rs` |

**What is not solved yet.** There is no dispute resolution. If a client disputes delivered work in good faith, the protocol's only outcome today is the timeout schedule paying the freelancer. That is a deliberate MVP boundary, not an oversight — see [Roadmap](#roadmap).

---

## Design Principles

**Non-custodial by construction, not by policy.** Custody is removed by making the signing key non-existent rather than by promising not to use it. The vault's authority is a PDA; `invoke_signed` with the vault seeds is the only way to move funds, and only escrow's own code can produce it.

**One responsibility per program.** A program that only moves tokens between a vault and two known parties can be reasoned about exhaustively. Adding ratings, badges, and jury voting to the same program would put unrelated logic inside the custody trust boundary and force a re-audit of payment code for every scoring tweak. Hence: escrow holds no reputation state; reputation holds no funds.

**No shared trust between programs today.** Escrow does not CPI into reputation. A bug in scoring cannot reach locked funds. The cost of this choice is that reputation updates are currently authorized by a hardcoded signer (see [Security Model](#security-model)) rather than proven by a CPI — an acknowledged trade-off with a designed migration path.

**Determinism over convenience.** `reputation_score` and `average_rating` are recomputed from totals on every mutation instead of being incrementally blended or passed in. Recomputing from `rating_sum` / `rating_count` makes the result independent of submission order and free of accumulated drift, and lets any third party verify it.

**Structural invariants over runtime checks.** Where a rule can be encoded in a PDA seed, it is. One rating per job and one badge per (profile, type) are not enforced by an `if` statement that could be missed — they are enforced by `init` failing on an already-existing address.

**Arithmetic is always checked.** Every balance and counter operation routes through `checked_add` / `checked_sub` / `percent_of` helpers that return a program error instead of wrapping. `overflow-checks = true` is additionally set on the release profile in the workspace `Cargo.toml`.

**Auditable by outsiders.** Every state-changing instruction emits a typed Anchor event, so an indexer can reconstruct full protocol history from logs without polling accounts.

---

## Implementation Matrix

Status derived from repository contents only.

| Component | Path | Purpose | Status | Notes |
|---|---|---|---|---|
| Gig program | `programs/gig` | Job/gig metadata and lifecycle | **Complete, internally audited** | 7 instructions (4 client + 3 CPI-only), 1 account type, 8 events, 15 error variants, 68 tests |
| Escrow program | `programs/escrow` | Milestone escrow, funding, and release | **Complete, internally audited** | 9 instructions, 2 account types, 7 events, 11 error variants, 148 tests |
| Reputation program | `programs/reputation` | Profiles, ratings, badges, deterministic scoring | **Complete, internally audited** | 5 instructions, 3 account types, 5 events (4 emitted), 11 error variants, 36 tests |
| Achievement program | `programs/achievement` | User-claimed NFT badges via Metaplex Core | **Implemented** | 2 instructions, 2 account types, 1 event, 4 error variants, 8 tests (see [Achievement Program](#achievement-program)) |
| Dispute program | `programs/dispute` | Third-party resolution | **Not implemented** | Empty directory (`src/.gitkeep`), not a workspace member |
| Escrow → Gig CPI | Cross-program status sync | `Assigned→InProgress`, `→Completed`, `→Cancelled` | **Implemented** | Signed by an `escrow_authority` PDA Gig verifies via `seeds::program`; see [ARCHITECTURE.md §5.3–5.4](./ARCHITECTURE.md) |
| Escrow → Reputation CPI | `settle_reputation`, `rate_freelancer` | Trustless completion/rating recording after settlement | **Implemented** | `update_completion`/`submit_rating` require the same `escrow_authority` PDA pattern as the Gig CPI; see [ARCHITECTURE.md §18](./ARCHITECTURE.md#18-escrow--reputation-cpi) |
| Frontend | `apps/web` | Next.js client | **Scaffold** | Default Next.js landing page + a devnet wallet-adapter provider. No program integration, no gig/escrow/reputation UI |
| API routes | `apps/web/src/app/api` | — | **Empty** | `.gitkeep` only |
| Reputation indexer | `services/reputation-indexer` | Off-chain event indexing + public read API | **Empty scaffold** | `.gitkeep` directories only, no source |
| Shared packages | `packages/{idl,shared-types,config}` | Generated IDLs, TS types, config | **Empty** | `.gitkeep` only |
| Infrastructure | `infrastructure/{scripts,supabase}` | Deploy/ops | **Empty** | `.gitkeep` only |
| CI | — | — | **None** | No `.github/workflows` in the repository |
| Deployment | `Anchor.toml` | — | **Localnet only** | Program IDs pinned under `[programs.localnet]`; cluster is `localnet` |
| Documentation | `*.md`, `docs/` | Architecture, security, testing | **Complete for both programs** | See [Documentation Index](#documentation-index) |
| External security audit | — | — | **Not performed** | Audits to date are internal, documented in `SECURITY.md` |
| NFT rewards | `programs/achievement` | Metaplex Core NFTs for earned badges | **Implemented** | See [Achievement Program](#achievement-program) |
| Governance | — | — | **Not implemented** | Described only as product vision in `docs/details.md` |

---

## System Architecture

Four programs. Solid arrows are implemented; dashed arrows are planned and absent from the code today. `Gig` and `Escrow` used to be one merged program; they were split so gig metadata and fund custody sit in separately auditable trust boundaries, connected only by the one CPI edge escrow needs to advance gig status.

```
                 ┌───────────────────────────────┐
                 │  Client  /  Freelancer         │
                 │  (wallet signers)              │
                 └───────┬───────────────┬────────┘
                         │               │
             signs gig + │               │ signs delivery
             funding ixs │               │ submission
                         ▼               ▼
        ┌───────────────────────────────────────────┐        ┌──────────────────────────────────────────────────┐
        │              GIG PROGRAM                    │        │               ESCROW PROGRAM                       │
        │  9LpGZY8p8dYfYdWm5D9MuvGXh9VXdF8DqEEAmdNZ92Na│◄───────│  FFJ8YAVGUJP4SeDZrQ3g1d9fdQFq9hutsU1m4f3o1UXS      │
        │                                              │  CPI:  │                                                     │
        │   Gig PDA (id, client, freelancer, status,   │ mark_in_progress /       Milestone PDA (per index)          │
        │   title, description, skills, category,      │ mark_completed_by_escrow /  EscrowVault PDA (milestone_count,│
        │   budget, deadline)                           │ mark_cancelled_by_escrow    active_milestone)               │
        │                                              │  signed by escrow_authority PDA                             │
        └──────────────────────────────────────────────┘        └──────────────────────┬──────────────────────────────┘
                                                                                        │ transfer_checked
                                                                                        │ (CPI, PDA-signed on release)
                                                                                        ▼
                                                                          ┌───────────────────────────────┐
                                                                          │  SPL Token Program             │
                                                                          │  Vault Token Account           │
                                                                          │  authority = EscrowVault PDA   │
                                                                          └───────────────┬───────────────┘
                                                                                          │ release
                                                                                          ▼
                                                                          ┌───────────────────────────────┐
                                                                          │  Freelancer token account      │
                                                                          └───────────────────────────────┘

        ┌────────────────────────────────────────────────┐
        │          REPUTATION PROGRAM                     │
        │  mXn62yZ4KFvPsdtMmEdGkB71jXcr17SQJHXftgPVGNB   │
        │                                                 │
        │   UserProfile PDA ──┬──< Badge PDA (per type)   │
        │                     └──< Rating PDA (per job)   │
        └───────▲───────────────────────────┬─────────────┘
                │ CPI: update_completion /  │ emits events
                │ submit_rating              ▼
                │ signed by escrow_authority ┌────────────────────┐
                │ PDA (same pattern as       │  Indexer (planned) │
                │ Escrow → Gig)              └────────────────────┘
        ┌───────┴───────────────────────────┐
        │  settle_reputation (permissionless)│         ┌────────────────────┐
        │  rate_freelancer (client-signed)   │┄┄┄┄┄┄┄┄▶│ Dispute (planned)  │
        │  in programs/escrow                │         └────────────────────┘
        └─────────────────────────────────────┘
```

**Interactions that exist today**

1. **Client → Gig.** The client signs `initialize_gig`, `update_gig`, `publish_gig`, `assign_freelancer`, `complete_gig`, `archive_gig`, and `cancel_gig`. Every one of these validates `has_one = client` against the `Gig` account.
2. **Client → Escrow.** The client signs `create_milestone`, `fund_milestone`, `approve_milestone`, `cancel_before_funding`, and `rate_freelancer`. Each validates `has_one = client` against the `Gig` account it reads (owned by the gig program, but readable and reference-checkable by escrow like any other account).
3. **Escrow → Gig (CPI).** `fund_milestone` calls `mark_in_progress` on the gig's first funding; `approve_milestone` and `full_timeout_release` call `mark_completed_by_escrow` when the last milestone clears; `cancel_before_funding` calls `mark_cancelled_by_escrow`. All three are signed by escrow's `escrow_authority` PDA, which the gig program accepts only via a `seeds::program = ESCROW_PROGRAM_ID` constraint — no other caller can produce that signature.
4. **Escrow → Reputation (CPI).** `settle_reputation` (permissionless, callable once a gig's vault is fully released, at most once per gig via `vault.reputation_synced`) calls `update_completion`; `rate_freelancer` (client-signed, only for a `Completed` gig) calls `submit_rating`. Both are signed by the same `escrow_authority` PDA as #3, verified by Reputation via the identical `seeds::program = ESCROW_PROGRAM_ID` constraint.
5. **Escrow → SPL Token.** Inbound funding is signed by the client; every outbound release is signed by the `EscrowVault` PDA via `CpiContext::new_with_signer`.
6. **Freelancer → Escrow.** Only `submit_delivery`, gated by `has_one = freelancer` on the `Gig` account. The freelancer can never move funds directly; they can only start the clock.
7. **Anyone → Escrow (timeouts, settlement).** `partial_timeout_release`, `full_timeout_release`, and `settle_reputation` take no signer at all. Any account can crank them once their precondition is met, and none of them can move funds or reputation credit anywhere but the pinned destination, so a permissionless caller has nothing to gain by front-running.
8. **Anyone → Reputation.** `award_badge` is permissionless — eligibility is recomputed from the profile's own public fields on every call, so there is no privileged caller to gate.

**Interactions that do not exist**

- Gig never calls back into escrow, and Reputation never calls back into escrow — both CPI edges are one-directional (escrow → gig, escrow → reputation).
- Neither Gig nor Escrow reads or writes reputation *scoring* state beyond issuing the two CPIs above; Reputation never reads Gig or Escrow accounts.
- No dispute program, no NFT minting, no governance.

---

## Gig Program

**Path:** `programs/gig` · **Program ID:** `9LpGZY8p8dYfYdWm5D9MuvGXh9VXdF8DqEEAmdNZ92Na`

### Purpose and boundaries

Owns job/gig metadata and the gig lifecycle: draft → published → assigned → (in progress →) completed → archived, plus cancellation. It knows nothing about tokens, vaults, or milestones — it holds no funds and makes no outbound CPI of its own.

This program used to be merged into `escrow`: gig metadata and fund custody lived in one Anchor program because a milestone release needed to flip gig status in the same instruction. It was split out because that merge put unrelated logic — title edits, category taxonomy, the draft/publish workflow — inside the trust boundary that also moves money, so a metadata bug and a custody bug were one audit surface instead of two. Splitting means gig-lifecycle changes never touch the code path that custodies funds, and vice versa.

The one place the split shows: `status` has an `InProgress` value that no client instruction can set. Only escrow's `fund_milestone` can move a gig into `InProgress` (via CPI), because only escrow knows whether the first milestone has actually been funded.

### State

| Account | Fields | Size (`INIT_SPACE`) |
|---|---|---|
| `Gig` | `id, client, freelancer, mint, status, created_at, updated_at, title, description, skills, category, budget, deadline, bump` | 8 + 8 + 32×3 + 1 + 8×2 + (4+100) + (4+500) + (4+200) + (4+50) + 8 + 8 + 1 |

### Enum

```rust
GigStatus { Draft, Published, Assigned, InProgress, Completed, Cancelled, Archived }
```

`InProgress` is entered only via escrow's CPI (`mark_in_progress`) — it is not a client-callable transition. No client instruction can set or unset it.

### Constants (`constants.rs`)

| Constant | Value | Meaning |
|---|---|---|
| `MIN_DEADLINE_SECS` | 86 400 s | A gig deadline must be strictly more than 1 day out |
| `MAX_TITLE_LEN` / `MAX_DESCRIPTION_LEN` / `MAX_SKILLS_LEN` / `MAX_CATEGORY_LEN` | 100 / 500 / 200 / 50 | Input length caps matching account space |
| `ESCROW_AUTHORITY_SEED` | `b"escrow_authority"` | Seed the three CPI-only instructions require escrow's signer PDA to match |
| `ESCROW_PROGRAM_ID` | `FFJ8YAVGUJP4SeDZrQ3g1d9fdQFq9hutsU1m4f3o1UXS` | Hardcoded (not a crate dependency) so gig can verify the CPI caller without depending on the escrow crate. Same value duplicated in `reputation`; consistency with escrow's own `declare_id!` is enforced by a compile-time assertion in `programs/escrow/src/lib.rs` — see [SECURITY.md §4c](./SECURITY.md#4c-escrow-program-id-trust-assumption-operational) and [docs/runbooks/escrow-redeploy.md](./docs/runbooks/escrow-redeploy.md) |

### State machine

```
                  ┌──────── cancel_gig ────────┐
                  │                            ▼
 initialize_gig   │                       ┌─────────┐
      │           │                       │Cancelled│ (terminal)
      ▼           │                       └─────────┘
  ┌───────┐  publish_gig   ┌──────────┐  assign_freelancer   ┌──────────┐
  │ Draft │───────────────►│Published │─────────────────────►│ Assigned │
  └───────┘                └──────────┘                      └──────────┘
      ▲                                                       │        │
   update_gig                                    complete_gig │        │ fund_milestone
   (Draft only)                                  (manual, no  │        │ (escrow CPI:
                                                  escrow ever  │        │  mark_in_progress)
                                                  touched it)  │        ▼
                                                                │  ┌────────────┐
                                                                │  │ InProgress │
                                                                │  └────────────┘
                                                                │        │ final milestone
                                                                │        │ release (escrow CPI:
                                                                ▼        ▼ mark_completed_by_escrow)
                                                            ┌───────────┐  archive_gig  ┌──────────┐
                                                            │ Completed │──────────────►│ Archived │
                                                            └───────────┘               └──────────┘
```

`cancel_gig` is valid from `Draft`, `Published`, or `Assigned` only — once a gig is `InProgress`, only escrow can cancel it (`mark_cancelled_by_escrow`, called from `cancel_before_funding`), since escrow owns the locked-fund state at that point. `Completed`, `Cancelled`, and `Archived` are terminal.

### Events (8)

`GigCreated`, `GigUpdated`, `GigPublished`, `FreelancerAssigned`, `GigInProgress`, `GigCompleted`, `GigArchived`, `GigCancelled`.

### Errors (15)

`GigError` variants: `Unauthorized`, `TitleTooLong`, `DescriptionTooLong`, `SkillsTooLong`, `CategoryTooLong`, `InvalidDeadline`, `InvalidBudget`, `FreelancerAlreadyAssigned`, `NotDraftStatus`, `NotPublishedStatus`, `NotAssignedStatus`, `NotInProgressStatus`, `NotCompletedStatus`, `InvalidStatus`, `Overflow`.

### Security posture

- Every client instruction requires `Signer` and validates it against stored state with `has_one`.
- The three CPI-only instructions (`mark_in_progress`, `mark_completed_by_escrow`, `mark_cancelled_by_escrow`) accept `escrow_authority` only under `seeds = [ESCROW_AUTHORITY_SEED], seeds::program = ESCROW_PROGRAM_ID` — the Solana runtime itself rejects a signer PDA that wasn't derived and signed by the real escrow program, so no other program or client can forge the call.
- `initialize_gig` has no `skills` parameter — the field starts empty and can only be set through `update_gig`, which is `Draft`-only.

---

## Escrow Program

**Path:** `programs/escrow` · **Program ID:** `FFJ8YAVGUJP4SeDZrQ3g1d9fdQFq9hutsU1m4f3o1UXS`

### Purpose and boundaries

Owns milestone escrow only: create → fund → deliver → release. It reads and cross-checks the `Gig` account owned by the gig program (client/freelancer identity, mint, status), but never writes to it directly — it advances gig status by CPI, signed by its own `escrow_authority` PDA, so the gig program stays the sole owner of gig state even while escrow drives it forward.

The program knows nothing about ratings, badges, scores, or disputes. Its external dependencies are the SPL Token program and the gig program (for the `Gig` account type and the three CPI-only instructions).

### State

| Account | Fields | Size (`INIT_SPACE`) |
|---|---|---|
| `Milestone` | `gig, index, amount, released, status, submitted_at, approved_at, bump` | 8 + 32 + 4 + 8 + 8 + 1 + 8 + 8 + 1 = 78 |
| `EscrowVault` | `gig, token_account, mint, total_locked, total_released, milestone_count, active_milestone, bump` | 8 + 32×3 + 8 + 8 + 4 + 4 + 1 = 129 |

Plus one SPL `TokenAccount` per gig, owned by the token program, whose authority is the `EscrowVault` PDA. `milestone_count` and `active_milestone` moved here from `Gig` in the program split — they are payment-lifecycle bookkeeping, not gig metadata.

### Enum

```rust
MilestoneStatus { PendingFunding, Funded, Submitted, PartialReleased, Completed }
```

### Constants (`constants.rs`)

| Constant | Value | Meaning |
|---|---|---|
| `PARTIAL_TIMEOUT` | 72 × 3600 s | Delay after submission before 20% may be released |
| `FULL_TIMEOUT` | 7 × 86 400 s | Delay after submission before the remainder may be released |
| `PARTIAL_RELEASE_PERCENT` | 20 | Fraction released at the first timeout |
| `FULL_RELEASE_PERCENT` | 80 | Declared complement of the partial release |
| `ESCROW_AUTHORITY_SEED` | `b"escrow_authority"` | Seed for escrow's own CPI-signer PDA into the gig program |

Note: `full_timeout_release` releases `amount - released` rather than `percent_of(amount, FULL_RELEASE_PERCENT)`. The remainder is computed from actual state, so the constant is descriptive rather than load-bearing.

### State machine

**Milestone**

```
                    cancel_before_funding
                    (closes account, rent → client,
   ┌────────────────  CPIs mark_cancelled_by_escrow → gig.status = Cancelled)
   │
   ▼
┌────────────────┐ fund_milestone ┌────────┐ submit_delivery ┌───────────┐
│ PendingFunding │───────────────►│ Funded │────────────────►│ Submitted │
└────────────────┘                └────────┘                 └───────────┘
                                                                   │
                          approve_milestone (client)               │
                          ┌────────────────────────────────────────┤
                          │                                        │
                          ▼                            72h elapsed │
                    ┌───────────┐                                  ▼
                    │ Completed │◄── 7d elapsed ──┌─────────────────┐
                    └───────────┘  full_timeout   │ PartialReleased │
                                    _release      └─────────────────┘
```

Once `Funded`, no instruction returns the milestone's tokens to the client. Funds leave the vault only toward `gig.freelancer`.

### Events (7)

`MilestoneCreated`, `MilestoneFunded`, `DeliverySubmitted`, `MilestoneApproved`, `PartialReleaseExecuted`, `FullReleaseExecuted`, `MilestoneCancelledBeforeFunding`.

Every state-changing instruction emits exactly one. Gig-status side effects driven by CPI (`InProgress`, `Completed`, `Cancelled`) are additionally observable through the gig program's own events (`GigInProgress`, `GigCompleted`, `GigCancelled`) emitted from inside the CPI call.

### Errors (11)

`EscrowError` variants: `Unauthorized`, `InvalidStatus`, `AlreadyFunded`, `InsufficientFunds`, `InvalidMint`, `MilestoneAlreadySubmitted`, `TimeoutNotReached`, `Overflow`, `MathError`, `InvalidAmount`, `GigNotFundable`.

`GigNotFundable` is what `create_milestone` and `fund_milestone` raise when the gig's status (read cross-program from the gig account) is neither `Assigned` nor `InProgress`.

### Security posture

- Every privileged instruction requires `Signer` and validates it against stored state with `has_one`.
- Vault release paths re-derive the vault PDA (`seeds = [VAULT_SEED, gig.key()]`, `bump = vault.bump`) rather than trusting a passed address, and pin `vault_token_account` with `address = vault.token_account`.
- Destination token accounts are checked for both `mint` and `owner` against `gig.freelancer`.
- `fund_milestone` uses `init_if_needed` for the vault and its token account; a re-funding path re-checks `vault.mint` against the passed mint before accumulating.
- `create_milestone` and `fund_milestone` re-derive the `Gig` PDA themselves (`seeds = [gig::GIG_SEED, gig.id], seeds::program = gig::ID`) rather than trusting a passed `Gig` account — the cross-program read is just as guarded as a same-program one.
- CPI calls into the gig program are signed with `escrow_authority`'s stored bump, never a caller-supplied one.
- All balance math uses the checked helpers in `utils.rs`.

### Known design characteristics

- **Gig PDA seeds are `[b"gig", id]` — the client is not a seed** (enforced in the gig program, read here by escrow). Gig ids therefore share one global namespace: the first client to create id *N* owns it, and a second `initialize_gig` with the same id fails at `init`. Clients must allocate ids collision-aware (e.g. randomly). This differs from a per-client namespace and is worth knowing before integrating.
- **`complete_gig` can be called by the client from `Assigned` regardless of milestone state**, and `approve_milestone` / `full_timeout_release` also move a gig to `Completed` once the final milestone settles (via CPI). Both paths are intentional; the former lets a client close a gig that never entered escrow.

---

## Reputation Program

**Path:** `programs/reputation` · **Program ID:** `mXn62yZ4KFvPsdtMmEdGkB71jXcr17SQJHXftgPVGNB`

### Purpose and boundaries

Maintains a tamper-evident, independently recomputable record per user authority. It custodies no funds, performs no CPI, and has no dependency on the escrow program. Its only dependency is `anchor-lang`.

### State

| Account | Fields |
|---|---|
| `UserProfile` | `authority, completed_jobs, successful_jobs, cancelled_jobs, total_earnings, rating_sum, rating_count, average_rating, reputation_score, badges_earned, created_at, updated_at, bump` |
| `Rating` | `job_id, client, freelancer, score, review_hash[32], submitted_at, bump` |
| `Badge` | `profile, badge_type, issuer, issued_at, metadata (≤128 bytes), bump` |

`average_rating` is stored scaled by `RATING_SCALE = 100` (450 = 4.50 stars). Review text stays off-chain; only its hash is bound on-chain in `review_hash`.

### Badge types (7)

`FirstGig`, `TenCompletedJobs`, `HundredCompletedJobs`, `FiveStarPerformer`, `TrustedFreelancer`, `FastDeliverer`, `TopRated`.

Eligibility (`is_eligible_for_badge`):

| Badge | Rule |
|---|---|
| `FirstGig` | `completed_jobs >= 1` |
| `TenCompletedJobs` | `completed_jobs >= 10` |
| `HundredCompletedJobs` | `completed_jobs >= 100` |
| `FiveStarPerformer` | `rating_count >= 5 && average_rating >= 500` |
| `TopRated` | `rating_count >= 10 && average_rating >= 450` |
| `TrustedFreelancer`, `FastDeliverer` | Always `false` — these depend on signals the program does not track on-chain (delivery timing, external endorsement). `award_badge` is permissionless, so eligibility can never rest on an unverified caller claim; these two types simply have no path to eligibility yet. |

### Scoring algorithm

`compute_reputation_score` is pure, deterministic, and bounded to `[0, 1000]`:

```
success_rate   = successful_jobs * 100 / completed_jobs        (0 if no jobs)
rating_score   = min(average_rating / 5, 100)                  (average_rating is ×100-scaled)
volume_score   = min(completed_jobs, VOLUME_SCORE_CAP = 100)
earnings_score = min(total_earnings, EARNINGS_SCORE_CAP = 1e9) * 100 / EARNINGS_SCORE_CAP
penalty        = min(cancelled_jobs * 5, 200)

score = min( 3*success_rate + 3*rating_score + 2*volume_score + 2*earnings_score
             saturating_sub 2*penalty,
             MAX_REPUTATION_SCORE = 1000 )
```

Quality (success rate + rating) carries weight 6; scale (volume + earnings) carries weight 4. The cancellation penalty is capped so a long-lived profile cannot be driven arbitrarily negative, and `saturating_sub` guarantees no underflow. `EARNINGS_SCORE_CAP` is expressed in mint base units — with 6-decimal USDC, 1e9 base units is 1,000 USDC.

`average_rating` is recomputed as `rating_sum * 100 / rating_count` on every submission rather than blended incrementally, so it is exact and order-independent.

### Events (5, of which 4 are emitted)

`ProfileCreated`, `RatingSubmitted`, `CompletionUpdated`, `BadgeAwarded` are emitted. `ProfileUpdated` is defined and reserved but no instruction emits it.

### Errors (11)

`ProfileAlreadyExists`, `ProfileNotFound`, `InvalidRating`, `DuplicateRating`, `BadgeAlreadyOwned`, `BadgeNotEligible`, `Unauthorized`, `MathOverflow`, `SelfDealing`, `InvalidEarnings`, `MetadataTooLong`.

Several are reserved rather than raised, because the corresponding rule is enforced structurally instead: `ProfileAlreadyExists` and `DuplicateRating` and `BadgeAlreadyOwned` are all enforced by `init` failing on an existing PDA, which surfaces as an Anchor account-already-initialized error rather than a custom code.

### State model

`UserProfile` has no status enum. It is a set of counters that only move forward (`completed_jobs`, `successful_jobs`, `cancelled_jobs`, `total_earnings`, `rating_sum`, `rating_count`), with `average_rating` and `reputation_score` recomputed from them on every mutation. `Rating` accounts are write-once and never mutated. `Badge` accounts are write-once.

### Escrow CPI

`update_completion` and `submit_rating` require an `escrow_authority: Signer<'info>` constrained to `seeds = [ESCROW_AUTHORITY_SEED], bump, seeds::program = ESCROW_PROGRAM_ID` — the same pattern Gig uses to trust Escrow (§5.4 above). A PDA has no private key, so the only way to satisfy this is `invoke_signed` from Escrow's own executing code; see [ARCHITECTURE.md §18](./ARCHITECTURE.md#18-escrow--reputation-cpi). `get_profile` exists to expose `reputation_score` through CPI return data for on-chain consumers.

---

## Achievement Program

**Path:** `programs/achievement` · **Program ID:** `GV8Z39NBK7qrojXCfnnwLTXpqsLoCW6sy9cLHGYjtrv9`

### Purpose and boundaries

Achievement owns exactly one thing: **minting an NFT for a badge the user has already earned.** It does not compute eligibility, store reputation counters, or run during escrow settlement — settlement stays exactly as lightweight as it was before this program existed. It reuses `reputation::BadgeType` rather than defining a parallel badge enum, so badge criteria live in exactly one place.

```
Escrow settles → Reputation records completion → award_badge() unlocks a Badge PDA
                                                            │
                                                            ▼
                                        User calls achievement::claim_achievement()
                                                            │
                                                            ▼
                                    Achievement re-derives the Badge PDA (proof of
                                    eligibility) → Metaplex Core CPI mints the NFT
```

NFT minting is always a separate, user-initiated transaction — never a side effect of `approve_milestone`, `full_timeout_release`, or any other escrow instruction. This keeps the money path free of Metaplex's account/CPI surface and lets a user claim on their own schedule (or never).

### State

- `AchievementConfig` (singleton, seeds `[b"config"]`) — `admin`, `collection` (the shared Metaplex Core collection every achievement mints into), `bump`.
- `Achievement` (seeds `[b"achievement", owner, badge_type]`) — `owner`, `badge_type`, `mint` (the minted asset's pubkey), `claimed`, `claimed_at`, `bump`. One PDA per `(owner, badge_type)`; `init` failing on a repeat call is the entire duplicate-claim/replay guard.

### Instructions (2)

- **`init_collection(name, uri)`** — one-time admin setup. Creates the shared Metaplex Core collection with `config` (a program PDA) as its update authority, so only a `claim_achievement` CPI signed by that same PDA can ever mint into it.
- **`claim_achievement(badge_type)`** — re-derives the caller's `UserProfile` and `Badge` PDAs *from the reputation program* (`seeds::program = reputation::ID`). The `Badge` PDA existing at all is the eligibility proof — `award_badge` already checked the criteria before creating it, so Achievement never recomputes that logic. Mints a Metaplex Core asset owned by the claimer via CPI, signed by the `config` PDA as collection authority, then records `Achievement.mint` and emits `AchievementClaimed`.

### Security posture

- **Forged profile / forged badge** — both are re-derived by seeds under `seeds::program = reputation::ID`; a substituted account fails Anchor's seeds/owner check before the handler runs.
- **Ineligible claim** — no `Badge` PDA exists for that `(owner, badge_type)`, so the account resolves to nothing and the transaction fails before any CPI.
- **Duplicate claim / replay** — the `Achievement` PDA's `init` constraint rejects a second claim outright; there is no explicit "already claimed" check to bypass.
- **Account substitution** — every account (`profile`, `badge`, `config`, `achievement`, `mpl_core_program`) is either a re-derived PDA or checked against a hardcoded/config-stored address.
- **NFT provenance** — the shared collection's update authority is the `config` PDA, not a keypair; only this program's own `invoke_signed` CPI can mint into it, so an NFT with that collection can only ever originate from a legitimate `claim_achievement` call.

See [SECURITY.md](./SECURITY.md#achievement-program-security-model) for the full threat-model writeup and [TESTING.md](./TESTING.md#achievement-program) for per-test coverage.

---

## Account Model

| Account | Program | Owner (writer) | Created by | Mutated by | Closed by | Cardinality |
|---|---|---|---|---|---|---|
| `Gig` | gig | gig | `initialize_gig` | `update_gig`, `publish_gig`, `assign_freelancer`, `complete_gig`, `archive_gig`, `cancel_gig` (client); `mark_in_progress`, `mark_completed_by_escrow`, `mark_cancelled_by_escrow` (escrow, via CPI) | never | 1 per gig id (global) |
| `Milestone` | escrow | escrow | `create_milestone` | `fund_milestone`, `submit_delivery`, `approve_milestone`, both timeout releases | `cancel_before_funding` (rent → client) | N per gig, indexed 0..`milestone_count` |
| `EscrowVault` | escrow | escrow | `create_milestone` (`init_if_needed`, on the gig's first milestone) | every milestone creation and funding/release | never | 1 per gig |
| Vault token account | SPL Token | authority = `EscrowVault` PDA | `fund_milestone` (`init_if_needed`) | SPL transfers only | never | 1 per gig |
| `UserProfile` | reputation | reputation | `initialize_profile` | `submit_rating`, `update_completion`, `award_badge` | never | 1 per authority |
| `Rating` | reputation | reputation | `submit_rating` | never (immutable) | never | 1 per `job_id` (global) |
| `Badge` | reputation | reputation | `award_badge` | never (immutable) | never | ≤1 per (authority, badge_type); ≤7 per profile |

**Relationships.** `Milestone.gig` and `EscrowVault.gig` both back-reference the `Gig`, and every instruction that touches a milestone or vault re-checks that back-reference in addition to the PDA derivation. On the reputation side, `Rating` records `client` and `freelancer` but is keyed only by `job_id`; the freelancer's profile is located by seed derivation from the passed `freelancer` account.

**Lifecycle constraints.**
- A gig's `mint` is fixed at `initialize_gig` and validated (`has_one = mint`) at funding time; a gig cannot mix tokens.
- A milestone's `amount` is fixed at creation. `released` only increases and never exceeds `amount` (`checked_sub(amount, released)` bounds every release).
- `EscrowVault.total_released <= total_locked` follows from every release being bounded by the milestone remainder.
- `Rating` accounts are the only write-once accounts in the protocol; their immutability is the basis of the reputation record's tamper-evidence.

---

## PDA Architecture

Every stateful account in both programs is a Program Derived Address: an address off the ed25519 curve, derived from seed bytes plus the program id, for which no private key can exist. Three properties follow, and the protocol depends on all three.

**1. Deterministic addressing.** Anyone — a client, an indexer, an auditor — can compute the canonical address for a role from public inputs. There is no admin-settable registry mapping "gig 7" to an account, so there is nothing to compromise or rewrite. Verification is `find_program_address`, not trust.

**2. Program-only signing.** A PDA has no keypair. The only way to authorize it is `invoke_signed` with the exact seeds, which only the deriving program can produce. This is what makes the vault non-custodial: `token::authority = vault` means the SPL Token program will move vault funds only when the escrow program itself signs with `[VAULT_SEED, gig, bump]`.

**3. Structural uniqueness.** Because the address is a function of the seeds, "one X per Y" becomes an address collision rather than a runtime check. Anchor's `init` fails on an already-initialized account, so duplicate prevention cannot be forgotten in a handler.

### Seed table

| PDA | Program | Seeds | Bump storage | What the seed structurally guarantees |
|---|---|---|---|---|
| `Gig` | gig | `[b"gig", id.to_le_bytes()]` | `gig.bump` | Exactly one gig per id, globally. Note this is a **global** id namespace, not per-client |
| `Milestone` | escrow | `[b"milestone", gig.key(), vault.milestone_count.to_le_bytes()]` | `milestone.bump` | Sequential, gap-free, collision-free indexing; a milestone index can never be created twice |
| `EscrowVault` | escrow | `[b"vault", gig.key()]` | `vault.bump` | Exactly one vault per gig; also the signer seeds for every release; carries `milestone_count`/`active_milestone` (moved off `Gig` in the program split) |
| Vault token account | escrow | `[b"vault", gig.key(), b"token"]` | derived at init | Exactly one token account per gig vault, distinct from the vault state account |
| `escrow_authority` | escrow | `[b"escrow_authority"]` | derived at use, no stored bump | Not a data account — used only as escrow's CPI-signer identity when calling into the gig program. The gig program accepts it solely under `seeds::program = ESCROW_PROGRAM_ID` |
| `UserProfile` | reputation | `[b"profile", authority.key()]` | `profile.bump` | Exactly one profile per authority; identity is the wallet itself, not an assigned id |
| `Rating` | reputation | `[b"rating", job_id.to_le_bytes()]` | `rating.bump` | One rating per job, permanently — a second `submit_rating` for the same job fails at `init` |
| `Badge` | reputation | `[b"badge", profile.authority, badge_type as u8]` | `badge.bump` | One badge per type per profile |

### Validation and anti-spoofing

Passing a fake account is the canonical Solana attack. The defenses used here:

- **Re-derivation on use, not just on creation.** Release instructions declare `seeds = [VAULT_SEED, gig.key().as_ref()]` with `bump = vault.bump` in the account constraint, so Anchor recomputes the address and rejects a substituted vault before the handler runs.
- **Canonical bump pinning.** Bumps are stored at init and re-supplied on later use, so a caller cannot present a non-canonical bump to derive a second valid address for the same logical account.
- **Type + owner checking.** `Account<'info, T>` enforces both the owning program and the account discriminator, so a `Milestone` cannot be passed where a `Gig` is expected, and an account owned by another program is rejected.
- **Cross-reference checking on top of derivation.** `milestone.gig == gig.key()` is asserted even though the milestone PDA already derives from the gig — defense in depth against a derivation mistake.
- **Address pinning for non-PDA accounts.** `vault_token_account` is pinned with `address = vault.token_account`, and the release destination's `owner` is checked against `gig.freelancer`.
- **`init` as the replay guard.** Reinitialization of `Gig`, `Milestone`, `Rating`, and `Badge` is impossible: the address is already occupied.

---

## Instruction Reference

### Gig — 7 instructions (4 client + 3 CPI-only)

| Instruction | Authority | Precondition | Key accounts | State change | Event | Notable errors |
|---|---|---|---|---|---|---|
| `initialize_gig(id, title, description, category, budget, deadline)` | `client` (signer, payer) | — | `client`, `mint`, `gig` (init), `system_program` | Creates `Gig` in `Draft`; `skills` empty; `freelancer = default` | `GigCreated` | `TitleTooLong`, `DescriptionTooLong`, `CategoryTooLong`, `InvalidBudget`, `InvalidDeadline` |
| `update_gig(title, description, skills, category, budget, deadline)` | `client` (`has_one`) | `status == Draft` | `client`, `gig` | Overwrites metadata, bumps `updated_at` | `GigUpdated` | `Unauthorized`, `NotDraftStatus`, length/budget/deadline errors |
| `publish_gig()` | `client` | `status == Draft` | `client`, `gig` | `Draft → Published` | `GigPublished` | `Unauthorized`, `NotDraftStatus` |
| `assign_freelancer()` | `client` | `status == Published`, `gig.freelancer` unset, freelancer ≠ client | `client`, `freelancer` (unchecked), `gig` | Sets `freelancer`; `Published → Assigned` | `FreelancerAssigned` | `NotPublishedStatus`, `FreelancerAlreadyAssigned`, `Unauthorized` |
| `complete_gig()` | `client` | `status == Assigned` | `client`, `gig` | `Assigned → Completed` (manual path, for gigs with no milestone activity) | `GigCompleted` | `NotAssignedStatus`, `Unauthorized` |
| `archive_gig()` | `client` | `status == Completed` | `client`, `gig` | `Completed → Archived` | `GigArchived` | `NotCompletedStatus`, `Unauthorized` |
| `cancel_gig()` | `client` | `status ∈ {Draft, Published, Assigned}` | `client`, `gig` | `→ Cancelled` | `GigCancelled` | `InvalidStatus`, `Unauthorized` |
| `mark_in_progress()` | **`escrow_authority` PDA only** (CPI, `seeds::program = ESCROW_PROGRAM_ID`) | `status == Assigned` | `escrow_authority`, `gig` | `Assigned → InProgress` | `GigInProgress` | `NotAssignedStatus` |
| `mark_completed_by_escrow()` | **`escrow_authority` PDA only** (CPI) | `status == InProgress` | `escrow_authority`, `gig` | `InProgress → Completed` | `GigCompleted` | `NotInProgressStatus` |
| `mark_cancelled_by_escrow()` | **`escrow_authority` PDA only** (CPI) | `status ∈ {Assigned, InProgress}` | `escrow_authority`, `gig` | `→ Cancelled` | `GigCancelled` | `InvalidStatus` |

The last three are not client-callable: a direct (non-CPI) call fails because the `escrow_authority` account can't satisfy `seeds::program = ESCROW_PROGRAM_ID` unless it was actually derived and signed by the deployed escrow program.

### Escrow — 7 instructions

| Instruction | Authority | Precondition | Key accounts | State change | Event | Notable errors |
|---|---|---|---|---|---|---|
| `create_milestone(amount)` | `client` | `gig.status ∈ {Assigned, InProgress}`, `amount > 0` | `client`, `gig` (cross-program, re-derived), `vault` (init_if_needed), `milestone` (init), `system_program` | Creates `Milestone` at index `vault.milestone_count` in `PendingFunding`; increments `vault.milestone_count` | `MilestoneCreated` | `InvalidAmount`, `GigNotFundable`, `Unauthorized` |
| `fund_milestone()` | `client` | `milestone.status == PendingFunding`, `gig.mint` matches | `client`, `gig`, `milestone`, `vault`, `vault_token_account` (init_if_needed), `client_token_account`, `mint`, `escrow_authority`, `gig_program`, token + system programs | `transfer_checked` client → vault; `vault.total_locked += amount`; `PendingFunding → Funded`; **CPI** `mark_in_progress` on the gig's first funding | `MilestoneFunded` | `AlreadyFunded`, `InvalidMint`, `Unauthorized`, `Overflow` |
| `submit_delivery()` | `freelancer` (`has_one` on `gig`) | `milestone.status == Funded` | `freelancer`, `gig`, `milestone` | Sets `submitted_at = now`; `Funded → Submitted` (starts both timeout clocks) | `DeliverySubmitted` | `MilestoneAlreadySubmitted`, `InvalidStatus`, `Unauthorized` |
| `approve_milestone()` | `client` | `milestone.status == Submitted`, remainder > 0 | `client`, `gig`, `milestone`, `vault`, `vault_token_account`, `freelancer`, `freelancer_token_account`, `mint`, `escrow_authority`, `gig_program`, `token_program` | PDA-signed `transfer_checked` of `amount - released`; `Milestone → Completed`; advances `vault.active_milestone`; **CPI** `mark_completed_by_escrow` if this was the last milestone | `MilestoneApproved` | `InvalidStatus`, `InsufficientFunds`, `InvalidMint`, `Unauthorized` |
| `partial_timeout_release()` | **none — permissionless** | `milestone.status == Submitted`, `now >= submitted_at + 72h` | `gig`, `milestone`, `vault`, `vault_token_account`, `freelancer_token_account`, `mint`, `token_program` | PDA-signed release of 20% of `amount`; `Submitted → PartialReleased` | `PartialReleaseExecuted` | `TimeoutNotReached`, `InvalidStatus`, `InsufficientFunds` |
| `full_timeout_release()` | **none — permissionless** | `milestone.status == PartialReleased`, `now >= submitted_at + 7d` | same as `partial_timeout_release`, plus `escrow_authority`, `gig_program` | PDA-signed release of `amount - released`; `Milestone → Completed`; advances `vault.active_milestone`; **CPI** `mark_completed_by_escrow` if last milestone | `FullReleaseExecuted` | `TimeoutNotReached`, `InvalidStatus`, `InsufficientFunds` |
| `cancel_before_funding()` | `client` | `milestone.status == PendingFunding` | `client`, `gig`, `milestone` (`close = client`), `escrow_authority`, `gig_program` | Closes the milestone, rent → client; **CPI** `mark_cancelled_by_escrow` | `MilestoneCancelledBeforeFunding` | `AlreadyFunded`, `Unauthorized` |

The full-timeout path requires the partial release to have happened first: `full_timeout_release` demands `PartialReleased`, so a milestone that sat untouched for 7 days needs one `partial_timeout_release` crank before the remainder can be released. Both cranks are permissionless, so anyone can perform them.

### Reputation — 5 instructions

| Instruction | Authority | Precondition | Key accounts | State change | Event | Notable errors |
|---|---|---|---|---|---|---|
| `initialize_profile()` | `authority` (signer, payer) | Profile does not exist | `authority`, `profile` (init), `system_program` | Creates a zeroed `UserProfile` | `ProfileCreated` | account-already-initialized on retry |
| `submit_rating(job_id, score, review_hash)` | `client` (signer) + `escrow_authority` (CPI-only PDA signer) | `1 <= score <= 5`, `client != freelancer`, no rating for `job_id` | `client`, `escrow_authority`, `freelancer` (unchecked), `freelancer_profile`, `rating` (init), `system_program` | Writes immutable `Rating`; folds score into `rating_sum`/`rating_count`; recomputes `average_rating` and `reputation_score` | `RatingSubmitted` | `InvalidRating`, `SelfDealing`, init failure on duplicate job, seeds-constraint failure if `escrow_authority` isn't Escrow's own PDA |
| `update_completion(successful, earnings)` | `escrow_authority` (CPI-only PDA signer) | — | `escrow_authority`, `profile` | `completed_jobs += 1`; `successful_jobs += 1` and `total_earnings += earnings`, **or** `cancelled_jobs += 1`; recomputes score | `CompletionUpdated` | `MathOverflow`, seeds-constraint failure if not a real Escrow CPI |
| `award_badge(badge_type, metadata)` | none — permissionless (`payer` only funds rent) | Eligibility rule met, `metadata.len() <= 128`, badge not already held | `payer`, `profile`, `badge` (init), `system_program` | Creates immutable `Badge`; `badges_earned += 1` | `BadgeAwarded` | `BadgeNotEligible`, `MetadataTooLong`, init failure on duplicate |
| `get_profile()` | none (read-only) | — | `profile` | none | none | — |

`get_profile` returns `reputation_score` as CPI return data. Off-chain clients should deserialize the account directly; this instruction exists for on-chain composability. `update_completion` and `submit_rating` can only ever be invoked via CPI from Escrow's `settle_reputation`/`rate_freelancer` — see the next section.

---

## Complete Protocol Flow

```
   CLIENT              GIG PROGRAM         ESCROW PROGRAM              FREELANCER
     │
     │ initialize_gig(id, …)
     ├───────────────► Gig PDA created  [Draft]
     │                 emit GigCreated
     │ update_gig(…)   (Draft only)
     ├───────────────► metadata updated
     │ publish_gig()
     ├───────────────► [Published]
     │ assign_freelancer(F)
     ├───────────────► [Assigned], gig.freelancer = F
     │ create_milestone(amount)                          │
     ├──────────────────────────────────────────────────►│ Milestone#i PDA  [PendingFunding]
     │ fund_milestone()                                  │
     ├──────────────────────── transfer_checked ────────►│ Vault Token Account (authority = Vault PDA)
     │                                                    │ total_locked += amount
     │                 ◄──── CPI: mark_in_progress ───────┤ Milestone#i      [Funded]
     │                 [InProgress]  (first funding only) │
     │                                                    │                       submit_delivery()
     │                                                    │ Milestone#i ◄──────────────┤
     │                                                    │ submitted_at = now [Submitted]
     │                                                    │ ── timeout clocks start ──
     │
     ├── PATH A: approve_milestone()                      │
     │        vault PDA signs transfer_checked of (amount − released)
     │        Milestone [Completed]; last milestone → CPI: mark_completed_by_escrow
     │
     ├── PATH B: client silent ≥72h → partial_timeout_release()  (anyone)
     │        20% released to freelancer; Milestone [PartialReleased]
     │        ↓ still silent ≥7d → full_timeout_release()        (anyone)
     │        remainder released; Milestone [Completed]; last milestone → CPI: mark_completed_by_escrow
     │
     └── PATH C: never funded → cancel_before_funding()
              milestone account closed, rent → client, CPI: mark_cancelled_by_escrow → Gig [Cancelled]

     After the last milestone completes → Gig [Completed] (via CPI) → archive_gig() → [Archived]

     Once vault.active_milestone >= vault.milestone_count → settle_reputation()  (anyone, once only)
              CPI: update_completion(true, vault.total_released) → freelancer profile updated
     After Gig [Completed] → client may call rate_freelancer(score, review_hash)
              CPI: submit_rating(gig.id, score, review_hash) → Rating PDA created, profile updated

   ─────────────── same escrow program, CPI into reputation program ───────────────

   ESCROW PROGRAM                              REPUTATION PROGRAM
     │ settle_reputation()  (permissionless, once vault fully released)
     ├── CPI: update_completion(true, earnings), signed by escrow_authority PDA ─►│ counters + score updated
     │        vault.reputation_synced = true                                     │ emit CompletionUpdated
     │
     │ rate_freelancer(score, review_hash)  (client-signed, gig must be Completed)
     ├── CPI: submit_rating(gig.id, score, review_hash), signed by escrow_authority PDA ─►│ Rating PDA (immutable)
     │                                                                            │ emit RatingSubmitted

   ANYONE                                     REPUTATION PROGRAM
     │ award_badge(type, metadata)  (permissionless; eligibility re-verified on-chain)
     └──────────────────────► Badge PDA (immutable), emit BadgeAwarded

   ┌ ─ ─ PLANNED, NOT IMPLEMENTED ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐
     dispute program; NFT badge minting; indexer read API           │
   └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘
```

The critical property of Path B: it requires no cooperation from the client **and** no cooperation from the freelancer's own key. Any account can crank the release, and the tokens can only land in `gig.freelancer`'s token account, so a permissionless crank confers no advantage on the caller.

---

## Security Model

Full writeup: [SECURITY.md](./SECURITY.md). Summary of what the code enforces.

### Threat model

| Adversary | Goal | Defense |
|---|---|---|
| Malicious client | Reclaim funded escrow after delivery | No refund path exists post-funding. `cancel_before_funding` requires `PendingFunding` |
| Malicious client | Stall payment indefinitely | Permissionless timeout releases at 72h and 7d |
| Malicious freelancer | Get paid without delivering | Release requires either client approval or an elapsed timeout that only starts at `submit_delivery` |
| Malicious freelancer | Submit delivery on someone else's gig | `has_one = freelancer` on the `Gig` |
| Any third party | Drain a vault by passing a fake vault or token account | Vault PDA re-derived with stored bump; `vault_token_account` pinned by `address`; SPL authority is the PDA |
| Any third party | Redirect a release to their own token account | `freelancer_token_account.owner == gig.freelancer` constraint |
| Any third party | Front-run a timeout crank for profit | Cranks are permissionless but the destination is fixed; there is nothing to extract |
| Any third party | Reinitialize an existing account | `init` on an occupied PDA fails |
| Rating spammer | Submit multiple ratings for one job | `Rating` PDA seeded by `job_id` alone |
| Self-dealer | Rate themselves | `require_keys_neq!(client, freelancer)` |
| Badge farmer | Collect the same badge twice | `Badge` PDA seeded by `(authority, badge_type)` |
| Overflow attacker | Wrap a counter or balance | Checked helpers everywhere + `overflow-checks = true` |

### Trust assumptions (explicit)

1. **`REPUTATION_AUTHORITY` is trusted.** A single hardcoded pubkey (`vo18wuiY77EZa16yYKRdAjp2mj3g6GCvMHH8wkn6LAz`) can call `update_completion` with arbitrary `successful`/`earnings` values and can award `TrustedFreelancer` / `FastDeliverer` badges without any on-chain proof. This is the protocol's largest current centralization. It exists because escrow does not yet CPI into reputation; the account layouts are designed so swapping this check for a CPI-caller check requires no schema change. See ARCHITECTURE.md §18 and SECURITY.md §15.4.
2. **`submit_rating` accepts caller-supplied job identity.** `job_id` is not verified against any escrow account, so a rating's link to real completed work is asserted, not proven. The immutability and one-per-job guarantees still hold.
3. **No dispute mechanism.** A client with a genuine quality complaint has no on-chain remedy; the timeout schedule pays the freelancer regardless.
4. **Programs are upgradeable by their deploy authority** until that authority is revoked. Nothing is deployed yet, so no upgrade policy is in force.

### Enforced controls

- **Signer validation** on every privileged instruction, matched against stored state (`has_one`, `address =`).
- **Ownership and type validation** via `Account<'info, T>` (owner + discriminator).
- **PDA re-derivation** with canonical stored bumps on every use, not just creation.
- **Replay/reinit protection** via `init` on deterministic addresses.
- **State-transition validation** before every mutation, expressed as account constraints so the check cannot be skipped by a handler bug.
- **Checked arithmetic** through `utils.rs` helpers; `saturating_sub` where a floor of zero is the intended semantic; `u128` intermediate in `percent_of` so `percent_of(u64::MAX, 20)` cannot overflow.
- **Input validation** on all strings, amounts, budgets, and deadlines at the trust boundary.
- **SPL Token integration via `transfer_checked` only** — mint and decimals are validated by the token program on every transfer; the legacy `transfer` instruction is never used.
- **Single outbound CPI surface.** Escrow calls only the SPL Token program; reputation calls nothing. There is no arbitrary-program-invocation path.

### Protocol invariants

```
milestone.released <= milestone.amount
vault.total_released <= vault.total_locked
vault.active_milestone <= vault.milestone_count
0 <= profile.reputation_score <= 1000
profile.successful_jobs + profile.cancelled_jobs == profile.completed_jobs
profile.average_rating == rating_sum * 100 / rating_count   (0 when count == 0)
Ratings and Badges are never mutated after creation
Funded milestone tokens are only ever payable to gig.freelancer
```

---

## Testing

**Philosophy.** Tests run against compiled program logic in [litesvm](https://github.com/LiteSVM/litesvm) — a lightweight in-process SVM — rather than against a local validator. This makes the suite fast enough to run on every change while still exercising real instruction dispatch, real account constraints, real cross-program CPI (both `gig.so` and `escrow.so` are deployed into the same litesvm instance), real SPL Token CPIs, and real clock manipulation for timeout paths. Tests are organized by *property under test* rather than by instruction: a change that weakens authorization fails `authorization.rs` regardless of which instruction it touched.

**Totals (counted from `#[test]` attributes):**

| Program | Integration tests | Unit tests | Modules |
|---|---|---|---|
| gig | 68 | 0 | 8 |
| escrow | 135 | 4 (`utils.rs`) | 12 |
| reputation | 135 | 9 (`utils.rs`) | 12 |
| **Total** | **338** | **13** | **32** |

`cargo test` (gig → escrow → reputation) runs all **351** tests, **0 failures**.

### Gig modules

| Module | Tests | Validates |
|---|---|---|
| `lifecycle.rs` | 14 | Full gig lifecycle across all seven `GigStatus` variants, including `InProgress`/`Completed` via CPI and manual `complete_gig` |
| `gig_updates.rs` | 12 | `update_gig` Draft-only semantics |
| `gig_creation.rs` | 11 | `initialize_gig` field initialization and rejects |
| `freelancer_assignment.rs` | 8 | Assignment rules, self-assignment, double assignment |
| `authorization.rs` | 6 | Wrong-signer rejection on client instructions |
| `publishing.rs` | 6 | `publish_gig` preconditions |
| `pda_security.rs` | 5 | Spoofed/substituted gig PDA rejection |
| `cpi_authorization.rs` | 5 | The three CPI-only instructions reject any direct, non-CPI caller |

### Escrow modules

| Module | Tests | Validates |
|---|---|---|
| `state_transitions.rs` | 18 | Every legal `MilestoneStatus`/CPI-driven `GigStatus` transition accepted, every illegal one rejected |
| `authorization.rs` | 15 | Wrong-signer rejection on every privileged instruction |
| `events.rs` | 14 | Emitted event presence and field correctness |
| `pda_security.rs` | 13 | Spoofed/substituted PDA rejection, including cross-gig vault substitution |
| `escrow_flow.rs` | 12 | Multi-milestone escrow sequencing |
| `gig_escrow_integration.rs` | 11 | Cross-program CPI integration: fund → `InProgress`, approve → `Completed`, cancel → `Cancelled`, and direct-call rejection of the gig CPI-only instructions |
| `token_validation.rs` | 11 | Mint mismatch, wrong owner, wrong token account |
| `happy_path.rs` | 10 | End-to-end create → fund → deliver → approve |
| `timeout_boundaries.rs` | 8 | Exact-boundary behavior at 72h and 7d |
| `vault_accounting.rs` | 8 | `total_locked` / `total_released` / `milestone_count` / `active_milestone` reconciliation |
| `arithmetic.rs` | 7 | Overflow/underflow paths |
| `validation.rs` | 7 | Input bounds on `create_milestone`/`fund_milestone`, `Assigned`/`InProgress` gig-status gating |

### Reputation modules

| Module | Tests | Validates |
|---|---|---|
| `badge_system.rs` | 18 | Eligibility rules, duplicate prevention, all 7 badge types |
| `rating_submission.rs` | 17 | Rating writes, profile folding, duplicate-job rejection |
| `pda_security.rs` | 12 | Spoofed profile/rating/badge rejection |
| `completion_updates.rs` | 11 | Counter movement for success and cancellation |
| `reputation_algorithm.rs` | 11 | Score weighting, caps, penalties |
| `profile_creation.rs` | 10 | Initialization and re-init rejection |
| `profile_authorization.rs` | 10 | `REPUTATION_AUTHORITY` enforcement |
| `state_invariants.rs` | 10 | Counter and score invariants hold after arbitrary sequences |
| `events.rs` | 10 | Event field correctness |
| `regressions.rs` | 10 | Locked-in fixes |
| `rating_validation.rs` | 9 | Score bounds, self-dealing |
| `math.rs` | 7 | Average and score math edge cases |

Shared helpers live in `tests/common/` (litesvm setup, mint and token-account creation, PDA derivation, clock warping). There are no performance or benchmark suites in the repository.

Full module-by-module breakdown: [TESTING.md](./TESTING.md).

---

## Tech Stack

| Layer | Technology | Version / notes |
|---|---|---|
| On-chain language | Rust | 2021 edition |
| Program framework | Anchor (`anchor-lang`) | 1.1.2, `init-if-needed` feature |
| Token integration | `anchor-spl` | 1.1.2, `token` feature — gig (for the `Mint` account type) and escrow |
| Program testing | litesvm | 0.10.0 |
| Test support crates | `solana-message`, `solana-transaction`, `solana-signer`, `solana-keypair`, `solana-pubkey`, `solana-clock`, `spl-token-interface` | 3.x / 2.0 |
| Build | Cargo workspace + Anchor CLI | `resolver = "2"`, release profile with `overflow-checks`, fat LTO, `codegen-units = 1` |
| Package manager (Anchor) | Yarn | per `Anchor.toml` `[toolchain]` |
| Frontend | Next.js | 16.2.11 |
| UI | React 19.2.4, Tailwind CSS 4 | |
| Wallet | `@solana/wallet-adapter-{base,react,react-ui,wallets}`, `@solana/web3.js` 1.98 | configured for **devnet** via `clusterApiUrl("devnet")` |
| Frontend tooling | TypeScript 5, ESLint 9 (`eslint-config-next`) | |
| Off-chain indexer | — | Directory scaffolded, no implementation, no stack chosen |
| CI/CD | — | None configured |

No environment variables are read anywhere in the repository; `.env*` is gitignored and no `.env.example` exists.

---

## Repository Structure

```
.
├── Cargo.toml                   Workspace: members = [programs/gig, programs/escrow, programs/reputation]
├── Anchor.toml                  Program IDs (localnet), provider, test script
├── programs/
│   ├── gig/                     Gig metadata + lifecycle program  ✅ complete
│   │   ├── src/
│   │   │   ├── lib.rs               #[program] entrypoints (7 instructions: 4 client + 3 CPI-only)
│   │   │   ├── constants.rs         Seeds, length caps, ESCROW_AUTHORITY_SEED, ESCROW_PROGRAM_ID
│   │   │   ├── errors.rs            GigError (15 variants)
│   │   │   ├── events.rs            8 typed Anchor events
│   │   │   ├── state/               Gig, GigStatus enum (Draft…Archived, incl. InProgress)
│   │   │   └── instructions/        One file per instruction
│   │   └── tests/                   8 litesvm modules, 68 tests, shared common/
│   ├── escrow/                  Milestone escrow program  ✅ complete
│   │   ├── src/
│   │   │   ├── lib.rs               #[program] entrypoints (7 instructions)
│   │   │   ├── constants.rs         Seeds, timeouts, release percentages, ESCROW_AUTHORITY_SEED
│   │   │   ├── errors.rs            EscrowError (11 variants)
│   │   │   ├── events.rs            7 typed Anchor events
│   │   │   ├── utils.rs             checked_add/sub, percent_of (+ 4 unit tests)
│   │   │   ├── state/               Milestone, EscrowVault (+ milestone_count/active_milestone), status enums
│   │   │   └── instructions/        One file per instruction; CPIs into `gig` on status-advancing paths
│   │   └── tests/                   12 litesvm modules, 135 tests, shared common/
│   ├── reputation/              Reputation program  ✅ complete
│   │   ├── src/
│   │   │   ├── lib.rs               #[program] entrypoints (5 instructions)
│   │   │   ├── constants.rs         Seeds, REPUTATION_AUTHORITY, score caps
│   │   │   ├── errors.rs            ReputationError (11 variants)
│   │   │   ├── events.rs            5 events (4 emitted)
│   │   │   ├── utils.rs             Scoring + badge eligibility (+ 9 unit tests)
│   │   │   ├── state/               UserProfile, Rating, Badge, BadgeType
│   │   │   └── instructions/        One file per instruction
│   │   └── tests/                   12 litesvm modules, 135 tests
│   └── dispute/                 ⛔ Placeholder — src/.gitkeep only
├── apps/web/                    Next.js 16 frontend (scaffold)
│   └── src/
│       ├── app/                     Default landing page; route dirs are .gitkeep stubs
│       ├── components/wallet/       SolanaProvider (devnet wallet adapter) — the only real component
│       └── lib/                     .gitkeep stubs for anchor/supabase/privy/moonpay/shadow-drive
├── services/reputation-indexer/ ⛔ Empty scaffold (db/, api/, indexer/ are .gitkeep)
├── packages/{idl,shared-types,config}/  ⛔ Empty
├── infrastructure/{scripts,supabase}/   ⛔ Empty
├── docs/details.md              Product vision (describes planned dispute/jury, Privy,
│                                Shadow Drive, MoonPay, 0.5% fee — none implemented)
├── ARCHITECTURE.md              full account/PDA/instruction/CPI design
├── SECURITY.md                  threat model and audit writeup
├── TESTING.md                   Test module breakdown
├── IMPLEMENTATION_PROGRESS.md   Per-program completion status
├── CHANGELOG.md                 Release history
└── LICENSE                      MIT
```

`docs/details.md` is a product-vision document written ahead of implementation. Where it conflicts with this README — jury disputes, a 0.5% platform fee, Privy embedded wallets, Shadow Drive storage, MoonPay off-ramp, automatic escrow→reputation CPI, badge NFT minting — **the README reflects the code and `details.md` reflects intent.** None of those items exist in the repository.

---

## Developer Guide

### Prerequisites

| Tool | Purpose |
|---|---|
| Rust toolchain (with `cargo`) | Build and test the programs |
| [Anchor CLI](https://www.anchor-lang.com/docs/installation) | `anchor build` / `anchor deploy` / IDL generation |
| Solana CLI | Keypairs, airdrops, deployment |
| Node.js + Yarn | Anchor's configured package manager |
| npm | The frontend has its own `package-lock.json` |

The Cargo test suite needs only Rust — litesvm runs in-process, with no validator and no Solana CLI required.

### Setup

```bash
git clone <repo-url> && cd hexafalls
cargo build                 # builds all three programs as native libs
cd apps/web && npm install  # frontend deps
```

### Testing

```bash
cargo test                        # all three programs: 351 tests
cargo test -p gig                 # 68
cargo test -p escrow              # 139
cargo test -p reputation          # 144
cargo test -p escrow --test authorization    # a single module
cargo test -p escrow --test authorization -- --nocapture
```

`anchor test` runs the script defined in `Anchor.toml`, which is the same three `cargo test` invocations:

```toml
[scripts]
test = "cargo test --manifest-path programs/gig/Cargo.toml && cargo test --manifest-path programs/escrow/Cargo.toml && cargo test --manifest-path programs/reputation/Cargo.toml"
```

### Building deployable artifacts

```bash
anchor build                # SBF binaries + IDLs under target/
```

The release profile in the workspace `Cargo.toml` enables `overflow-checks = true`, fat LTO, and `codegen-units = 1`. Do not disable `overflow-checks` — the arithmetic safety argument in [Security Model](#security-model) depends on it as a backstop behind the explicit checked helpers.

### Running a local validator

```bash
solana-test-validator                       # terminal 1
anchor deploy --provider.cluster localnet   # terminal 2
```

`Anchor.toml` pins `cluster = "localnet"` and `wallet = "~/.config/solana/id.json"`.

### Deploying elsewhere

Nothing is deployed today. To target devnet: set `[provider] cluster = "devnet"`, fund the deployer keypair, then `anchor deploy --provider.cluster devnet`. If the deployed program ids differ from those pinned in `Anchor.toml`, update both `Anchor.toml` and the `declare_id!` in each `lib.rs` and rebuild — a mismatch breaks every PDA derivation.

Note: `REPUTATION_AUTHORITY` in `programs/reputation/src/constants.rs` is compiled into the program. Changing it for a non-local deployment requires a rebuild and redeploy.

### Frontend

```bash
cd apps/web
npm run dev     # http://localhost:3000
npm run build
npm run lint    # eslint
```

The frontend currently renders the default Next.js starter page and wires a devnet wallet-adapter provider. It does not talk to either program.

### Formatting and linting

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cd apps/web && npm run lint
```

No `rustfmt.toml` or `clippy.toml` is committed; defaults apply. `unexpected_cfgs` is configured as a warning in both program manifests for the `target_os = "solana"` cfg.

---

## Documentation Index

| Document | Contents |
|---|---|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | §1–10 escrow: protocol overview, account architecture, instruction flow, state machine, event architecture, token flow, PDA architecture. §11–19 reputation: account model, PDA architecture, instruction flow, state transitions, events, future CPI compatibility |
| [SECURITY.md](./SECURITY.md) | §1–14 escrow threat model and audit. §15–26 reputation threat model, trust assumptions, and audit |
| [TESTING.md](./TESTING.md) | Per-module test breakdown for both programs |
| [IMPLEMENTATION_PROGRESS.md](./IMPLEMENTATION_PROGRESS.md) | Completion status per program |
| [CHANGELOG.md](./CHANGELOG.md) | Release history for both programs and their docs |
| [docs/details.md](./docs/details.md) | Product vision and planned architecture — **aspirational, not implemented** |
| [LICENSE](./LICENSE) | MIT |
| [AGENTS.md](./AGENTS.md) / [CLAUDE.md](./CLAUDE.md) | Repository conventions for AI coding agents |

There are no per-program READMEs and no SDK documentation, because no SDK exists.

`ARCHITECTURE.md`, `SECURITY.md`, `TESTING.md`, and `IMPLEMENTATION_PROGRESS.md` are synced to the current tree (gig + escrow split, 7 + 7 instructions, 351 tests total). `CHANGELOG.md` keeps historical entries as originally written; it records the gig/escrow program split and which earlier figures were superseded.

---

## Roadmap

### Completed

- [x] Gig program — 7 instructions (4 client + 3 CPI-only) covering gig metadata and lifecycle
- [x] Escrow program — 7 instructions covering milestone escrow, split out of the original merged program
- [x] Escrow → Gig CPI — `escrow_authority` PDA, `seeds::program`-gated, advances `GigStatus` on funding/completion/cancellation
- [x] PDA custody model — vault authority is a PDA with no keypair
- [x] SPL Token integration via `transfer_checked` on both funding and PDA-signed release
- [x] Permissionless timeout releases (72h partial / 7d full)
- [x] 8 typed gig events + 7 typed escrow events for indexing
- [x] Reputation program — profiles, immutable ratings, 7 badge types
- [x] Deterministic, recomputable reputation scoring
- [x] Structural duplicate prevention via PDA seeds (one rating per job, one badge per type)
- [x] 351 tests across all three programs
- [x] Internal security audit of all three programs, no open findings
- [x] Architecture, security, and testing documentation

### In progress

Nothing is partially implemented in the on-chain programs. The frontend scaffold (`apps/web`) is the only component with started-but-incomplete work: a wallet provider exists, program integration does not.

### Planned

Ordered by dependency, drawn from `IMPLEMENTATION_PROGRESS.md`, `ARCHITECTURE.md` §18, `SECURITY.md` §15.4, and `docs/details.md`.

- [ ] **Devnet deployment** of all three programs, with IDLs published to `packages/idl`
- [ ] **Escrow → Reputation CPI** — `approve_milestone` calls `update_completion` directly, removing the `REPUTATION_AUTHORITY` trust assumption. Designed to require no account-layout change
- [ ] **Dispute program** (`programs/dispute`) — evidence submission, juror voting, resolution; escrow gains a dispute trigger that freezes remaining funds
- [ ] **Frontend integration** — gig creation, hiring, funding, delivery, approval, and reputation views against live programs
- [ ] **Reputation indexer** (`services/reputation-indexer`) — consume program events, expose a public read API for external platforms
- [ ] **Shared packages** — generated IDLs, TypeScript types, shared config
- [ ] **CI** — no workflows exist; `cargo test` + `cargo clippy` + `npm run lint` on PR is the obvious first step
- [ ] **External security audit** before mainnet
- [ ] **Product-vision items from `docs/details.md`, not yet designed on-chain:** NFT badge minting, platform fee, Privy embedded wallets, Shadow Drive delivery storage, MoonPay off-ramp

There is no governance program, and none is currently planned in the repository.

---

## Contributing

**Branching.** `master` is the main branch. Work on a feature branch and open a PR against `master`.

**Commit conventions.** The existing history uses Conventional Commits — `feat:`, `fix:`, `test:`, `docs:`, with a scope-free imperative subject:

```
feat: add reputation program
test: split reputation test suite into focused modules
docs: add architecture, security, testing, and changelog docs
```

**Coding standards.**

- `cargo fmt --all` and `cargo clippy --workspace --all-targets` clean before review.
- One instruction per file under `src/instructions/`, exported through `mod.rs`, with a thin `#[program]` wrapper in `lib.rs` that only forwards to `handler`.
- Express preconditions as Anchor account constraints where possible (`has_one`, `constraint =`, `address =`, `seeds =`) rather than as handler-body checks. A constraint cannot be skipped by an early return.
- Never introduce unchecked arithmetic on balances or counters. Use the `utils.rs` helpers.
- Every state-changing instruction emits an event.
- New errors go in `errors.rs` with a `#[msg]` string, not `ProgramError` or `msg!` + generic failure.

**Testing requirements.**

- Every new instruction needs coverage in at least: its own behavior module, `authorization.rs` (wrong-signer rejection), `state_transitions.rs` or `lifecycle.rs` (illegal-precondition rejection), and `events.rs` (emitted fields).
- Bug fixes get a regression test that fails before the fix.
- Run the full suite (`cargo test`) before opening a PR — not just the module you touched.

**Security review.** Any change touching custody, PDA derivation, authorization, or arithmetic must be checked against the invariants in [SECURITY.md](./SECURITY.md) and the invariant list in [Security Model](#security-model). If a change alters an invariant, the PR should say so explicitly.

---

## License

MIT — see [LICENSE](./LICENSE). Copyright (c) 2026 PayGig.

**No warranty.** These programs are unaudited by any external firm and undeployed. Do not use them to custody real value without an independent security audit.
