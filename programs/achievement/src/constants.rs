use anchor_lang::prelude::*;

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const ACHIEVEMENT_SEED: &[u8] = b"achievement";

/// Base URI for off-chain badge metadata JSON; the badge's slug is appended
/// (e.g. `<BASE>/first-gig.json`). Point this at the real metadata host
/// before mainnet deploy.
pub const METADATA_BASE_URI: &str = "https://paygig.app/achievements";

pub fn badge_name(badge_type: reputation::BadgeType) -> &'static str {
    use reputation::BadgeType::*;
    match badge_type {
        FirstGig => "First Gig",
        TenCompletedJobs => "Ten Gigs",
        HundredCompletedJobs => "Hundred Gigs",
        FiveStarPerformer => "Five Star",
        TrustedFreelancer => "Trusted Freelancer",
        FastDeliverer => "Fast Deliverer",
        TopRated => "Top Rated",
    }
}

pub fn badge_slug(badge_type: reputation::BadgeType) -> &'static str {
    use reputation::BadgeType::*;
    match badge_type {
        FirstGig => "first-gig",
        TenCompletedJobs => "ten-gigs",
        HundredCompletedJobs => "hundred-gigs",
        FiveStarPerformer => "five-star",
        TrustedFreelancer => "trusted-freelancer",
        FastDeliverer => "fast-deliverer",
        TopRated => "top-rated",
    }
}

pub fn badge_uri(badge_type: reputation::BadgeType) -> String {
    format!("{}/{}.json", METADATA_BASE_URI, badge_slug(badge_type))
}
