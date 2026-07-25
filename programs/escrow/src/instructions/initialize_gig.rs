use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::constants::GIG_SEED;
use crate::errors::EscrowError;
use crate::events::GigCreated;
use crate::state::{Gig, GigStatus};

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct InitializeGig<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    /// CHECK: freelancer is only stored as a pubkey reference, never signs here.
    pub freelancer: UncheckedAccount<'info>,

    /// The USDC (or other SPL) mint every milestone of this gig will be funded in.
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = client,
        space = Gig::INIT_SPACE,
        seeds = [GIG_SEED, id.to_le_bytes().as_ref()],
        bump,
    )]
    pub gig: Account<'info, Gig>,

    pub system_program: Program<'info, System>,
}

/// Creates a new Gig PDA owned by `client`, paired with `freelancer`, pinned to `mint`.
pub fn handler(ctx: Context<InitializeGig>, id: u64) -> Result<()> {
    require_keys_neq!(
        ctx.accounts.client.key(),
        ctx.accounts.freelancer.key(),
        EscrowError::Unauthorized
    );

    let gig = &mut ctx.accounts.gig;
    let now = Clock::get()?.unix_timestamp;

    gig.id = id;
    gig.client = ctx.accounts.client.key();
    gig.freelancer = ctx.accounts.freelancer.key();
    gig.mint = ctx.accounts.mint.key();
    gig.milestone_count = 0;
    gig.active_milestone = 0;
    gig.status = GigStatus::Active;
    gig.created_at = now;
    gig.bump = ctx.bumps.gig;

    emit!(GigCreated {
        gig: gig.key(),
        id,
        client: gig.client,
        freelancer: gig.freelancer,
        created_at: now,
    });

    Ok(())
}
