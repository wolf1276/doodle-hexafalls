# Escrow & Reputation Programs — Test Suite

**106 tests across 11 modules**, run against a `litesvm` in-process Solana runtime (no local validator required). Run with:

```bash
cargo test -p escrow
```

Module breakdown (`programs/escrow/tests/*.rs`, plus inline unit tests in `src/utils.rs`):

| Module | Tests | Validates |
|---|---|---|
| `happy_path.rs` | 10 | End-to-end success paths: gig → milestone → fund → submit → approve, across single and multi-milestone gigs, confirming every account's final state and every SPL balance is exactly as expected. |
| `authorization.rs` | 10 | Every signer-gated instruction rejects the wrong signer — wrong client, wrong freelancer, unrelated third-party keys attempting client/freelancer actions. |
| `state_transitions.rs` | 18 | Every `MilestoneStatus`/`GigStatus` edge: valid forward transitions succeed, every out-of-order or repeated transition (double-fund, double-submit, approve-before-submit, release-after-complete, etc.) is rejected with the correct error. |
| `timeout_boundaries.rs` | 8 | Exact boundary behavior of the 72-hour partial and 7-day full timeout windows — one second before the deadline rejects (`TimeoutNotReached`), exactly at/after the deadline succeeds; also confirms `full_timeout_release` cannot fire before `partial_timeout_release` has run. |
| `arithmetic.rs` | 7 (+ 4 unit tests in `src/utils.rs`) | Checked-arithmetic helpers (`checked_add`, `checked_sub`, `percent_of`) at boundary values including `u64::MAX`, confirming overflow/underflow abort rather than wrap. |
| `vault_accounting.rs` | 8 | `EscrowVault.total_locked`/`total_released` counters stay consistent with actual on-chain SPL token balances across funding, partial release, full release, and multi-milestone/multi-vault scenarios. |
| `pda_security.rs` | 8 | PDA spoofing resistance — wrong gig/milestone/vault PDA, wrong bump, milestone from a different gig, vault/token-account mismatch, cross-gig vault substitution, and uninitialized spoofed PDAs are all rejected. |
| `token_validation.rs` | 11 | Mint pinning — funding or release attempted with a token account of the wrong mint, or a vault/gig mint mismatch, is rejected at every account boundary that touches a token account. |
| `events.rs` | 10 | Every instruction emits its documented event (`GigCreated`, `MilestoneCreated`, `MilestoneFunded`, `DeliverySubmitted`, `MilestoneApproved`, `PartialReleaseExecuted`, `FullReleaseExecuted`, `GigCancelled`) with fields matching the instruction's actual effects. |
| `escrow_flow.rs` | 12 | Regression suite covering combinations exercised during development — multi-milestone gigs, cancellation before funding, interleaved timeout/approval races, and other scenarios found during hardening. |
| `src/utils.rs` (`#[cfg(test)]`) | 4 | Unit-level checks of the checked-math helpers in isolation from any Anchor/litesvm context. |

## What Each Category Guarantees

- **Happy Path** — the program does what it's supposed to do when every party behaves correctly, for both single- and multi-milestone gigs.
- **Authorization** — no instruction can be executed by a party who isn't the required signer for that action.
- **State Transitions** — the milestone/gig state machines cannot be driven out of order, replayed, or skipped.
- **Timeout Logic** — the 72h/7d windows are enforced to the boundary in both directions (too early rejected, on-time accepted), and the two-stage sequencing (partial before full) is mandatory.
- **Arithmetic** — no balance or counter can overflow or underflow; percentage math is exact and doesn't lose precision at extreme values.
- **Vault Accounting** — the program's internal bookkeeping (`total_locked`/`total_released`) never drifts from the real SPL token balances it's tracking.
- **PDA Security** — every account address is validated against its expected derivation; nothing resembling account substitution or spoofing succeeds.
- **Token Validation** — only the mint fixed at gig creation can ever enter or leave the vault.
- **Events** — off-chain indexers can trust emitted events to be a complete, accurate log of on-chain state changes.
- **Regression Tests** — scenarios identified during development/hardening stay fixed as the codebase evolves.
- **Multi-milestone Flow** — gigs with more than one milestone correctly isolate per-milestone state while sharing a single vault, across both `happy_path.rs` and `escrow_flow.rs`.

## Test Infrastructure

Tests run against `litesvm`, an in-process, dependency-free implementation of the Solana runtime — no `solana-test-validator` process, no network I/O, fast enough to run the full suite in-process on every change. Shared setup (keypair generation, mint creation, gig/milestone bootstrapping helpers) lives in `tests/common/mod.rs` and is reused across all 11 test modules to keep each test file focused on the behavior it's validating rather than boilerplate.

---

# Reputation Program — Test Suite

**145 tests across 12 modules**, run against the same in-process `litesvm` runtime. Run with:

