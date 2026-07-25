use anchor_lang::prelude::*;

use crate::constants::{PROFILE_SEED, REPUTATION_AUTHORITY};
use crate::errors::ReputationError;
use crate::events::CompletionUpdated;
use crate::state::UserProfile;
use crate::utils::{checked_add, compute_reputation_score};

#[derive(Accounts)]
pub struct UpdateCompletion<'info> {
    /// Trusted caller recording the outcome of a completed job. Hardcoded
    /// to `REPUTATION_AUTHORITY` for the MVP; the Escrow Program will
    /// become this signer via CPI once milestone completions call into
    /// this instruction directly, with no change to the account layout.
    #[account(address = REPUTATION_AUTHORITY @ ReputationError::Unauthorized)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [PROFILE_SEED, profile.authority.as_ref()],
        bump = profile.bump,
    )]
    pub profile: Account<'info, UserProfile>,
}

/// Records the outcome of a completed job: increments `completed_jobs`
/// and either `successful_jobs` or `cancelled_jobs`, adds any earnings,
/// and recomputes the deterministic reputation score.
pub fn handler(
    ctx: Context<UpdateCompletion>,
    successful: bool,
    earnings: u64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let profile = &mut ctx.accounts.profile;

    profile.completed_jobs = checked_add(profile.completed_jobs, 1)?;
    if successful {
        profile.successful_jobs = checked_add(profile.successful_jobs, 1)?;
        profile.total_earnings = checked_add(profile.total_earnings, earnings)?;
    } else {
        profile.cancelled_jobs = checked_add(profile.cancelled_jobs, 1)?;
    }

    profile.reputation_score = compute_reputation_score(profile)?;
    profile.updated_at = now;

    emit!(CompletionUpdated {
        profile: profile.key(),
        completed_jobs: profile.completed_jobs,
        successful_jobs: profile.successful_jobs,
        total_earnings: profile.total_earnings,
        reputation_score: profile.reputation_score,
    });

    Ok(())
}
