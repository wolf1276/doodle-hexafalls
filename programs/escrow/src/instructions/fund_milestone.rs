use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, TransferChecked};

use crate::constants::VAULT_SEED;
use crate::errors::EscrowError;
use crate::events::MilestoneFunded;
use crate::state::{EscrowVault, Gig, Milestone, MilestoneStatus};
use crate::utils::checked_add;

#[derive(Accounts)]
pub struct FundMilestone<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        has_one = client @ EscrowError::Unauthorized,
        has_one = mint @ EscrowError::InvalidMint,
    )]
    pub gig: Account<'info, Gig>,

    #[account(
        mut,
        constraint = milestone.gig == gig.key() @ EscrowError::Unauthorized,
        constraint = milestone.status == MilestoneStatus::PendingFunding @ EscrowError::AlreadyFunded,
    )]
    pub milestone: Account<'info, Milestone>,

    #[account(
        init_if_needed,
        payer = client,
        space = EscrowVault::INIT_SPACE,
        seeds = [VAULT_SEED, gig.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, EscrowVault>,

    #[account(
        init_if_needed,
        payer = client,
        token::mint = mint,
        token::authority = vault,
        seeds = [VAULT_SEED, gig.key().as_ref(), b"token"],
        bump,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = client_token_account.mint == mint.key() @ EscrowError::InvalidMint,
        constraint = client_token_account.owner == client.key() @ EscrowError::Unauthorized,
    )]
    pub client_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Transfers `milestone.amount` in USDC from the client into the gig's escrow vault.
pub fn handler(ctx: Context<FundMilestone>) -> Result<()> {
    let amount = ctx.accounts.milestone.amount;

    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.client_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.client.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.mint.decimals,
    )?;

    let vault = &mut ctx.accounts.vault;
    if vault.token_account == Pubkey::default() {
        vault.gig = ctx.accounts.gig.key();
        vault.token_account = ctx.accounts.vault_token_account.key();
        vault.mint = ctx.accounts.mint.key();
        vault.bump = ctx.bumps.vault;
    } else {
        require_keys_eq!(vault.mint, ctx.accounts.mint.key(), EscrowError::InvalidMint);
    }
    vault.total_locked = checked_add(vault.total_locked, amount)?;

    let milestone = &mut ctx.accounts.milestone;
    milestone.status = MilestoneStatus::Funded;

    emit!(MilestoneFunded {
        gig: ctx.accounts.gig.key(),
        milestone: milestone.key(),
        amount,
    });

    Ok(())
}
