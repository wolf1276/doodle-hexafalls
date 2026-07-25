use anchor_lang::prelude::*;
use mpl_core::instructions::CreateCollectionV2CpiBuilder;

use crate::constants::CONFIG_SEED;
use crate::state::AchievementConfig;

#[derive(Accounts)]
pub struct InitCollection<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = AchievementConfig::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump,
    )]
    pub config: Account<'info, AchievementConfig>,

    /// CHECK: fresh account for the new Metaplex Core collection, created by the CPI below.
    #[account(mut)]
    pub collection: Signer<'info>,

    /// CHECK: verified against the hardcoded Metaplex Core program id.
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// One-time setup: creates the shared achievement collection with this
/// program's `config` PDA as its update authority, so only `claim_achievement`
/// (which signs as `config`) can ever mint assets into it.
pub fn handler(ctx: Context<InitCollection>, name: String, uri: String) -> Result<()> {
    CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .update_authority(Some(&ctx.accounts.config.to_account_info()))
        .payer(&ctx.accounts.admin.to_account_info())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .name(name)
        .uri(uri)
        .invoke()?;

    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.admin.key();
    config.collection = ctx.accounts.collection.key();
    config.bump = ctx.bumps.config;

    Ok(())
}
