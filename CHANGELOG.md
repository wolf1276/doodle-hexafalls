# Changelog

All notable changes to the Escrow and Reputation programs and their documentation are recorded here.

Entries below the current one are kept as written at the time of release. Where an older entry says "production-ready", read it as "implementation, tests, and internal audit complete" — neither program is deployed, and neither has had an external audit. Test counts in older entries predate later suite growth; see [TESTING.md](./TESTING.md) for current figures.

## [Unreleased — Escrow Gig Lifecycle]

### Added
- Six gig-lifecycle instructions on the Escrow program: `update_gig`, `publish_gig`, `assign_freelancer`, `complete_gig`, `archive_gig`, `cancel_gig` — bringing the escrow program to **14 instructions**. Gig state lives in the escrow program (the `programs/gig` directory is an empty scaffold, not a separate program) so a milestone release can advance `GigStatus` atomically in the same instruction.
- `GigStatus` expanded from `Active | Completed | Cancelled` to the full six-state machine: `Draft → Published → Assigned → Completed → Archived`, with `Cancelled` reachable from any of the first three. `create_milestone` now requires `Assigned`, so no gig can escrow funds before a freelancer is assigned.
- On-chain listing metadata on `Gig`: `title`, `description`, `skills`, `category`, `budget`, `deadline`, `mint`, `updated_at` — a gig is fully describable from chain state with no off-chain database. Length caps (`MAX_TITLE_LEN` 100, `MAX_DESCRIPTION_LEN` 500, `MAX_SKILLS_LEN` 200, `MAX_CATEGORY_LEN` 50) and `MIN_DEADLINE_SECS` (1 day) are enforced at every write.
- 5 new events (`GigUpdated`, `GigPublished`, `FreelancerAssigned`, `GigCompleted`, `GigArchived`), bringing Escrow to **13 events** — still one per state-changing instruction.
- 13 new `EscrowError` variants for gig status and input validation (`NotDraftStatus`, `NotPublishedStatus`, `NotAssignedStatus`, `NotCompletedStatus`, `TerminalStatus`, `FreelancerAlreadyAssigned`, `InvalidBudget`, `InvalidDeadline`, `TitleTooLong`, `DescriptionTooLong`, `SkillsTooLong`, `CategoryTooLong`, `MetadataTooLong`), bringing the total to **24**.
- 6 new Escrow test modules — `lifecycle.rs` (15), `gig_creation.rs` (11), `gig_updates.rs` (12), `validation.rs` (12), `freelancer_assignment.rs` (8), `publishing.rs` (6) — bringing Escrow to **180 integration tests across 16 modules** (184 including unit tests).

### Changed
- `Gig` PDA seeds are `[GIG_SEED, id]` — the client is **not** a seed. Gig ids therefore share one global namespace: the first client to create id *N* owns it, and a later `initialize_gig` with the same id fails at `init`. Integrators must allocate ids collision-aware.
- `initialize_gig` now creates a gig in `Draft` (previously immediately active) and takes `title`, `description`, `category`, `budget`, and `deadline`. It has no `skills` parameter; `skills` starts empty and can only be set through `update_gig`, which is `Draft`-only.
- `assign_freelancer` replaces passing the freelancer at gig creation. It enforces `client != freelancer` and rejects reassignment once set.

### Documentation
- Rewrote [README.md](./README.md) as the canonical repository entry point, verified line-by-line against program source: implementation matrix with per-component status, full instruction reference, PDA architecture, account model, threat model, invariant list, and an explicit statement that nothing is deployed and no external audit exists.
- Corrected test counts across all docs. Previously-published figures (106 Escrow / 145 Reputation) were inaccurate; the counted totals are **184 Escrow** and **144 Reputation** (**328** total, 0 failures).
- Updated [ARCHITECTURE.md](./ARCHITECTURE.md) §4.1, §5, §6, §8.2, §10 for the gig lifecycle, and [SECURITY.md](./SECURITY.md) §2 and §6 for gig-status and `assign_freelancer` authority rules. Corrected a prior claim that `initialize_gig` enforced `require_keys_neq!(client, freelancer)` — that check lives in `assign_freelancer`.
- Marked `docs/details.md` explicitly as product vision, not implementation, wherever it conflicts with program code.

### Status
- Both programs remain implemented, internally audited, and **undeployed**. No external audit has been performed.

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
