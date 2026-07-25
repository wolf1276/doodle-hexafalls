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

---

# Reputation Program — Architecture

Status: **Production Ready**. Program: `programs/reputation` (Anchor, `declare_id!("mXn62yZ4KFvPsdtMmEdGkB71jXcr17SQJHXftgPVGNB")`).

## 11. Program Overview

The Reputation program is an independently-deployed Anchor program that tracks, on-chain, a freelancer's job-completion history, ratings, and derived reputation score. It has exactly one job: **maintain a tamper-evident, deterministically-recomputable reputation record per authority.** It holds no funds and has no custody responsibilities — see §3 above for why this is a separate program from Escrow rather than a module inside it.

## 12. Responsibilities

- Create one reputation profile per user authority (`initialize_profile`).
- Record an immutable, one-time rating per job (`submit_rating`).
- Record the outcome (success/cancellation) and earnings of a completed job, and recompute the profile's deterministic reputation score (`update_completion`).
- Award badges to profiles that meet deterministic on-chain eligibility criteria, or that a trusted authority attests to (`award_badge`).
- Expose a profile's current reputation score for on-chain composability (`get_profile`).

It does **not** move funds, does not verify that a `job_id` corresponds to a real Escrow `Gig`, and does not itself decide who the "client" and "freelancer" of a job are — those identities are supplied by the caller and are only as trustworthy as the caller. See §14.4 (Trust Assumptions) for the security implications of this.

## 13. Account Model

### 13.1 UserProfile PDA

```
UserProfile {
  authority: Pubkey        // the user this profile belongs to
  completed_jobs: u64
  successful_jobs: u64
  cancelled_jobs: u64
  total_earnings: u64
  rating_sum: u64           // sum of all raw 1-5 scores ever submitted
  rating_count: u64
  average_rating: u32        // rating_sum * 100 / rating_count, i.e. scaled by RATING_SCALE
  reputation_score: u64      // deterministic, recomputed on every mutation, 0..=1000
  badges_earned: u32
  created_at: i64
  updated_at: i64
  bump: u8
}
```

One `UserProfile` per authority. It is the single source of truth every reputation figure is derived from — `reputation_score` is never independently mutated, only recomputed from the other fields (§16).

### 13.2 Rating PDA

```
Rating {
  job_id: u64
  client: Pubkey
  freelancer: Pubkey
  score: u8                 // 1-5
  review_hash: [u8; 32]      // hash of off-chain review text; raw text stays off-chain
  submitted_at: i64
  bump: u8
}
```

One `Rating` per `job_id`. Immutable after creation — there is no `update_rating` or `delete_rating` instruction, so once written a rating can never be edited, retracted, or overwritten (§14.7).

### 13.3 Badge PDA

```
Badge {
  profile: Pubkey            // the UserProfile this badge was awarded to
  badge_type: BadgeType       // FirstGig | TenCompletedJobs | HundredCompletedJobs |
                               // FiveStarPerformer | TrustedFreelancer | FastDeliverer | TopRated
  issuer: Pubkey              // always REPUTATION_AUTHORITY today
  issued_at: i64
  metadata: String            // free-form, max 128 bytes (e.g. a URI)
  bump: u8
}
```

One `Badge` PDA per `(profile, badge_type)` pair — a profile can hold at most one badge of each type (§14.8).

## 14. PDA Architecture

### 14.1 Why PDAs

As with Escrow (§8.1), every stateful account here is a Program Derived Address: computed deterministically from seed bytes and the program ID, with **no corresponding private key**. Nothing can sign for a `UserProfile`, `Rating`, or `Badge` PDA except this program itself, via `invoke_signed` — and in practice this program never needs to sign *outbound* from these PDAs, since it holds no funds. The value of PDAs here is entirely about **deterministic addressing and spoofing resistance**, not custody: anyone can independently recompute a profile's, rating's, or badge's address and be certain it either doesn't exist, or holds exactly the data the program itself wrote.

### 14.2 Seed Derivation

| PDA | Seeds | Rationale |
|---|---|---|
| **UserProfile PDA** | `[PROFILE_SEED, authority.key()]` | One profile per authority, addressable by anyone who knows the authority's pubkey alone — no registry or off-chain index needed to find "Alice's profile" |
| **Rating PDA** | `[RATING_SEED, job_id.to_le_bytes()]` | Keyed solely by `job_id` (not by client/freelancer) so that a second `submit_rating` call for the same job fails at `init` time — this **is** the duplicate-rating guard, not a separate check (§14.6) |
| **Badge PDA** | `[BADGE_SEED, profile.authority.as_ref(), &[badge_type.as_seed()]]` | Scoped to both the profile and the badge type, so `award_badge` for a badge type the profile already holds fails at `init` time — this **is** the duplicate-badge guard (§14.8) |

