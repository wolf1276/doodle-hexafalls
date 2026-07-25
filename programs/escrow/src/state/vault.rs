use anchor_lang::prelude::*;

#[account]
pub struct EscrowVault {
    pub gig: Pubkey,
    pub token_account: Pubkey,
    pub mint: Pubkey,
    pub total_locked: u64,
    pub total_released: u64,
    pub bump: u8,
}

impl EscrowVault {
    pub const INIT_SPACE: usize = 8 // discriminator
        + 32 // gig
        + 32 // token_account
        + 32 // mint
        + 8 // total_locked
        + 8 // total_released
        + 1; // bump
}
