use anchor_lang::prelude::*;

use crate::errors::EscrowError;
use crate::events::GigCancelled;
use crate::state::{Gig, GigStatus, Milestone, MilestoneStatus};

#[derive(Accounts)]
pub struct CancelBeforeFunding<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(mut, has_one = client @ EscrowError::Unauthorized)]
    pub gig: Account<'info, Gig>,

    #[account(
        mut,
        close = client,
        constraint = milestone.gig == gig.key() @ EscrowError::Unauthorized,
        constraint = milestone.status == MilestoneStatus::PendingFunding @ EscrowError::AlreadyFunded,
    )]
    pub milestone: Account<'info, Milestone>,
}

/// Closes a milestone that was never funded, refunding rent to the client.
pub fn handler(ctx: Context<CancelBeforeFunding>) -> Result<()> {
    ctx.accounts.gig.status = GigStatus::Cancelled;

    emit!(GigCancelled {
        gig: ctx.accounts.gig.key(),
        milestone: ctx.accounts.milestone.key(),
        index: ctx.accounts.milestone.index,
    });

    Ok(())
}
