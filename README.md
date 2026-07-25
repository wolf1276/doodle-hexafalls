# PayGig

PayGig is a decentralized freelance payment protocol built on Solana. It replaces the centralized intermediary that traditional freelance platforms require to hold funds, arbitrate payments, and maintain reputation — with a set of transparent, auditable on-chain programs.

## The Problem

On centralized freelance platforms, a client's funds, a freelancer's payout, and a freelancer's reputation history are all custodied by the platform operator. Users must trust that operator to:

- Actually hold client funds until work is delivered, rather than commingle or lose them.
- Release payment fairly when work is disputed.
- Not alter, delete, or fabricate a freelancer's rating and job history.
- Remain solvent and operational.

None of these guarantees are independently verifiable. PayGig removes the operator from the trust boundary: funds are held by program-derived vaults governed by fixed, auditable rules, and reputation is a deterministic function of on-chain history that anyone can recompute and verify.

## Features

- **Non-custodial escrow** — client funds are locked in a vault controlled only by program logic, never by a private key any single party holds.
- **Milestone-based payments** — a gig is split into independently funded, delivered, and released milestones rather than one all-or-nothing payment.
- **Automatic timeout protection** — if a client goes silent after a freelancer submits work, funds release on a fixed schedule (partial at 72h, full at 7d) without requiring either party to act.
- **On-chain reputation** — job completion history, ratings, and a deterministic reputation score are recorded on-chain and independently recomputable by anyone.
- **Transparent payment lifecycle** — every state transition emits a typed event, giving indexers and auditors a complete, replayable log without polling account state.
- **Modular protocol architecture** — payments, reputation, and (future) dispute resolution are independent programs with their own audit boundaries, not one monolith.

## Architecture Overview

PayGig splits on-chain responsibility across independent programs instead of building one large program that does everything:

```
                     ┌────────────────────┐
                     │       Client         │
                     └──────────┬──────────┘
                                │
                                ▼
                     ┌────────────────────┐
                     │   Escrow Program     │  holds & releases milestone payments
                     └──────────┬──────────┘
                                │
                                ▼
                     ┌────────────────────┐
                     │      Vault PDA        │  SPL token custody, no private key
                     └──────────┬──────────┘
                                │
                                ▼
                     ┌────────────────────┐
                     │     Freelancer        │
                     └──────────┬──────────┘
                                │
                                ▼
                     ┌────────────────────┐
                     │ Reputation Program   │  records rating, updates score
                     └────────────────────┘
```

This is deliberate, not incidental:

