use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Signer is not authorized to perform this action")]
    Unauthorized,
    #[msg("Account is not in the required status for this action")]
    InvalidStatus,
    #[msg("Milestone has already been funded")]
    AlreadyFunded,
    #[msg("Vault does not hold sufficient funds")]
    InsufficientFunds,
    #[msg("Token mint does not match the expected mint")]
    InvalidMint,
    #[msg("Milestone has already been submitted")]
    MilestoneAlreadySubmitted,
    #[msg("Timeout window has not yet elapsed")]
    TimeoutNotReached,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic error")]
    MathError,
    #[msg("Gig id exceeds maximum length")]
    GigIdTooLong,
    #[msg("Milestone amount must be greater than zero")]
    InvalidAmount,
    #[msg("Title exceeds maximum length")]
    TitleTooLong,
    #[msg("Description exceeds maximum length")]
    DescriptionTooLong,
    #[msg("Skills string exceeds maximum length")]
    SkillsTooLong,
    #[msg("Category exceeds maximum length")]
    CategoryTooLong,
    #[msg("Metadata exceeds maximum length")]
    MetadataTooLong,
    #[msg("Deadline is too soon or in the past")]
    InvalidDeadline,
    #[msg("Budget must be greater than zero")]
    InvalidBudget,
    #[msg("Freelancer is already assigned to this gig")]
    FreelancerAlreadyAssigned,
    #[msg("Gig must be in Draft status")]
    NotDraftStatus,
    #[msg("Gig must be in Published status")]
    NotPublishedStatus,
    #[msg("Gig must be in Assigned status")]
    NotAssignedStatus,
    #[msg("Gig must be in Completed status")]
    NotCompletedStatus,
    #[msg("Gig must be in Completed or Cancelled or Archived status")]
    TerminalStatus,
}
