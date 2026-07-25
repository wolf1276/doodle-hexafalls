# Changelog

All notable changes to the Escrow and Reputation programs and their documentation are recorded here.

## [Reputation Program — Production Release]

### Added
- Complete reputation architecture: `UserProfile`, `Rating`, `Badge` account model, one PDA per authority/job/badge-type.
- Full instruction set: `initialize_profile`, `submit_rating`, `update_completion`, `award_badge`, `get_profile`.
- PDA-based addressing and duplicate-prevention model — `Rating` seeded by `job_id` alone and `Badge` seeded by `(profile, badge_type)`, so duplicate ratings and duplicate badge awards fail structurally at account-init time.
- Deterministic, recomputable reputation scoring (`compute_reputation_score`) — a pure function of a profile's own stored counters, with no randomness or off-chain input.
- 4 typed Anchor events (`ProfileCreated`, `RatingSubmitted`, `CompletionUpdated`, `BadgeAwarded`) for off-chain indexing; a 5th (`ProfileUpdated`) is defined and reserved.
- 145-test `litesvm` regression and security suite across 12 modules: profile creation, authorization, rating submission/validation, badge system, completion updates, reputation algorithm, math, PDA security, events, state invariants, and regressions.

### Security
- Completed a full internal security audit covering signer validation, ownership validation, PDA validation and anti-spoofing, checked arithmetic (overflow/underflow protection, saturating score clamping), reinitialization/replay protection, duplicate-rating and duplicate-badge prevention, deterministic reputation-score computation, badge-eligibility rules, event correctness, and state consistency. No open findings. Two trust assumptions — the single hardcoded `REPUTATION_AUTHORITY` signer and caller-supplied job identity in `submit_rating` — are explicitly documented as accepted pre-CPI trade-offs rather than silent gaps. Full detail in [SECURITY.md § Reputation Program Security](./SECURITY.md#reputation-program--security).
- Completed a dedicated PDA audit: seed derivation, deterministic addressing, bump-seed handling, and anti-spoofing coverage for `UserProfile`, `Rating`, and `Badge` PDAs. Full detail in [ARCHITECTURE.md § 14](./ARCHITECTURE.md#14-pda-architecture).

### Documentation
- Extended [ARCHITECTURE.md](./ARCHITECTURE.md) with a full Reputation Program Architecture section: account model, PDA architecture, instruction flow, state transitions, event architecture, future CPI compatibility, and design rationale.
- Extended [SECURITY.md](./SECURITY.md) with a full Reputation Program Security section: threat model (including explicit trust assumptions), signer/PDA/arithmetic validation, duplicate prevention, deterministic scoring, and a summary of every enforced invariant.
- Extended [TESTING.md](./TESTING.md) with a breakdown of all 12 Reputation test modules and what each validates.
- Updated [IMPLEMENTATION_PROGRESS.md](./IMPLEMENTATION_PROGRESS.md) marking the Reputation program production-ready across architecture, instructions, PDA design, events, errors, tests, and security audit.
- Extended [README.md](./README.md) with a Reputation Program section.

### Status
- **Reputation program: production-ready.** Architecture, PDA design, instruction set, events, error handling, test suite, and security audit are all complete.

## [Escrow Program — Production Release]

### Added
- Complete escrow architecture: `Gig`, `Milestone`, `EscrowVault` account model with a single shared vault per gig.
- Full instruction set: `initialize_gig`, `create_milestone`, `fund_milestone`, `submit_delivery`, `approve_milestone`, `partial_timeout_release`, `full_timeout_release`, `cancel_before_funding`.
- PDA-based custody model — Gig, Milestone, Vault, and Vault Token Account are all Program Derived Addresses, with the vault token account's SPL authority assigned to the `EscrowVault` PDA (no private key, no external signer possible).
- SPL Token integration via `transfer_checked` for both client-funded deposits and PDA-signed releases, with mint validation enforced at every account boundary.
- 8 typed Anchor events (`GigCreated`, `MilestoneCreated`, `MilestoneFunded`, `DeliverySubmitted`, `MilestoneApproved`, `PartialReleaseExecuted`, `FullReleaseExecuted`, `GigCancelled`) for off-chain indexing.
- 106-test `litesvm` regression and security suite across 11 modules: happy path, authorization, state transitions, timeout boundaries, arithmetic, vault accounting, PDA security, token validation, events, and general regression coverage.

### Security
- Completed a full internal security audit covering signer validation, ownership validation, PDA validation and anti-spoofing, checked arithmetic (overflow/underflow protection), state-transition enforcement, mint validation, vault accounting invariants, double-spend prevention, reinitialization protection, permission validation, timeout security, and CPI safety. No open findings. Full detail in [SECURITY.md](./SECURITY.md).

### Documentation
- Added [ARCHITECTURE.md](./ARCHITECTURE.md): protocol overview, account architecture, instruction flow/state machine, event architecture, token flow, and a dedicated PDA architecture section.
- Added [SECURITY.md](./SECURITY.md): threat model and full audit writeup of every enforced invariant.
- Added [TESTING.md](./TESTING.md): breakdown of all 11 test modules and what each validates.
- Added [IMPLEMENTATION_PROGRESS.md](./IMPLEMENTATION_PROGRESS.md): completion status for Escrow, Reputation, and Dispute programs.
- Extended [README.md](./README.md) with an Escrow Program section and links to the above.

### Status
- **Escrow program: production-ready.** Architecture, PDA design, instruction set, SPL Token integration, events, error handling, test suite, and security audit are all complete.
