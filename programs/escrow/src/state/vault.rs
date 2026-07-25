use anchor_lang::prelude::*;

#[account]
pub struct EscrowVault {
    pub gig: Pubkey,
    pub token_account: Pubkey,
    pub mint: Pubkey,
    pub total_locked: u64,
    pub total_released: u64,
    /// Milestones created for this gig. Escrow-owned: this is payment-lifecycle
    /// bookkeeping, not gig metadata, so it lives here rather than on the Gig account.
    pub milestone_count: u32,
    /// Milestones fully released so far.
    pub active_milestone: u32,
    /// Set once `settle_reputation` has notified the Reputation Program of this
    /// gig's completion, so the CPI (and its earnings credit) can never fire twice.
    pub reputation_synced: bool,
    pub bump: u8,
}

impl EscrowVault {
    pub const INIT_SPACE: usize = 8 // discriminator
        + 32 // gig
        + 32 // token_account
        + 32 // mint
        + 8 // total_locked
        + 8 // total_released
        + 4 // milestone_count
        + 4 // active_milestone
        + 1 // reputation_synced
        + 1; // bump
}