```bash
cargo test -p reputation
```

Module breakdown (`programs/reputation/tests/*.rs`, plus inline unit tests in `src/utils.rs`):

| Module | Tests | Validates |
|---|---|---|
| `profile_creation.rs` | 10 | `initialize_profile` happy path — profile fields initialized to zero/defaults, PDA derived correctly, `ProfileCreated` emitted. |
| `profile_authorization.rs` | 20 | Every signer-gated instruction rejects the wrong signer — wrong authority on `initialize_profile`, wrong `REPUTATION_AUTHORITY` on `update_completion`/`award_badge`, client/freelancer self-dealing on `submit_rating`. |
| `rating_submission.rs` | 34 | `submit_rating` end-to-end — rating account fields, running `rating_sum`/`rating_count`/`average_rating`/`reputation_score` updates, one rating per `job_id`. |
| `rating_validation.rs` | 18 | Score-range enforcement (`1..=5`), self-dealing rejection, and other input validation on `submit_rating`. |
| `badge_system.rs` | 36 | `award_badge` for all 7 `BadgeType` variants — deterministic eligibility (`FirstGig`, `*CompletedJobs`, `FiveStarPerformer`, `TopRated`), authority-attested types (`TrustedFreelancer`, `FastDeliverer`), duplicate-award rejection, metadata length bounds. |
| `completion_updates.rs` | 22 | `update_completion` — `completed_jobs`/`successful_jobs`/`cancelled_jobs`/`total_earnings` bookkeeping, reputation-score recomputation on every call, authority gating. |
| `reputation_algorithm.rs` | 22 | `compute_reputation_score` correctness — determinism, capping at `MAX_REPUTATION_SCORE`, cancellation penalties, extreme-value overflow resistance. |
| `math.rs` | 14 | `average_rating` and the `checked_*` arithmetic helpers at boundary values, including exact-mean computation across many ratings. |
| `pda_security.rs` | 24 | PDA spoofing resistance — wrong profile/rating/badge PDA, wrong bump, profile-for-a-different-authority, cross-profile badge substitution, uninitialized spoofed PDAs. |
| `events.rs` | 20 | Every emitting instruction's event (`ProfileCreated`, `RatingSubmitted`, `CompletionUpdated`, `BadgeAwarded`) matches the instruction's actual effects. |
| `state_invariants.rs` | 20 | Structural invariants hold across arbitrary operation sequences — monotonic counters, immutable `authority`/ratings, bounded `average_rating`, unique badges per type. |
| `regressions.rs` | 10 | Previously-verified checks stay enforced — range, authority, duplicate-prevention, self-dealing, and eligibility checks are re-asserted directly rather than only incidentally. |
| `src/utils.rs` (`#[cfg(test)]`) | inline | Unit-level checks of `checked_add`/`checked_sub`, `average_rating`, and `compute_reputation_score` in isolation from any Anchor/litesvm context. |

## What Each Category Guarantees

- **Profile Creation** — a profile is correctly initialized exactly once per authority, at the correct PDA, with all counters zeroed.
- **Authorization** — no instruction can be executed by a party who isn't the required signer, including the single hardcoded `REPUTATION_AUTHORITY` for privileged actions.
- **Rating Submission & Validation** — ratings are recorded exactly once per job, only for in-range scores, and correctly fold into the freelancer's running average.
- **Badge System** — every badge type's eligibility rule is enforced (or explicitly authority-attested where no on-chain signal exists), and no badge type can be awarded twice to the same profile.
- **Reputation Calculation** — `compute_reputation_score` is deterministic, bounded, and resistant to overflow/underflow at extreme inputs.
- **PDA Security** — every account address is validated against its expected derivation; nothing resembling account substitution or spoofing succeeds.
- **Arithmetic** — no counter or score can overflow, underflow, or wrap; the score is clamped rather than allowed to wrap negative.
- **Events** — off-chain indexers can trust emitted events to reflect the instruction's actual on-chain effects.
- **State Invariants** — the profile's data can never drift into an inconsistent state (decreasing counters, mutated authority, edited ratings) regardless of operation order.
- **Regression Tests** — checks verified during development/hardening stay fixed as the codebase evolves.

## Test Infrastructure

Reputation's tests share the same `litesvm` approach as Escrow's. Shared setup — keypair generation, profile/rating/badge bootstrapping helpers, and fixtures for common profile states — lives in `tests/common/mod.rs`, `tests/common/fixtures.rs`, and `tests/common/helpers.rs`, reused across all 12 test modules.

## Coverage & Production Readiness

`cargo test -p reputation --tests` runs all 145 tests with **0 failures**. Combined with the completed internal security audit ([SECURITY.md](./SECURITY.md)), the Reputation program's implementation, test suite, and audit are all complete — see [IMPLEMENTATION_PROGRESS.md](./IMPLEMENTATION_PROGRESS.md).
