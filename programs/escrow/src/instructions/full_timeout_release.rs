use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, TransferChecked};

use crate::constants::{FULL_TIMEOUT, VAULT_SEED};
use crate::errors::EscrowError;
use crate::events::FullReleaseExecuted;
use crate::state::{EscrowVault, Gig, GigStatus, Milestone, MilestoneStatus};
use crate::utils::{checked_add, checked_sub};

#[derive(Accounts)]
pub struct FullTimeoutRelease<'info> {
    #[account(mut)]
    pub gig: Account<'info, Gig>,

    #[account(
        mut,
        constraint = milestone.gig == gig.key() @ EscrowError::Unauthorized,
        constraint = milestone.status == MilestoneStatus::PartialReleased @ EscrowError::InvalidStatus,
    )]
    pub milestone: Account<'info, Milestone>,

    #[account(
        mut,
        seeds = [VAULT_SEED, gig.key().as_ref()],
        bump = vault.bump,
        has_one = mint @ EscrowError::InvalidMint,
    )]
    pub vault: Account<'info, EscrowVault>,

    #[account(mut, address = vault.token_account @ EscrowError::Unauthorized)]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = freelancer_token_account.mint == mint.key() @ EscrowError::InvalidMint,
        constraint = freelancer_token_account.owner == gig.freelancer @ EscrowError::Unauthorized,
    )]
    pub freelancer_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

/// Permissionlessly releases the remaining balance once 7 days have elapsed
/// since delivery submission without client approval.
pub fn handler(ctx: Context<FullTimeoutRelease>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let milestone = &ctx.accounts.milestone;

    require!(
        now >= milestone.submitted_at.saturating_add(FULL_TIMEOUT),
        EscrowError::TimeoutNotReached
    );

    let remaining = checked_sub(milestone.amount, milestone.released)?;
    require!(remaining > 0, EscrowError::InsufficientFunds);

    let gig_key = ctx.accounts.gig.key();
    let vault_bump = ctx.accounts.vault.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[VAULT_SEED, gig_key.as_ref(), &[vault_bump]]];

    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.vault_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.freelancer_token_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            signer_seeds,
        ),
        remaining,
        ctx.accounts.mint.decimals,
    )?;

    let vault = &mut ctx.accounts.vault;
    vault.total_released = checked_add(vault.total_released, remaining)?;

    let milestone = &mut ctx.accounts.milestone;
    milestone.released = checked_add(milestone.released, remaining)?;
    milestone.status = MilestoneStatus::Completed;

    let gig = &mut ctx.accounts.gig;
    if gig.active_milestone + 1 >= gig.milestone_count {
        gig.status = GigStatus::Completed;
    } else {
        gig.active_milestone = checked_add(gig.active_milestone as u64, 1)? as u32;
    }

    emit!(FullReleaseExecuted {
        gig: gig_key,
        milestone: milestone.key(),
        amount_released: remaining,
    });

    Ok(())
}
