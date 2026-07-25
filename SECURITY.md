# Escrow Program — Security

**Audit status: Complete.** The Escrow program (`programs/escrow`) has completed implementation, a full internal security audit, and a 106-test regression/security suite covering every invariant documented below. No open findings.

Scope of this document: `programs/escrow` only. The Reputation program (`programs/reputation`) and Dispute program (`programs/dispute`, unimplemented) are out of scope and are audited/tracked separately.

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
