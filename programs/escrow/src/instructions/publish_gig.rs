use anchor_lang::prelude::*;

use crate::errors::EscrowError;
use crate::events::GigPublished;
use crate::state::{Gig, GigStatus};

#[derive(Accounts)]
pub struct PublishGig<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        mut,
        has_one = client @ EscrowError::Unauthorized,
        constraint = gig.status == GigStatus::Draft @ EscrowError::NotDraftStatus,
    )]
    pub gig: Account<'info, Gig>,
}

pub fn handler(ctx: Context<PublishGig>) -> Result<()> {
    let gig = &mut ctx.accounts.gig;
    gig.status = GigStatus::Published;
    gig.updated_at = Clock::get()?.unix_timestamp;

    emit!(GigPublished {
        gig: gig.key(),
        id: gig.id,
    });

    Ok(())
}
