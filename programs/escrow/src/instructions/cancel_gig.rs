use anchor_lang::prelude::*;

use crate::errors::EscrowError;
use crate::events::GigCancelled;
use crate::state::{Gig, GigStatus};

#[derive(Accounts)]
pub struct CancelGig<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        mut,
        has_one = client @ EscrowError::Unauthorized,
        constraint = gig.status == GigStatus::Draft
            || gig.status == GigStatus::Published
            || gig.status == GigStatus::Assigned
            @ EscrowError::InvalidStatus,
    )]
    pub gig: Account<'info, Gig>,
}

pub fn handler(ctx: Context<CancelGig>) -> Result<()> {
    let gig = &mut ctx.accounts.gig;
    gig.status = GigStatus::Cancelled;
    gig.updated_at = Clock::get()?.unix_timestamp;

    emit!(GigCancelled {
        gig: gig.key(),
        milestone: Pubkey::default(),
        index: 0,
    });

    Ok(())
}
