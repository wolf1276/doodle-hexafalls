use anchor_lang::prelude::*;

/// Singleton config holding the shared Metaplex Core collection that every
/// claimed achievement NFT is minted into.
#[account]
pub struct AchievementConfig {
    pub admin: Pubkey,
    pub collection: Pubkey,
    pub bump: u8,
}

impl AchievementConfig {
    pub const INIT_SPACE: usize = 8 // discriminator
        + 32 // admin
        + 32 // collection
        + 1; // bump
}
