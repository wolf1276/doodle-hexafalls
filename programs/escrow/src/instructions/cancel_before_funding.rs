use anchor_lang::prelude::*;
use gig::cpi::accounts::MarkCancelledByEscrow;
use gig::Gig;

use crate::constants::ESCROW_AUTHORITY_SEED;
use crate::errors::EscrowError;
use crate::events::MilestoneCancelledBeforeFunding;
use crate::state::{Milestone, MilestoneStatus};

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

    /// CHECK: PDA identity only; used purely as escrow's CPI-signer into the gig program.
    #[account(seeds = [ESCROW_AUTHORITY_SEED], bump)]
    pub escrow_authority: UncheckedAccount<'info>,

    pub gig_program: Program<'info, gig::program::Gig>,
}

/// Closes a milestone that was never funded, refunding rent to the client, and
/// CPIs into the gig program to cancel the gig (escrow owns this transition
/// since only escrow knows a funded milestone never delivered).
pub fn handler(ctx: Context<CancelBeforeFunding>) -> Result<()> {
    emit!(MilestoneCancelledBeforeFunding {
        gig: ctx.accounts.gig.key(),
        milestone: ctx.accounts.milestone.key(),
        index: ctx.accounts.milestone.index,
    });

    let authority_bump = ctx.bumps.escrow_authority;
    let authority_signer_seeds: &[&[&[u8]]] = &[&[ESCROW_AUTHORITY_SEED, &[authority_bump]]];

    gig::cpi::mark_cancelled_by_escrow(CpiContext::new_with_signer(
        ctx.accounts.gig_program.key(),
        MarkCancelledByEscrow {
            escrow_authority: ctx.accounts.escrow_authority.to_account_info(),
            gig: ctx.accounts.gig.to_account_info(),
        },
        authority_signer_seeds,
    ))?;

    Ok(())
}
