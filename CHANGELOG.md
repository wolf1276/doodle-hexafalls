# Changelog

All notable changes to the Escrow program and its documentation are recorded here.

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