- **Smaller attack surface per program.** A program that only moves SPL tokens between a vault and two known parties is far easier to audit and formally reason about than one that also handles ratings, badges, or dispute votes.
- **Independent upgrade paths.** Reputation-scoring rules and future dispute mechanics will change far more often than escrow's payment rules should. Coupling them would force a re-audit of vault-custody logic every time an unrelated scoring tweak ships.
- **Failure isolation.** A bug in the reputation program cannot touch funds already locked in an escrow vault — the vault's signing authority is a PDA owned solely by the escrow program's own seeds.
- **Composability without shared trust today.** Escrow does not currently CPI into reputation or a dispute program. Reputation updates and (eventually) dispute resolution are additive integrations layered on later, once each program has its own completed audit — see [Future Components](#future-components).

## Protocol Components

### Escrow Program

**Status: Production ready.** Path: [`programs/escrow`](./programs/escrow). Holds exactly one responsibility: lock client funds and release them to a freelancer according to a fixed set of rules. It has no knowledge of ratings, disputes, or reputation.

| | |
|---|---|
| Program ID | `FFJ8YAVGUJP4SeDZrQ3g1d9fdQFq9hutsU1m4f3o1UXS` |
| Core accounts | `Gig`, `Milestone`, `EscrowVault` (all PDAs) |
| Instructions | `initialize_gig`, `create_milestone`, `fund_milestone`, `submit_delivery`, `approve_milestone`, `partial_timeout_release`, `full_timeout_release`, `cancel_before_funding` |
| Token integration | SPL Token `transfer_checked` only — no legacy `transfer`, no outbound CPI to any other custom program |
| Tests | 107+ integration tests across 10 modules (litesvm) |

### Reputation Program

**Status: Production ready.** Path: [`programs/reputation`](./programs/reputation). Maintains a tamper-evident, deterministically recomputable reputation record per user authority. Holds no funds and has no custody responsibilities.

| | |
|---|---|
| Program ID | `mXn62yZ4KFvPsdtMmEdGkB71jXcr17SQJHXftgPVGNB` |
| Core accounts | `UserProfile`, `Rating`, `Badge` (all PDAs) |
| Instructions | `initialize_profile`, `submit_rating`, `update_completion`, `award_badge`, `get_profile` |
| Score model | `reputation_score` is a pure function of stored counters (completed/successful/cancelled jobs, earnings, average rating), recomputed on every mutation — never set directly |
| Tests | 145+ integration tests across 12 modules (litesvm) |

### Future Components

| Component | Status | Purpose |
|---|---|---|
| Dispute Program | Not yet implemented (`programs/dispute` is a placeholder) | Third-party or jury-based resolution when a client and freelancer disagree after a milestone timeout |
| Escrow → Reputation CPI | Not yet wired | `approve_milestone` will eventually call `update_completion` directly via CPI, replacing today's `REPUTATION_AUTHORITY`-signed trust boundary (see [Security Model](#security-model)) |

## Repository Structure

```
programs/
  escrow/           # Escrow Anchor program (production)
  reputation/       # Reputation Anchor program (production)
  dispute/          # Placeholder — not yet implemented

apps/
  web/              # Next.js frontend

services/
  reputation-indexer/  # Off-chain indexer consuming program events

packages/
  idl/              # Generated Anchor IDLs
  shared-types/      # Shared TypeScript types
  config/            # Shared config

infrastructure/
  scripts/          # Deployment / ops scripts
  supabase/          # Off-chain data store config

docs/
  details.md        # Product vision and future composition notes
```

## Technology Stack

| Layer | Technology |
|---|---|
| On-chain programs | Rust, Anchor |
| Token standard | SPL Token (`transfer_checked`) |
| Program testing | litesvm |
| Frontend | Next.js, TypeScript |
| Off-chain indexing | Rust service consuming on-chain events |

## Protocol Flow

```
Client creates gig
        │
        ▼
Milestone created (PendingFunding)
        │
        ▼
Client funds escrow ──► transfer_checked ──► Vault holds SPL tokens
        │
        ▼
Freelancer submits work (Submitted, submitted_at recorded)
        │
        ├──► Client approves ──────────────────────► Funds released (100%)
        │
        ├──► No response, 72h elapses ──► partial_timeout_release (20% released)
        │           │
        │           └──► No response, 7d elapses ──► full_timeout_release (remaining 80%)
        │
        ▼
Reputation updated (submit_rating / update_completion)
```

`cancel_before_funding` is the only exit prior to funding — once a milestone is funded, funds can only leave the vault through approval or timeout release, never through cancellation.

## Program Architecture

Full account-level, instruction-level, and PDA-level design for both programs — including diagrams, seed derivation tables, and state machines — is documented in [ARCHITECTURE.md](./ARCHITECTURE.md). Summary below.

### Account Architecture

**Escrow**

```
Gig (1) ──┬──< Milestone (N, seeded by gig + index)
          │
          └──< EscrowVault (1, seeded by gig)
                    │
                    └── Vault Token Account (authority = EscrowVault PDA)
```

**Reputation**

```
UserProfile (1, seeded by authority)
      │
      ├──< Badge (0..7, seeded by authority + badge_type — one per type)
      │
      └── updated by ──< Rating (N, seeded by job_id — immutable, one per job)
```

### PDA Architecture

Every stateful account in both programs is a Program Derived Address — computed deterministically from seed bytes and the program ID, with no corresponding private key. Nothing can sign for these accounts except the owning program itself, via `invoke_signed`.

| PDA | Seeds | Guarantee the seed enforces |
|---|---|---|
| `Gig` | `[GIG_SEED, client, gig_id]` | Deterministic, spoof-proof gig addressing |
| `Milestone` | `[MILESTONE_SEED, gig, index]` | Sequential, collision-free milestone addressing |
| `EscrowVault` | `[VAULT_SEED, gig]` | Exactly one vault per gig |
| `UserProfile` | `[PROFILE_SEED, authority]` | Exactly one profile per authority |
| `Rating` | `[RATING_SEED, job_id]` | Duplicate-rating guard — a second `submit_rating` for the same job fails at `init` |
| `Badge` | `[BADGE_SEED, authority, badge_type]` | Duplicate-badge guard — one badge per type per profile |

Because every seed is derived from public inputs, any client, indexer, or auditor can independently compute an account's address and verify it is the one canonical account for that role — there is no admin-settable mapping to trust.

## Security Model

Full threat model and every enforced invariant, for both programs, is documented in [SECURITY.md](./SECURITY.md). Both programs have completed a full internal security audit with no open findings — the reputation program has two explicitly documented trust assumptions accepted as pre-CPI trade-offs (see below).

Enforced across both programs:

- **Signer validation** on every privileged instruction.
- **Ownership and account-type validation** via Anchor's `Account<'info, T>` and `has_one` constraints.
- **PDA re-derivation and anti-spoofing** — every instruction re-derives and matches expected PDAs, not just trusts a passed-in address.
- **Reinitialization and replay protection** — `init` constraints prevent an account from being created twice at the same derived address.
- **State transition validation** — every instruction checks the account's current status before mutating it; invalid transitions are rejected.
- **Checked arithmetic** — all balance and counter math uses `checked_add`/`checked_sub`; no unchecked overflow/underflow path exists.
- **SPL Token integration via `transfer_checked` only** — mint- and decimal-aware transfers, not the legacy `transfer` instruction.

**Known, documented trust assumption:** `update_completion` and `award_badge` are gated by a single hardcoded `REPUTATION_AUTHORITY` pubkey rather than a CPI-only check, since Escrow does not yet CPI into Reputation. This is a deliberate MVP boundary — the account layouts are designed so migrating to CPI-only authorization later requires no schema changes. See §18 of [ARCHITECTURE.md](./ARCHITECTURE.md) and §15.4 of [SECURITY.md](./SECURITY.md).

## State Machines

**Escrow — Milestone lifecycle**

```
PendingFunding ──fund_milestone──► Funded ──submit_delivery──► Submitted
     │                                                             │
     └──cancel_before_funding──► (closed)                          ├──approve_milestone──► Completed
                                                                    │
                                                                    └──72h──► PartialReleased ──7d──► Completed
```

**Reputation — UserProfile**

`UserProfile` has no discrete status enum. Its state is a set of monotonically-moving counters (`completed_jobs`, `successful_jobs`, `cancelled_jobs`, `total_earnings`, `rating_sum`, `rating_count`) from which `average_rating` and `reputation_score` are deterministically recomputed on every mutation — never set directly by instruction input.

## Testing

Both programs are tested with [litesvm](https://github.com/LiteSVM/litesvm) integration tests, run directly against compiled program logic without a local validator.

**Escrow — 107+ tests, 10 modules:** happy path, full state-transition coverage, PDA spoofing/security, authorization, timeout boundary conditions, vault accounting reconciliation, arithmetic overflow/underflow, token mint validation, and event-field correctness.

**Reputation — 145+ tests, 12 modules:** profile creation, rating submission and validation, badge system eligibility, reputation-score calculation, completion updates, PDA security, profile authorization, state invariants, event correctness, and regression coverage.

Full breakdown of every test module and what it guarantees is in [TESTING.md](./TESTING.md).

## Documentation

| Doc | Escrow | Reputation |
|---|---|---|
| Architecture | [ARCHITECTURE.md §1–10](./ARCHITECTURE.md#1-protocol-overview) | [ARCHITECTURE.md §11–19](./ARCHITECTURE.md#11-program-overview) |
| Security | [SECURITY.md §1–14](./SECURITY.md#1-threat-model) | [SECURITY.md §15–26](./SECURITY.md#15-threat-model) |
| Testing | [TESTING.md](./TESTING.md#escrow--reputation-programs--test-suite) | [TESTING.md](./TESTING.md#reputation-program--test-suite) |

Also see [CHANGELOG.md](./CHANGELOG.md) for release history and [IMPLEMENTATION_PROGRESS.md](./IMPLEMENTATION_PROGRESS.md) for completion status across all programs.

## Getting Started

### Prerequisites

- Rust and Cargo
- [Anchor](https://www.anchor-lang.com/) CLI
- Solana CLI (for deployment)
- Node.js and Yarn (for the frontend and Anchor scripts)

### Building

```bash
anchor build
```

### Testing

```bash
# Both programs
cargo test

# A single program
cargo test -p escrow
cargo test -p reputation
```

### Running the frontend

```bash
cd apps/web
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

### Deployment

Program IDs are pinned in [`Anchor.toml`](./Anchor.toml) under `[programs.localnet]`. Deploying to devnet or mainnet requires updating the cluster in `Anchor.toml` and funding a deployer keypair before running `anchor deploy`.

## Project Roadmap

- [x] Escrow program — full instruction set, PDA design, SPL integration, audit, 107+ tests
- [x] Reputation program — full instruction set, PDA design, audit, 145+ tests
- [ ] Escrow → Reputation CPI integration (replacing the `REPUTATION_AUTHORITY` trust boundary)
- [ ] Dispute program
- [ ] Frontend integration with both programs
- [ ] Devnet deployment

## Contributing

Issues and pull requests are welcome. Before submitting a change to either program, run its full test suite (`cargo test -p escrow` / `cargo test -p reputation`) and review [SECURITY.md](./SECURITY.md) for the invariants your change must not violate.

## License

MIT — see [LICENSE](./LICENSE).
