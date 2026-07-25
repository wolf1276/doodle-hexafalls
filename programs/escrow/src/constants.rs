use anchor_lang::prelude::*;

#[constant]
pub const GIG_SEED: &[u8] = b"gig";

#[constant]
pub const MILESTONE_SEED: &[u8] = b"milestone";

#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

pub const SECONDS_PER_DAY: i64 = 86_400;

pub const PARTIAL_RELEASE_PERCENT: u64 = 20;
pub const FULL_RELEASE_PERCENT: u64 = 80;

pub const PARTIAL_TIMEOUT: i64 = 72 * 3_600; // 72 hours
pub const FULL_TIMEOUT: i64 = 7 * SECONDS_PER_DAY; // 7 days

pub const MAX_GIG_ID_LEN: usize = 32;

pub const MAX_TITLE_LEN: usize = 100;
pub const MAX_DESCRIPTION_LEN: usize = 500;
pub const MAX_SKILLS_LEN: usize = 200;
pub const MAX_CATEGORY_LEN: usize = 50;
pub const MAX_METADATA_LEN: usize = 256;

pub const MIN_DEADLINE_SECS: i64 = 86_400; // 1 day from now minimum deadline
