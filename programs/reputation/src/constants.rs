use anchor_lang::prelude::*;

#[constant]
pub const PROFILE_SEED: &[u8] = b"profile";

#[constant]
pub const RATING_SEED: &[u8] = b"rating";

#[constant]
pub const BADGE_SEED: &[u8] = b"badge";

/// Signer authorized to record job completions until the Escrow Program
/// can invoke `update_completion` directly via CPI. Swapping this for a
/// CPI-only check later does not require any account layout changes.
pub const REPUTATION_AUTHORITY: Pubkey = pubkey!("vo18wuiY77EZa16yYKRdAjp2mj3g6GCvMHH8wkn6LAz");

pub const MIN_RATING: u8 = 1;
pub const MAX_RATING: u8 = 5;

/// `average_rating` is stored scaled by 100 (e.g. 450 == 4.50 stars).
pub const RATING_SCALE: u64 = 100;

/// Completed-job count at which the volume component of the reputation
/// score saturates.
pub const VOLUME_SCORE_CAP: u64 = 100;

/// Lifetime earnings (in mint base units) at which the earnings component
/// of the reputation score saturates.
pub const EARNINGS_SCORE_CAP: u64 = 1_000_000_000;

pub const CANCELLATION_PENALTY_PER_JOB: u64 = 5;
pub const MAX_REPUTATION_SCORE: u64 = 1000;