`PROFILE_SEED`, `RATING_SEED`, `BADGE_SEED` are `#[constant]` byte strings (`programs/reputation/src/constants.rs`), embedded in the IDL for client-side derivation.

### 14.3 Account Relationships

```
UserProfile (1, seeded by authority)
      │
      ├──< Badge (0..7, seeded by profile.authority + badge_type — at most one per type)
      │
      └── updated by ──< Rating (N, seeded by job_id — each rating folds
                            its score into exactly one freelancer's profile)
```

A `Rating` references its `freelancer`'s `UserProfile` by account, not by embedding — `submit_rating` takes the `freelancer_profile` account directly and mutates it in the same instruction that creates the `Rating`.

### 14.4 Deterministic Addressing & Ownership Model

Because every PDA is derived from public inputs (an authority pubkey, a job ID, a badge type enum), any client, indexer, or auditor can compute a profile/rating/badge's address without querying the chain first, and can verify that an account claiming to be "Alice's profile" is in fact at the one address that could ever hold that role. There is no admin-settable "which account is the real profile" mapping to trust — the seed derivation *is* the mapping.

### 14.5 Why No Private Keys Exist — and Why That Matters Here

A PDA is deliberately computed off the Ed25519 curve, so no keypair exists that could sign for it. Combined with Anchor's `seeds`/`bump` account constraints (re-derived and matched on every instruction, exactly as in Escrow §8.3), this means:

- **No one can fabricate a `UserProfile`, `Rating`, or `Badge` at an address other than its one canonical derivation** — an attacker cannot pass an account they control and claim it is "the freelancer's profile" for a given authority; the derived address would not match and Anchor's constraint check fails the transaction (§14.6, verified by `tests/pda_security.rs`, 24 tests).
- **Program signing** is not exercised for outbound transfers here (no funds are held), but the same `invoke_signed` mechanism that protects Escrow's vault is available for any future instruction that needs the program to act as one of these PDAs.
- **Bump seeds**: every PDA stores its own `bump`, captured at `init` via `ctx.bumps.<account>`; every later instruction constrains reuse with `bump = stored_bump`, so a caller cannot supply an alternate bump to derive a different, attacker-influenced address that happens to collide.

### 14.6 Why Each Account Has Its Own PDA

Splitting `UserProfile`, `Rating`, and `Badge` into independently-addressed PDAs (rather than, say, an array of ratings inside the profile) gives:

- **Fixed account size** for `UserProfile` — it never grows regardless of how many ratings or badges a user accumulates, so there's no realloc/resize logic and no per-profile cap on rating or badge count.
- **Parallel, conflict-free writes** — two different freelancers' profiles, or two different jobs' ratings, never contend for the same account.
- **The seed itself enforces the core invariants** (one rating per job, one badge per type per profile) instead of requiring a manual "does this already exist" scan.

## 15. Instruction Flow

```
initialize_profile (authority signs)
      │
      ▼
UserProfile { all counters = 0 }
      │
      ├──► submit_rating (client signs, job_id + score 1-5 + review_hash)
      │        └──► Rating PDA created (job_id-keyed, immutable)
      │        └──► freelancer_profile.rating_sum/rating_count/average_rating/
      │              reputation_score updated in the same instruction
      │
      ├──► update_completion (REPUTATION_AUTHORITY signs, successful + earnings)
      │        └──► completed_jobs += 1; successful_jobs or cancelled_jobs += 1;
      │              total_earnings += earnings (only if successful);
      │              reputation_score recomputed
      │
      ├──► award_badge (REPUTATION_AUTHORITY signs, badge_type + metadata)
      │        └──► is_eligible_for_badge(profile, badge_type) checked
      │        └──► Badge PDA created (profile + badge_type-keyed, one per type)
      │
      └──► get_profile (read-only, no signer) ──► returns reputation_score
```

There is no ordering dependency between `submit_rating` and `update_completion` — a client can rate a job before or after the authority records its completion; both simply fold their respective inputs into the profile and recompute `reputation_score` from the resulting totals.

## 16. State Transitions

Unlike Escrow's `Gig`/`Milestone`, `UserProfile` has no discrete status enum — its "state" is the accumulated counters, and every mutating instruction moves it strictly forward:

