use anchor_lang::prelude::*;

use super::enums::GigStatus;

#[account]
pub struct Gig {
    pub id: u64,
    pub client: Pubkey,
    pub freelancer: Pubkey,
    pub mint: Pubkey,
    pub milestone_count: u32,
    pub active_milestone: u32,
    pub status: GigStatus,
    pub created_at: i64,
    pub bump: u8,
}

impl Gig {
    pub const INIT_SPACE: usize = 8 // discriminator
        + 8 // id
        + 32 // client
        + 32 // freelancer
        + 32 // mint
        + 4 // milestone_count
        + 4 // active_milestone
        + 1 // status
        + 8 // created_at
        + 1; // bump
}
