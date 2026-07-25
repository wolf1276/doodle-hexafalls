# Escrow Program — Test Suite

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
