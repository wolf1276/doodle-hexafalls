use anchor_lang::prelude::*;

use crate::errors::EscrowError;
use crate::events::GigCompleted;
use crate::state::{Gig, GigStatus};

#[derive(Accounts)]
pub struct CompleteGig<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        mut,
        has_one = client @ EscrowError::Unauthorized,
        constraint = gig.status == GigStatus::Assigned @ EscrowError::NotAssignedStatus,
    )]
    pub gig: Account<'info, Gig>,
}

pub fn handler(ctx: Context<CompleteGig>) -> Result<()> {
    let gig = &mut ctx.accounts.gig;
    gig.status = GigStatus::Completed;
    gig.updated_at = Clock::get()?.unix_timestamp;

    emit!(GigCompleted {
        gig: gig.key(),
        id: gig.id,
    });

    Ok(())
}
