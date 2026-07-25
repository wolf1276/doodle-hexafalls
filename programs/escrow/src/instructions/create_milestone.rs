use anchor_lang::prelude::*;

use crate::constants::MILESTONE_SEED;
use crate::errors::EscrowError;
use crate::events::MilestoneCreated;
use crate::state::{Gig, GigStatus, Milestone, MilestoneStatus};
use crate::utils::checked_add;

#[derive(Accounts)]
pub struct CreateMilestone<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        mut,
        has_one = client @ EscrowError::Unauthorized,
        constraint = gig.status == GigStatus::Active @ EscrowError::InvalidStatus,
    )]
    pub gig: Account<'info, Gig>,

    #[account(
        init,
        payer = client,
        space = Milestone::INIT_SPACE,
        seeds = [MILESTONE_SEED, gig.key().as_ref(), gig.milestone_count.to_le_bytes().as_ref()],
        bump,
    )]
    pub milestone: Account<'info, Milestone>,

    pub system_program: Program<'info, System>,
}

/// Creates the next sequential Milestone PDA for `gig`, awaiting funding.
pub fn handler(ctx: Context<CreateMilestone>, amount: u64) -> Result<()> {
    require!(amount > 0, EscrowError::InvalidAmount);

    let gig = &mut ctx.accounts.gig;
    let index = gig.milestone_count;

    let milestone = &mut ctx.accounts.milestone;
    milestone.gig = gig.key();
    milestone.index = index;
    milestone.amount = amount;
    milestone.released = 0;
    milestone.status = MilestoneStatus::PendingFunding;
    milestone.submitted_at = 0;
    milestone.approved_at = 0;
    milestone.bump = ctx.bumps.milestone;

    gig.milestone_count = checked_add(gig.milestone_count as u64, 1)? as u32;

    emit!(MilestoneCreated {
        gig: gig.key(),
        milestone: milestone.key(),
        index,
        amount,
    });

    Ok(())
}