| Field | Only ever | Enforced by |
|---|---|---|
| `completed_jobs`, `successful_jobs`, `cancelled_jobs` | increases | `checked_add`, no decrement instruction exists |
| `total_earnings` | increases | `checked_add`, only on `successful = true` |
| `rating_sum`, `rating_count` | increase | `checked_add`, one increment per `Rating` PDA ever created |
| `average_rating` | recomputed from `rating_sum`/`rating_count` | exact recomputation each time, not incremental blending (`utils::average_rating`) — avoids any drift from repeated rounding |
| `reputation_score` | recomputed from current totals | `utils::compute_reputation_score`, pure function of the profile's stored fields, called at the end of `submit_rating`, `update_completion`, and (implicitly, via `badges_earned`) `award_badge` |
| `created_at` | set once, at `init` | never written again |
| `updated_at` | monotonically non-decreasing | set to `Clock::get()?.unix_timestamp` on every mutating instruction |

`reputation_score` is a deterministic function of `(completed_jobs, successful_jobs, total_earnings, average_rating, cancelled_jobs)` alone — see §21.5. It is never set directly by an instruction argument, so there is no code path that lets a caller assign an arbitrary score.

## 17. Event Architecture

| Event | Emitted by | Notes |
|---|---|---|
| `ProfileCreated` | `initialize_profile` | |
| `RatingSubmitted` | `submit_rating` | includes `new_average_rating` so indexers don't need to re-derive it |
| `CompletionUpdated` | `update_completion` | includes the recomputed `reputation_score` |
| `BadgeAwarded` | `award_badge` | |
| `ProfileUpdated` | *(defined, not currently emitted by any instruction)* | Reserved for a future generic "profile mutated" event; today `RatingSubmitted`/`CompletionUpdated`/`BadgeAwarded` each already carry the fields an indexer needs, so nothing currently emits it. Documented here rather than silently left as dead code. |

Event coverage is verified in `tests/events.rs` (20 tests) — every emitting instruction's event fields are asserted against the instruction's actual effects.

## 18. Future CPI Compatibility

`update_completion` and `award_badge` are gated by `#[account(address = REPUTATION_AUTHORITY)]` — a single hardcoded pubkey (`programs/reputation/src/constants.rs`) acting as a trusted off-chain (or, later, on-chain) caller. This is a deliberate MVP simplification, documented in the source:

```rust
/// Signer authorized to record job completions until the Escrow Program
/// can invoke `update_completion` directly via CPI. Swapping this for a
/// CPI-only check later does not require any account layout changes.
pub const REPUTATION_AUTHORITY: Pubkey = pubkey!("vo18wuiY77EZa16yYKRdAjp2mj3g6GCvMHH8wkn6LAz");
```

Because the account constraint is a simple `address = REPUTATION_AUTHORITY` check on a `Signer`, migrating to a CPI-only model later (Escrow's `approve_milestone` invoking `update_completion` via CPI, signed by an Escrow-owned PDA) requires changing only that one constraint — no change to `UserProfile`, `Rating`, or `Badge` account layouts, and no change to `submit_rating` (already permissionless-by-the-client) or `award_badge`'s eligibility logic. This is why the account model was designed with fixed, independently-addressed PDAs rather than embedding completion data inside an Escrow-owned account: the two programs can compose later without either being redesigned. See §21.4 for the security implications of this trust assumption in its current, pre-CPI form.

## 19. Design Rationale Summary

| Decision | Why |
|---|---|
| `Rating` PDA seeded by `job_id` alone (not `job_id + client`) | The seed itself is the duplicate-submission guard — a second `submit_rating` for the same job fails at `init`, with no separate "already rated" existence check needed |
| `average_rating` recomputed from `rating_sum`/`rating_count` every time, not blended incrementally | Guarantees the stored average is always the exact mean of every rating ever submitted, regardless of submission order — no floating-point or rounding drift accumulates |
| `reputation_score` is a pure function of stored counters, recomputed on every mutation | Anyone can independently recompute and verify a profile's score from its public fields alone; there is no code path where a score is set directly, so it cannot silently diverge from the data that justifies it |
| Badge eligibility is deterministic where possible (`FirstGig`, `*CompletedJobs`, `FiveStarPerformer`, `TopRated`), authority-attested otherwise (`TrustedFreelancer`, `FastDeliverer`) | Some signals (delivery timing, external endorsements) aren't tracked on-chain by this program; rather than fabricate an on-chain proxy for them, those two badge types are explicitly documented as authority-attested, while duplicate-award protection (one PDA per type) still applies uniformly to all seven types |
| `REPUTATION_AUTHORITY` is a single hardcoded pubkey today, not a CPI check | Lets `update_completion`/`award_badge` ship and be fully tested before Escrow's CPI surface to this program is built; the account-layout compatibility is preserved for that migration (§18) |
| Reputation is a separate program from Escrow | See §3 — isolates the audited payment path from reputation-scoring logic that will change more often |
