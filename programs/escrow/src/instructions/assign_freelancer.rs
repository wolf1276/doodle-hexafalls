use anchor_lang::prelude::*;

use crate::errors::EscrowError;
use crate::events::FreelancerAssigned;
use crate::state::{Gig, GigStatus};

#[derive(Accounts)]
pub struct AssignFreelancer<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    /// CHECK: freelancer is only stored as a pubkey reference, never signs here.
    pub freelancer: UncheckedAccount<'info>,

    #[account(
        mut,
        has_one = client @ EscrowError::Unauthorized,
        constraint = gig.status == GigStatus::Published @ EscrowError::NotPublishedStatus,
    )]
    pub gig: Account<'info, Gig>,
}

pub fn handler(ctx: Context<AssignFreelancer>) -> Result<()> {
    require_keys_neq!(
        ctx.accounts.client.key(),
        ctx.accounts.freelancer.key(),
        EscrowError::Unauthorized
    );
    require!(
        ctx.accounts.gig.freelancer == Pubkey::default(),
        EscrowError::FreelancerAlreadyAssigned
    );

    let gig = &mut ctx.accounts.gig;
    gig.freelancer = ctx.accounts.freelancer.key();
    gig.status = GigStatus::Assigned;
    gig.updated_at = Clock::get()?.unix_timestamp;

    emit!(FreelancerAssigned {
        gig: gig.key(),
        id: gig.id,
        freelancer: gig.freelancer,
    });

    Ok(())
}
