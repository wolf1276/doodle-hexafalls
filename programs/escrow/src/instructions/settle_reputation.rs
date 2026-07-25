use anchor_lang::prelude::*;
use reputation::cpi::accounts::UpdateCompletion;
use reputation::UserProfile;

use crate::constants::{ESCROW_AUTHORITY_SEED, VAULT_SEED};
use crate::errors::EscrowError;
use crate::state::EscrowVault;
use gig::Gig;

#[derive(Accounts)]
pub struct SettleReputation<'info> {
    pub gig: Account<'info, Gig>,

    #[account(
        mut,
        seeds = [VAULT_SEED, gig.key().as_ref()],
        bump = vault.bump,
        constraint = vault.gig == gig.key() @ EscrowError::Unauthorized,
        constraint = vault.milestone_count > 0 && vault.active_milestone >= vault.milestone_count @ EscrowError::InvalidStatus,
        constraint = !vault.reputation_synced @ EscrowError::InvalidStatus,
    )]
    pub vault: Account<'info, EscrowVault>,

    /// CHECK: PDA identity only; used purely as escrow's CPI-signer into the reputation program.
    #[account(seeds = [ESCROW_AUTHORITY_SEED], bump)]
    pub escrow_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [reputation::PROFILE_SEED, gig.freelancer.as_ref()],
        bump = freelancer_profile.bump,
        seeds::program = reputation::ID,
    )]
    pub freelancer_profile: Account<'info, UserProfile>,

    pub reputation_program: Program<'info, reputation::program::Reputation>,
}

/// Permissionlessly notifies the Reputation Program once every milestone in
/// this gig has been released -- escrow's own state is the sole trigger, and
/// `vault.reputation_synced` ensures this CPI (and its earnings credit) can
/// only ever fire once per gig.
pub fn handler(ctx: Context<SettleReputation>) -> Result<()> {
    let authority_bump = ctx.bumps.escrow_authority;
    let authority_signer_seeds: &[&[&[u8]]] = &[&[ESCROW_AUTHORITY_SEED, &[authority_bump]]];

    let earnings = ctx.accounts.vault.total_released;

    reputation::cpi::update_completion(
        CpiContext::new_with_signer(
            ctx.accounts.reputation_program.key(),
            UpdateCompletion {
                escrow_authority: ctx.accounts.escrow_authority.to_account_info(),
                profile: ctx.accounts.freelancer_profile.to_account_info(),
            },
            authority_signer_seeds,
        ),
        true,
        earnings,
    )?;

    ctx.accounts.vault.reputation_synced = true;

    Ok(())
}
