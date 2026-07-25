pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use errors::*;
pub use events::*;
pub use instructions::*;
pub use state::*;

declare_id!("FFJ8YAVGUJP4SeDZrQ3g1d9fdQFq9hutsU1m4f3o1UXS");

#[program]
pub mod escrow {
    use super::*;

    /// Creates a new gig in Draft status with title, description, category, budget, and deadline.
    pub fn initialize_gig(
        ctx: Context<InitializeGig>,
        id: u64,
        title: String,
        description: String,
        category: String,
        budget: u64,
        deadline: i64,
    ) -> Result<()> {
        initialize_gig::handler(ctx, id, title, description, category, budget, deadline)
    }

    /// Updates a Draft gig's metadata.
    pub fn update_gig(
        ctx: Context<UpdateGig>,
        title: String,
        description: String,
        skills: String,
        category: String,
        budget: u64,
        deadline: i64,
    ) -> Result<()> {
        update_gig::handler(ctx, title, description, skills, category, budget, deadline)
    }

    /// Publishes a Draft gig, making it visible for freelancer assignment.
    pub fn publish_gig(ctx: Context<PublishGig>) -> Result<()> {
        publish_gig::handler(ctx)
    }

    /// Assigns a freelancer to a Published gig, transitioning it to Assigned.
    pub fn assign_freelancer(ctx: Context<AssignFreelancer>) -> Result<()> {
        assign_freelancer::handler(ctx)
    }

    /// Creates the next milestone for a gig, awaiting funding.
    pub fn create_milestone(ctx: Context<CreateMilestone>, amount: u64) -> Result<()> {
        create_milestone::handler(ctx, amount)
    }

    /// Funds a milestone by transferring USDC from the client into the escrow vault.
    pub fn fund_milestone(ctx: Context<FundMilestone>) -> Result<()> {
        fund_milestone::handler(ctx)
    }

    /// Records the freelancer's delivery submission.
    pub fn submit_delivery(ctx: Context<SubmitDelivery>) -> Result<()> {
        submit_delivery::handler(ctx)
    }

    /// Client-approved release of the full remaining milestone balance.
    pub fn approve_milestone(ctx: Context<ApproveMilestone>) -> Result<()> {
        approve_milestone::handler(ctx)
    }

    /// Permissionless 20% release after 72 hours of client inaction.
    pub fn partial_timeout_release(ctx: Context<PartialTimeoutRelease>) -> Result<()> {
        partial_timeout_release::handler(ctx)
    }

    /// Permissionless release of the remaining balance after 7 days of client inaction.
    pub fn full_timeout_release(ctx: Context<FullTimeoutRelease>) -> Result<()> {
        full_timeout_release::handler(ctx)
    }

    /// Marks a gig as Completed.
    pub fn complete_gig(ctx: Context<CompleteGig>) -> Result<()> {
        complete_gig::handler(ctx)
    }

    /// Archives a Completed gig.
    pub fn archive_gig(ctx: Context<ArchiveGig>) -> Result<()> {
        archive_gig::handler(ctx)
    }

    /// Cancels a non-terminal gig.
    pub fn cancel_gig(ctx: Context<CancelGig>) -> Result<()> {
        cancel_gig::handler(ctx)
    }

    /// Cancels and closes a milestone that was never funded.
    pub fn cancel_before_funding(ctx: Context<CancelBeforeFunding>) -> Result<()> {
        cancel_before_funding::handler(ctx)
    }
}
