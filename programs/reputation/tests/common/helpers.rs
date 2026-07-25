use std::fmt::Debug;

use super::*;

/// Creates a profile and records `n` successful completions, each with `earnings_per`.
pub fn create_profile_with_completions(
    env: &mut Env,
    key: &Keypair,
    n: u64,
    earnings_per: u64,
) -> Pubkey {
    let pk = key.pubkey();
    let profile = init_profile(env, key);
    for _ in 0..n {
        update_completion_for(env, &pk, true, earnings_per).unwrap();
        env.svm.expire_blockhash();
    }
    profile
}

/// Creates a profile and submits the given ratings (by job_id 0..scores.len()).
/// client in env is used as the rater.
pub fn create_profile_with_ratings(env: &mut Env, key: &Keypair, scores: &[u8]) -> Pubkey {
    let pk = key.pubkey();
    let profile = init_profile(env, key);
    for (i, &score) in scores.iter().enumerate() {
        submit_rating_for(env, &pk, i as u64, score).unwrap();
        env.svm.expire_blockhash();
    }
    profile
}

/// Creates a fully populated profile with completions, ratings, and badges.
/// Returns (profile_key, profile_data).
pub fn create_full_profile(
    env: &mut Env,
    key: &Keypair,
) -> (Pubkey, UserProfile) {
    let pk = key.pubkey();
    let profile = create_profile_with_completions(env, key, 50, 1_000_000);
    for i in 0..10u64 {
        submit_rating_for(env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }
    for bt in [BadgeType::FirstGig, BadgeType::TenCompletedJobs, BadgeType::FiveStarPerformer] {
        award_badge_for(env, &pk, bt).unwrap();
        env.svm.expire_blockhash();
    }
    (profile, read_profile(&env.svm, &profile))
}

/// Verify the error string contains the given error code pattern.
pub fn expect_error<T: Debug>(result: Result<T, String>, expected: &str, msg: &str) {
    let err = result.unwrap_err();
    assert!(err.contains(expected), "{}: expected error containing {}, got: {}", msg, expected, err);
}

/// Verify the error string contains the given error code pattern, for raw send results.
pub fn expect_send_error(err: String, expected: &str, msg: &str) {
    assert!(
        err.contains(expected),
        "{}: expected error containing {}, got: {}",
        msg,
        expected,
        err
    );
}

/// Check that two profiles are identical in all fields.
pub fn assert_profiles_equal(a: &UserProfile, b: &UserProfile) {
    assert_eq!(a.authority, b.authority, "authority mismatch");
    assert_eq!(a.completed_jobs, b.completed_jobs, "completed_jobs mismatch");
    assert_eq!(a.successful_jobs, b.successful_jobs, "successful_jobs mismatch");
    assert_eq!(a.cancelled_jobs, b.cancelled_jobs, "cancelled_jobs mismatch");
    assert_eq!(a.total_earnings, b.total_earnings, "total_earnings mismatch");
    assert_eq!(a.rating_sum, b.rating_sum, "rating_sum mismatch");
    assert_eq!(a.rating_count, b.rating_count, "rating_count mismatch");
    assert_eq!(a.average_rating, b.average_rating, "average_rating mismatch");
    assert_eq!(a.reputation_score, b.reputation_score, "reputation_score mismatch");
    assert_eq!(a.badges_earned, b.badges_earned, "badges_earned mismatch");
    assert_eq!(a.created_at, b.created_at, "created_at mismatch");
    assert_eq!(a.bump, b.bump, "bump mismatch");
}

/// Check all state invariants hold for a profile.
pub fn assert_invariants(profile: &UserProfile) {
    assert!(
        profile.completed_jobs >= profile.successful_jobs,
        "invariant: completed_jobs ({}) >= successful_jobs ({})",
        profile.completed_jobs,
        profile.successful_jobs
    );
    assert!(
        profile.completed_jobs >= profile.cancelled_jobs,
        "invariant: completed_jobs ({}) >= cancelled_jobs ({})",
        profile.completed_jobs,
        profile.cancelled_jobs
    );
    assert!(
        profile.average_rating <= 500,
        "invariant: average_rating ({}) <= 500",
        profile.average_rating
    );
    assert!(
        profile.reputation_score <= 1000,
        "invariant: reputation_score ({}) <= 1000",
        profile.reputation_score
    );
    if profile.rating_count > 0 {
        assert!(
            profile.average_rating >= 100,
            "invariant: with ratings, average_rating ({}) >= 100 (1 star scaled)",
            profile.average_rating
        );
    }
    assert!(
        profile.created_at <= profile.updated_at,
        "invariant: created_at ({}) <= updated_at ({})",
        profile.created_at,
        profile.updated_at
    );
}

/// Assert that a specific event was emitted (by checking for "Program data:" in logs).
pub fn assert_event_emitted(logs: &[String]) {
    assert!(
        logs.iter().any(|l| l.contains("Program data:")),
        "Expected an event to be emitted (Program data: log entry)"
    );
}

pub fn assert_event_not_emitted(logs: &[String]) {
    let has: Vec<_> = logs.iter().filter(|l| l.contains("Program data:")).collect();
    assert!(
        has.is_empty(),
        "Expected NO event emitted, but found {} 'Program data:' entries",
        has.len()
    );
}
