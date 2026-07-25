use anchor_lang::prelude::*;

use crate::constants::{MAX_RATING, MIN_RATING, PROFILE_SEED, RATING_SEED, REPUTATION_AUTHORITY};
use crate::errors::ReputationError;
use crate::events::RatingSubmitted;
use crate::state::{Rating, UserProfile};
use crate::utils::{average_rating, checked_add, compute_reputation_score};

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct SubmitRating<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    /// Trusted co-signer attesting that `job_id` is a real completed job
    /// between `client` and `freelancer`. Hardcoded to `REPUTATION_AUTHORITY`
    /// for the MVP; the Escrow Program will become this signer via CPI once
    /// it can attest completions directly, with no change to account layout.
    #[account(address = REPUTATION_AUTHORITY @ ReputationError::Unauthorized)]
    pub authority: Signer<'info>,

    /// CHECK: only used as the seed/reference the freelancer profile is derived from.
    pub freelancer: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [PROFILE_SEED, freelancer.key().as_ref()],
        bump = freelancer_profile.bump,
    )]
    pub freelancer_profile: Account<'info, UserProfile>,

    #[account(
        init,
        payer = client,
        space = Rating::INIT_SPACE,
        seeds = [RATING_SEED, job_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub rating: Account<'info, Rating>,

    pub system_program: Program<'info, System>,
}

/// Records an immutable 1-5 rating for a completed job and folds it into
/// the freelancer's running average and reputation score. The `rating`
/// PDA is keyed solely by `job_id`, so a second call for the same job
/// fails at account-init time -- ratings can never be edited or replayed.
pub fn handler(
    ctx: Context<SubmitRating>,
    job_id: u64,
    score: u8,
    review_hash: [u8; 32],
) -> Result<()> {
    require!(
        (MIN_RATING..=MAX_RATING).contains(&score),
        ReputationError::InvalidRating
    );
    require_keys_neq!(
        ctx.accounts.client.key(),
        ctx.accounts.freelancer.key(),
        ReputationError::SelfDealing
    );

    let now = Clock::get()?.unix_timestamp;

    let rating = &mut ctx.accounts.rating;
    rating.job_id = job_id;
    rating.client = ctx.accounts.client.key();
    rating.freelancer = ctx.accounts.freelancer.key();
    rating.score = score;
    rating.review_hash = review_hash;
    rating.submitted_at = now;
    rating.bump = ctx.bumps.rating;

    let profile = &mut ctx.accounts.freelancer_profile;
    profile.rating_sum = checked_add(profile.rating_sum, score as u64)?;
    profile.rating_count = checked_add(profile.rating_count, 1)?;
    profile.average_rating = average_rating(profile.rating_sum, profile.rating_count)?;
    profile.reputation_score = compute_reputation_score(profile)?;
    profile.updated_at = now;

    emit!(RatingSubmitted {
        rating: rating.key(),
        job_id,
        client: rating.client,
        freelancer: rating.freelancer,
        score,
        new_average_rating: profile.average_rating,
    });

    Ok(())
}
