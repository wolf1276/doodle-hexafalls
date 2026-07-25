pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use errors::*;
pub use events::*;
pub use instructions::*;
pub use reputation::BadgeType;
pub use state::*;

declare_id!("GV8Z39NBK7qrojXCfnnwLTXpqsLoCW6sy9cLHGYjtrv9");

#[program]
pub mod achievement {
    use super::*;

    /// One-time admin setup: creates the shared NFT collection every
    /// achievement is minted into. Settlement never touches this program --
    /// claiming is always a separate, user-initiated transaction.
    pub fn init_collection(ctx: Context<InitCollection>, name: String, uri: String) -> Result<()> {
        init_collection::handler(ctx, name, uri)
    }

    /// User-initiated: mints the NFT for a badge already earned in the
    /// Reputation program. Reputation is never queried or modified by this
    /// call beyond reading the existing profile/badge PDAs.
    pub fn claim_achievement(ctx: Context<ClaimAchievement>, badge_type: BadgeType) -> Result<()> {
        claim_achievement::handler(ctx, badge_type)
    }
}
