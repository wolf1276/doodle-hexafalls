# Escrow & Reputation Programs — Security

**Audit status: Complete for both programs.** The Escrow program (`programs/escrow`) and the Reputation program (`programs/reputation`) have each completed implementation, a full internal security audit, and their own regression/security test suite — 106 tests (Escrow) and 145 tests (Reputation) — covering every invariant documented below. No open findings in either program.

Scope of this document: `programs/escrow` (§1–14) and `programs/reputation` (§15–26). The Dispute program (`programs/dispute`, unimplemented) is out of scope and tracked separately.

## 1. Threat Model

Actors:

- **Client** — funds milestones, approves releases. Trusted to sign only their own transactions; **not** trusted to act honestly (may go silent, may attempt to re-fund, may attempt to reference someone else's milestone).
- **Freelancer** — submits delivery. Trusted to sign only their own transactions; **not** trusted to fabricate submissions for gigs they aren't party to.
- **Permissionless caller** — anyone, for `partial_timeout_release` / `full_timeout_release`. Must not be able to extract more than the fixed percentage, regardless of who calls it or how many times.
- **Adversarial transaction builder** — may supply arbitrary accounts to any instruction, including accounts that are the right *type* but the wrong *instance* (e.g. a vault from a different gig), or accounts that are uninitialized/attacker-owned, attempting to spoof a PDA.

Assets at risk: SPL tokens held in vault token accounts. The program's job is to guarantee those tokens can only leave a vault via the three defined release paths, in the defined amounts, to the defined recipient.

## 2. Signer Validation

Every instruction that changes ownership-sensitive state requires the correct `Signer<'info>`:

- `initialize_gig` — `client` must sign; `require_keys_neq!(client, freelancer)` prevents a gig where the same key is both parties.
- `create_milestone` / `fund_milestone` / `approve_milestone` / `cancel_before_funding` — `client` must sign, and is additionally checked against `gig.client` via `has_one = client`.
- `submit_delivery` — `freelancer` must sign, checked against `gig.freelancer` via `has_one = freelancer`.
- `partial_timeout_release` / `full_timeout_release` — **intentionally permissionless** (no signer requirement beyond fee-payer). This is a deliberate design choice (§ "Timeout Security" below), not a missing check.

`tests/authorization.rs` (10 tests) asserts every signer-gated instruction rejects the wrong signer.

## 3. Ownership & Account-Type Validation

Anchor's typed `Account<'info, T>` wrapper deserializes and checks the account discriminator on every account in every instruction, so a caller cannot substitute an account of the wrong type (e.g. passing a `Milestone` where a `Gig` is expected fails at the framework level before the handler body runs).

## 4. PDA Validation & Anti-Spoofing

Full design rationale in [ARCHITECTURE.md § PDA Architecture](./ARCHITECTURE.md#8-pda-architecture). Security-relevant guarantees:

- Every PDA account is constrained with `seeds = [...], bump` (on creation) or `seeds = [...], bump = stored_bump` (on reuse), forcing the runtime to re-derive and match the exact expected address.
- Every PDA is additionally cross-checked against its logical parent: `milestone.gig == gig.key()`, `vault` seeded from `gig.key()`, `vault_token_account` checked via `address = vault.token_account`.
- **PDA spoofing protection**: an attacker cannot pass an account they control and claim it is "the vault" or "the milestone" for a given gig — the derived address would not match, and Anchor's constraint check fails the transaction before any state mutation or token transfer occurs.
- **Vault ownership guarantees**: the vault token account's SPL `authority` is set to the `EscrowVault` PDA at creation (`token::authority = vault`) and never reassigned. Because that PDA has no private key, only this program (via `invoke_signed` with the correct seeds) can ever authorize a debit.

Verified by `tests/pda_security.rs` (8 tests): wrong gig PDA, wrong milestone PDA, wrong vault PDA, wrong bump, milestone-from-a-different-gig, vault/token-account mismatch, cross-gig vault substitution in `approve_milestone`, and spoofed-but-uninitialized PDAs are all rejected.

## 5. Reinitialization & Replay Protection

- `Gig` and `Milestone` accounts use `init` (not `init_if_needed`) — Anchor's `init` fails if the account already exists, so a gig or milestone address can never be reinitialized once created, and the one-time `id`/`index`-derived seed guarantees no two gigs/milestones ever share an address.
- `EscrowVault` and the vault token account use `init_if_needed` deliberately, because multiple milestones under the same gig legitimately fund into the *same* vault. Reinitialization is not a vulnerability here because `init_if_needed` is idempotent at the account level (Anchor skips re-running `init` logic if the account is already initialized) and the handler additionally re-validates `vault.mint` on every subsequent fund (`require_keys_eq!(vault.mint, mint.key())`) so a second funding call cannot silently swap the vault's mint.
- Each milestone's `status` state machine (below) means a given milestone cannot be funded, submitted, or released twice — every transition consumes the prior state.

## 6. State Transition Validation

`MilestoneStatus` only moves forward: `PendingFunding → Funded → Submitted → {PartialReleased → Completed | Completed}`. Every handler asserts the exact required starting status before mutating:

| Instruction | Required starting status | Error on mismatch |
|---|---|---|
| `fund_milestone` | `PendingFunding` | `AlreadyFunded` |
| `submit_delivery` | `Funded` (and not already `Submitted`) | `InvalidStatus` / `MilestoneAlreadySubmitted` |
| `approve_milestone` | `Submitted` | `InvalidStatus` |
| `partial_timeout_release` | `Submitted` | `InvalidStatus` |
| `full_timeout_release` | `PartialReleased` | `InvalidStatus` |
| `cancel_before_funding` | `PendingFunding` | `AlreadyFunded` |

This ordering also enforces the intended timeout sequencing: `full_timeout_release` cannot fire before `partial_timeout_release` has already moved the milestone to `PartialReleased`, since that's its required precondition. `tests/state_transitions.rs` (18 tests) exhaustively exercises every valid and invalid transition.

`GigStatus` (`Active → Completed | Cancelled`) is likewise checked — `create_milestone` requires `gig.status == Active`, preventing new milestones on a cancelled or already-completed gig.

## 7. Checked Arithmetic — Overflow & Underflow Protection

All balance/counter math routes through `programs/escrow/src/utils.rs`, never raw `+`/`-`:

- `checked_add(a, b)` → `EscrowError::Overflow` on overflow.
- `checked_sub(a, b)` → `EscrowError::MathError` on underflow.
- `percent_of(amount, percent)` promotes to `u128` before multiplying, so `amount * percent` cannot overflow `u64` even at `amount = u64::MAX`, then checks the `u128 → u64` downcast explicitly.

Every counter that money flows through — `Gig.milestone_count`/`active_milestone`, `EscrowVault.total_locked`/`total_released`, `Milestone.released` — is updated exclusively through these helpers. The release path always computes the remaining payable amount as `checked_sub(milestone.amount, milestone.released)` and requires it to be `> 0` (`InsufficientFunds`), so a milestone can never pay out more than `milestone.amount` in total even across a partial + full release pair. `tests/arithmetic.rs` (7 tests, plus 4 unit tests in `utils.rs`) covers overflow, underflow, and percentage-split edge cases including `u64::MAX`.

## 8. Token Mint Validation

Every account that could conceivably carry the wrong asset is pinned to the gig's canonical mint:

- `Gig.mint` is fixed at `initialize_gig` and never mutated afterward.
- `fund_milestone` requires `gig.mint == mint` (`has_one`) and `client_token_account.mint == mint`.
- `EscrowVault.mint` is fixed on first funding and re-checked (`require_keys_eq!`) on every subsequent funding call, so a vault cannot be "topped up" with a different mint.
- `approve_milestone` / `partial_timeout_release` / `full_timeout_release` all require `vault.mint == mint` (`has_one`) and `freelancer_token_account.mint == mint`.
- All transfers use `transfer_checked`, which independently validates the mint and decimals passed match the token accounts at the SPL Token program level — a second, protocol-level check beyond Anchor's own constraints.

`tests/token_validation.rs` (11 tests) covers wrong-mint funding, wrong-mint release destinations, and mint-substitution attempts.

## 9. Vault Accounting Invariants

`EscrowVault.total_locked` and `total_released` are maintained as running counters alongside the actual SPL token balance, giving two independent sources of truth that must never diverge:

- `total_locked` only increases, only in `fund_milestone`, only by the exact amount transferred in.
- `total_released` only increases, only in the three release instructions, only by the exact amount transferred out.
- Per-milestone `released` is the authoritative cap: `remaining = milestone.amount - milestone.released` bounds every release, so cumulative payout across a `partial_timeout_release` followed by a `full_timeout_release` can never exceed `milestone.amount`.

`tests/vault_accounting.rs` (8 tests) asserts vault counters match actual on-chain token balances across funding, partial release, full release, and multi-milestone scenarios.

## 10. Double-Spend Prevention

Double-spending is prevented by the composition of §6 (state transitions) and §9 (per-milestone `released` cap): once a milestone reaches `Completed`, no further release instruction accepts it (all three release instructions require a specific non-`Completed` starting status), and even within the timeout sequence the `released` field ensures a second release only ever pays the *remaining* balance, never the full amount again.

## 11. Permission Validation

- Fund-moving actions requiring a specific party's consent (`fund_milestone`, `approve_milestone`, `cancel_before_funding`) require that exact party's signature, checked against the `Gig`'s stored `client`/`freelancer`, not merely "some signer."
- `approve_milestone`'s destination account is constrained to `address = gig.freelancer` — the client cannot redirect an approval payout to an arbitrary wallet.
- Timeout releases resolve the destination the same way (`freelancer_token_account.owner == gig.freelancer`), so even though the *caller* is permissionless, the *recipient* is not — a third party triggering a timeout release cannot redirect funds to themselves.

## 12. Timeout Security

`partial_timeout_release` (≥ 72h since `submitted_at`) and `full_timeout_release` (≥ 7 days since `submitted_at`) are deliberately callable by anyone, with no signer-identity check beyond the transaction fee payer. This is intentional: the entire purpose of the timeout mechanism is to guarantee a freelancer is paid even if the client disappears *and* the freelancer's own wallet is temporarily unable to submit a transaction (e.g. relies on a relayer/automation service). Because the recipient is hard-pinned to `gig.freelancer` (§11) and the amount is hard-pinned to the fixed percentage/remaining-balance formula (§7, §9), permissionless calling cannot be leveraged to misdirect or inflate a payout — the worst a third party can do is trigger a release that was already going to happen, slightly early is impossible (both instructions `require!(now >= ...)`) but never late-blocked, since anyone can call once the window opens. `tests/timeout_boundaries.rs` (8 tests) checks the exact boundary (`submitted_at + timeout - 1` rejected, `submitted_at + timeout` accepted) for both windows.

## 13. CPI Safety

The program makes exactly one class of outbound CPI: `anchor_spl::token::transfer_checked` into the SPL Token program, always with an explicit, hardcoded `token_program` account typed as `Program<'info, Token>` (Anchor validates this is the genuine SPL Token program, not an attacker-supplied lookalike). Outbound vault transfers are signed via `CpiContext::new_with_signer` using seeds derived from the account's own stored `bump` (see [ARCHITECTURE.md § 8.3](./ARCHITECTURE.md#83-bump-seeds-and-program-signing)), never a caller-supplied bump. The program never CPIs into an arbitrary/caller-specified program ID, eliminating an entire class of CPI-confusion attacks.

## 14. Summary of Enforced Invariants

1. A milestone can be funded exactly once.
2. A milestone can be submitted exactly once, and only after funding.
3. A milestone's cumulative `released` can never exceed its `amount`.
4. Only `gig.client` can approve, fund, or cancel; only `gig.freelancer` can submit delivery.
5. Release funds can only ever land in `gig.freelancer`'s token account.
6. Only the mint fixed at gig creation is ever accepted into or paid out of the vault.
7. Vault funds can only move via a CPI signed by the `EscrowVault` PDA's own seeds.
8. Timeout releases cannot fire before their exact deadline, but are permissionless once eligible.
9. All arithmetic on balances/counters is checked; overflow/underflow abort the transaction.
10. Every PDA used by any instruction is re-derived and validated against its logical parent, blocking substitution/spoofing.

---

# Reputation Program — Security

**Audit status: Complete.** The Reputation program (`programs/reputation`) has completed implementation, a full internal security audit, and a 145-test regression/security suite (`cargo test -p reputation`, `programs/reputation/tests/`) covering every invariant documented below. No open findings.

## 15. Threat Model

Actors:

- **Profile authority** — the user a `UserProfile` belongs to. Signs `initialize_profile` only. Trusted to sign only their own transactions.
- **Client** — signs `submit_rating` for a job they claim to have commissioned. Trusted to sign only their own transactions; **not** trusted to submit an honest score, or to necessarily be the real client of the referenced job (§15.4, Trust Assumptions).
- **`REPUTATION_AUTHORITY`** — a single hardcoded pubkey, the only signer accepted for `update_completion` and `award_badge`. Fully trusted for the correctness of completion/earnings data and badge issuance in the current (pre-CPI) design; see §18 in ARCHITECTURE.md.
- **Adversarial transaction builder** — may supply arbitrary accounts to any instruction, including the right account *type* at the wrong *instance* (e.g. someone else's profile), or uninitialized/attacker-owned accounts, attempting to spoof a PDA.

Assets at risk: the integrity of on-chain reputation data (scores, ratings, badges). The program holds no funds, so there is no direct custody risk; the risk is data integrity and manipulation of a signal other systems (including, eventually, Escrow) may rely on.

### 15.4 Trust Assumptions (explicitly out of this program's control)

Documented rather than hidden, because an accurate security posture requires naming what is *not* enforced on-chain:

- **Job identity is caller-supplied.** `submit_rating` takes `client` (the signer) and `freelancer` (an unchecked account) directly as instruction inputs; the program does not verify against Escrow that `job_id` corresponds to a real, completed `Gig` between those two parties. Anyone who can sign a transaction and knows a `job_id` that hasn't been rated yet can submit a rating for any `freelancer` account of their choosing. This is a consequence of Reputation being deliberately decoupled from Escrow today (ARCHITECTURE.md §3, §18) and is the responsibility of the caller (or, once CPI wiring exists, of Escrow) to constrain.
- **`REPUTATION_AUTHORITY` is a single centralized signer** for `update_completion` and `award_badge`. Its private key is a trust bottleneck: whoever holds it can record arbitrary completions/earnings and award `TrustedFreelancer`/`FastDeliverer` badges (which have no on-chain eligibility check — see §21). This is an accepted, documented MVP trade-off (ARCHITECTURE.md §18), not an oversight; the account-constraint shape was chosen specifically so it can be replaced by a CPI-only check from Escrow without an account-layout migration.

## 16. Signer Validation

- `initialize_profile` — `authority` must sign; the PDA is derived from that same signer's key, so a profile can only ever be created "as" its own authority.
- `submit_rating` — `client` must sign; `require_keys_neq!(client, freelancer)` prevents a client from rating their own freelancer profile (self-dealing).
- `update_completion` / `award_badge` — the signer is constrained with `#[account(address = REPUTATION_AUTHORITY @ ReputationError::Unauthorized)]`, i.e. only the one hardcoded authority pubkey is ever accepted, not merely "some signer."
- `get_profile` — read-only, no signer required.

`tests/profile_authorization.rs` (20 tests) and `tests/regressions.rs` (10 tests, including `test_authority_check_not_bypassed` and `test_self_dealing_check_not_bypassed`) assert every signer-gated instruction rejects the wrong signer.

## 17. Ownership & Account-Type Validation

As with Escrow (§3), Anchor's typed `Account<'info, T>` wrapper checks the account discriminator on every typed account, so a `Rating` cannot be substituted where a `UserProfile` is expected, and vice versa — the framework rejects the wrong account type before the handler body runs.

## 18. PDA Validation & Anti-Spoofing

Full design rationale in [ARCHITECTURE.md § 14](./ARCHITECTURE.md#14-pda-architecture). Security-relevant guarantees:

- Every PDA is constrained with `seeds = [...], bump` (on `init`) or `seeds = [...], bump = stored_bump` (on reuse), forcing re-derivation and an exact address match.
- `freelancer_profile` in `submit_rating` and `profile` in `update_completion`/`award_badge`/`get_profile` are all re-derived from `[PROFILE_SEED, authority.as_ref()]` — a caller cannot pass a different authority's profile and have it accepted as "the" profile for a given authority.
- `Badge` PDAs are seeded by `[BADGE_SEED, profile.authority, badge_type]`, so a spoofed badge account for the wrong profile or wrong type fails derivation.

Verified by `tests/pda_security.rs` (24 tests): wrong profile PDA, wrong rating PDA, wrong badge PDA, wrong bump, profile-for-a-different-authority, cross-profile badge substitution, and uninitialized spoofed PDAs are all rejected.

## 19. Reinitialization & Replay Protection

- `UserProfile`, `Rating`, and `Badge` all use Anchor `init` (never `init_if_needed`), so none of them can ever be reinitialized once created at their canonical address.
- `initialize_profile` therefore cannot be called twice for the same authority — the second call fails at account-init time with no separate existence check needed.
- `submit_rating`'s `Rating` PDA is seeded by `job_id` alone, so a second `submit_rating` for the same `job_id` — a replay or duplicate-rating attempt — fails at `init` time. This is the program's entire duplicate-rating defense, and it is structural (seed collision) rather than a runtime `require!` check that could be forgotten on a future code path.
- `award_badge`'s `Badge` PDA is seeded by `(profile, badge_type)`, so a second award of the same badge type to the same profile fails at `init` time — duplicate-badge prevention, structurally enforced the same way.

`tests/regressions.rs` includes `test_duplicate_prevention_not_bypassed`; `tests/rating_submission.rs` (34 tests) and `tests/badge_system.rs` (36 tests) each dedicate cases to the duplicate-`job_id`/duplicate-`badge_type` paths specifically.

## 20. Checked Arithmetic — Overflow & Underflow Protection

All counter/score math routes through `programs/reputation/src/utils.rs`'s `checked_add`/`checked_sub`/`checked_mul`/`checked_div` helpers, never raw operators:

- `completed_jobs`, `successful_jobs`, `cancelled_jobs`, `total_earnings`, `rating_sum`, `rating_count`, `badges_earned` are all updated exclusively via `checked_add`, returning `ReputationError::MathOverflow` on overflow.
- `average_rating` promotes through `checked_mul(rating_sum, RATING_SCALE)` then `checked_div(_, rating_count)` before a final `u32::try_from` bounds check — an overflow at any step aborts rather than wrapping or truncating silently.
- `compute_reputation_score` (§21) uses `checked_mul`/`checked_add` for every weighted term and `saturating_sub` (not raw subtraction) for the cancellation penalty, explicitly to avoid underflow when the penalty term exceeds the weighted sum — the result is clamped to `0` rather than wrapping to a near-`u64::MAX` value.

`tests/math.rs` (14 tests) and `tests/reputation_algorithm.rs` (22 tests, including `test_score_does_not_overflow_with_extreme_values`) cover boundary values including near-`u64::MAX` inputs; `src/utils.rs` also carries inline `#[cfg(test)]` unit tests (`checked_add_overflows`, `checked_sub_underflows`) exercising the helpers directly.

## 21. Reputation Score & Rating Integrity

### 21.1 Immutable Ratings

`Rating` has no update or delete instruction. Once `submit_rating` succeeds, `score`, `review_hash`, `client`, `freelancer`, and `submitted_at` are permanent. `tests/state_invariants.rs` includes `test_ratings_immutable`.

### 21.2 Immutable Profile Authority

`UserProfile.authority` is set once at `init` and never written by any other instruction. `tests/state_invariants.rs` includes `test_profile_authority_immutable`.

### 21.3 Rating Validation

`submit_rating` requires `(MIN_RATING..=MAX_RATING).contains(&score)` i.e. `1..=5`, rejecting `0` and anything `> 5` with `ReputationError::InvalidRating`. `tests/rating_validation.rs` (18 tests) and `tests/regressions.rs::test_range_check_not_bypassed` cover the boundary.

### 21.4 Authority Validation for Privileged Actions

`update_completion` and `award_badge` both require the signer to equal `REPUTATION_AUTHORITY` exactly (§16, §15.4). This is the program's central trust assumption today and is disclosed, not hidden, in ARCHITECTURE.md §18.

### 21.5 Deterministic Reputation Calculation

`compute_reputation_score` is a pure function of `UserProfile`'s own stored fields (`completed_jobs`, `successful_jobs`, `total_earnings`, `average_rating`, `cancelled_jobs`) — a weighted sum of four capped components (success rate, average rating, completed-job volume, lifetime earnings) minus a cancellation penalty, clamped to `[0, MAX_REPUTATION_SCORE]`. No randomness, no external oracle, no off-chain input: the same stored fields always produce the same score, and any observer can recompute and verify it independently from public account data. `tests/reputation_algorithm.rs` (22 tests) includes `test_score_is_deterministic` and `test_score_equal_for_identical_inputs`.

### 21.6 Badge Eligibility

`is_eligible_for_badge` deterministically checks five of the seven badge types against on-chain profile fields (`FirstGig`, `TenCompletedJobs`, `HundredCompletedJobs`, `FiveStarPerformer`, `TopRated`); `TrustedFreelancer` and `FastDeliverer` return `true` unconditionally and rely entirely on `REPUTATION_AUTHORITY`'s judgment plus the structural one-per-type duplicate guard (§19). This split is documented in source (`utils.rs`) and in ARCHITECTURE.md §19 rather than presented as a uniform on-chain guarantee. `tests/badge_system.rs` (36 tests) covers eligibility for every badge type, including the two authority-attested ones.

### 21.7 Metadata Bounds

`award_badge` requires `metadata.len() <= Badge::MAX_METADATA_LEN` (128 bytes), rejecting oversized metadata with `ReputationError::MetadataTooLong` before any account write.

## 22. Event Correctness

Every emitting instruction's event (`ProfileCreated`, `RatingSubmitted`, `CompletionUpdated`, `BadgeAwarded`) is asserted field-for-field against the instruction's actual resulting account state in `tests/events.rs` (20 tests). `ProfileUpdated` is defined but not currently emitted by any instruction — see ARCHITECTURE.md §17 — and is called out here so it is not mistaken for a monitored, silently-broken event path.

## 23. State Consistency

`tests/state_invariants.rs` (20 tests) directly asserts the invariants a reputation record must never violate: `completed_jobs >= successful_jobs`, `total_earnings` never decreases, `updated_at` is monotonically non-decreasing, `created_at` never changes, `average_rating` stays within `[0, 500]`, badges are unique per type, and `reputation_score` is reproducible from stored fields after an arbitrary sequence of operations (`test_all_invariants_hold_after_multiple_operations`).

## 24. Error Handling

`ReputationError` defines 11 variants. `ProfileAlreadyExists`, `ProfileNotFound`, and `InvalidEarnings` are defined but not currently returned by any instruction — duplicate-profile and duplicate-job protection are enforced structurally via PDA `init` (§19) rather than via an explicit existence check, and no instruction currently decreases `total_earnings`. Documented here rather than left as unexplained dead code; each is a reserved slot for a future explicit check rather than a broken current one.

## 25. Regression Coverage

`tests/regressions.rs` (10 tests) specifically re-asserts, as a group, that no previously-fixed or previously-verified check has been silently bypassed: range validation, authority validation, duplicate-prevention, self-dealing prevention, and eligibility validation are each re-checked directly rather than only incidentally through happy-path tests.

## 26. Summary of Enforced Invariants (Reputation)

1. A profile can be created exactly once per authority.
2. A job can be rated exactly once, ever, regardless of which client submits it.
3. A badge type can be awarded to a given profile exactly once.
4. Only `REPUTATION_AUTHORITY` can record completions or award badges.
5. A client cannot rate a job where they are also the freelancer.
6. Ratings are immutable once submitted.
7. A profile's `authority` field never changes after creation.
8. `total_earnings`, `completed_jobs`, `successful_jobs`, `cancelled_jobs`, `rating_count`, `badges_earned` only ever increase.
9. `reputation_score` is always a pure, deterministic, independently-verifiable function of the profile's own stored fields.
10. All arithmetic on counters/scores is checked or explicitly saturating; overflow aborts the transaction, and the score is clamped rather than allowed to wrap.
11. Every PDA used by any instruction is re-derived and validated against its logical parent (authority, profile, or badge type), blocking substitution/spoofing.
12. `REPUTATION_AUTHORITY` centralization and caller-supplied job identity are explicit, documented trust assumptions, not silently-assumed guarantees (§15.4).
