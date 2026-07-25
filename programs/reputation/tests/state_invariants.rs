mod common;

use common::*;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

/// Verify that completed_jobs >= successful_jobs ALWAYS holds.
#[test]
fn test_completed_jobs_ge_successful_jobs() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    // Initial state
    let p = read_profile(&env.svm, &profile_key);
    assert!(p.completed_jobs >= p.successful_jobs);

    // After successful completion
    update_completion_for(&mut env, &pk, true, 100).unwrap();
    let p = read_profile(&env.svm, &profile_key);
    assert!(p.completed_jobs >= p.successful_jobs);

    // After cancelled completion
    env.svm.expire_blockhash();
    update_completion_for(&mut env, &pk, false, 0).unwrap();
    let p = read_profile(&env.svm, &profile_key);
    assert!(p.completed_jobs >= p.successful_jobs);

    // After many mixed completions
    for _ in 0..20 {
        update_completion_for(&mut env, &pk, true, 100).unwrap();
        env.svm.expire_blockhash();
    }
    for _ in 0..5 {
        update_completion_for(&mut env, &pk, false, 0).unwrap();
        env.svm.expire_blockhash();
    }
    let p = read_profile(&env.svm, &profile_key);
    assert!(p.completed_jobs >= p.successful_jobs);
    assert_eq!(p.completed_jobs, p.successful_jobs + p.cancelled_jobs);
}

/// Verify that average_rating is always in [0, 500] (0..=5.00 scaled by 100).
#[test]
fn test_average_rating_bounds() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    // No ratings: average is 0
    let p = read_profile(&env.svm, &profile_key);
    assert_eq!(p.average_rating, 0);

    // After 1-star ratings
    for i in 0..5u64 {
        submit_rating_for(&mut env, &pk, i, 1).unwrap();
        env.svm.expire_blockhash();
    }
    let p = read_profile(&env.svm, &profile_key);
    assert!(p.average_rating >= 0);
    assert!(p.average_rating <= 500);

    // After 5-star ratings
    for i in 5..10u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }
    let p = read_profile(&env.svm, &profile_key);
    assert!(p.average_rating >= 0);
    assert!(p.average_rating <= 500);
}

/// Verify that badges are unique per type on a profile.
#[test]
fn test_badges_unique() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    for _ in 0..100 {
        update_completion_for(&mut env, &pk, true, 100).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..10u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let badges = award_all_eligible(&mut env, &pk);
    // Each badge type should appear at most once
    let mut seen_types: Vec<u8> = Vec::new();
    for bt in &badges {
        let code = *bt as u8;
        assert!(!seen_types.contains(&code), "duplicate badge type {}", code);
        seen_types.push(code);
    }
    assert!(badges.len() > 0, "should have awarded at least one badge");
    assert!(badges.len() <= ALL_BADGE_TYPES.len(), "cannot exceed total badge types");
}

fn award_all_eligible(env: &mut Env, pk: &Pubkey) -> Vec<BadgeType> {
    let mut awarded = Vec::new();
    for &bt in &ALL_BADGE_TYPES {
        match award_badge_for(env, pk, bt) {
            Ok(_) => awarded.push(bt),
            Err(_) => {}
        }
        env.svm.expire_blockhash();
    }
    awarded
}

/// Verify that earnings never decrease.
#[test]
fn test_earnings_never_decrease() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for amount in &[100u64, 200, 0, 500, 1_000_000_000] {
        let before = read_profile(&env.svm, &profile_key).total_earnings;
        update_completion_for(&mut env, &pk, true, *amount).unwrap();
        let after = read_profile(&env.svm, &profile_key).total_earnings;
        assert!(after >= before, "earnings decreased from {} to {}", before, after);
    }
}

/// Verify that ratings are immutable once submitted.
#[test]
fn test_ratings_immutable() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &pk, 1, 5).unwrap();
    let (rating_key, _) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.score, 5);

    // Cannot modify (Anchor init constraint prevents this)
    let result = submit_rating_for(&mut env, &pk, 1, 3);
    assert!(result.is_err());
    let rating_after = read_rating(&env.svm, &rating_key);
    assert_eq!(rating_after.score, 5);
    assert_eq!(rating_after.freelancer, rating.freelancer);
    assert_eq!(rating_after.client, rating.client);
    assert_eq!(rating_after.job_id, rating.job_id);
}

/// Verify that profile authority is immutable.
#[test]
fn test_profile_authority_immutable() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    let profile = read_profile(&env.svm, &profile_key);
    let original_authority = profile.authority;

    // No instruction modifies authority; verify it stays the same after operations
    update_completion_for(&mut env, &pk, true, 100).unwrap();
    submit_rating_for(&mut env, &pk, 1, 5).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.authority, original_authority);
}

/// Verify that created_at never changes.
#[test]
fn test_created_at_never_changes() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    let created_at = read_profile(&env.svm, &profile_key).created_at;

    warp_seconds(&mut env.svm, 86400);
    update_completion_for(&mut env, &pk, true, 100).unwrap();
    let p = read_profile(&env.svm, &profile_key);
    assert_eq!(p.created_at, created_at, "created_at should never change");

    submit_rating_for(&mut env, &pk, 1, 5).unwrap();
    let p = read_profile(&env.svm, &profile_key);
    assert_eq!(p.created_at, created_at, "created_at should never change after rating");
}

/// Verify that updated_at is monotonic (never decreases).
#[test]
fn test_updated_at_monotonic() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    let initial_updated = read_profile(&env.svm, &profile_key).updated_at;

    warp_seconds(&mut env.svm, 1);
    update_completion_for(&mut env, &pk, true, 100).unwrap();
    let p = read_profile(&env.svm, &profile_key);
    assert!(p.updated_at >= initial_updated, "updated_at should not decrease after completion");

    warp_seconds(&mut env.svm, 1);
    submit_rating_for(&mut env, &pk, 1, 5).unwrap();
    let p = read_profile(&env.svm, &profile_key);
    assert!(p.updated_at >= initial_updated, "updated_at should not decrease after rating");
}

/// Verify reputation_score is deterministic (same inputs -> same output).
#[test]
fn test_reputation_score_deterministic() {
    // Create two identical profiles and verify they have the same score
    let mut env = setup();
    let f1 = env.freelancer.insecure_clone();
    let pk1 = f1.pubkey();
    init_profile(&mut env, &f1);
    update_completion_for(&mut env, &pk1, true, 500).unwrap();
    submit_rating_for(&mut env, &pk1, 1, 4).unwrap();
    let score1 = read_profile(&env.svm, &profile_pda(&pk1).0).reputation_score;

    let f2 = Keypair::new();
    env.svm.airdrop(&f2.pubkey(), 10_000_000_000).unwrap();
    let pk2 = f2.pubkey();
    init_profile(&mut env, &f2);
    env.svm.expire_blockhash();
    update_completion_for(&mut env, &pk2, true, 500).unwrap();
    env.svm.expire_blockhash();
    submit_rating_for(&mut env, &pk2, 2, 4).unwrap();
    let score2 = read_profile(&env.svm, &profile_pda(&pk2).0).reputation_score;

    assert_eq!(score1, score2, "reputation score must be deterministic");
}

/// All invariants hold after a complex sequence of operations.
#[test]
fn test_all_invariants_hold_after_multiple_operations() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    // Complex sequence
    for i in 0..15 {
        let successful = i % 4 != 0;
        update_completion_for(
            &mut env,
            &pk,
            successful,
            if successful { (i as u64 + 1) * 100 } else { 0 },
        )
        .unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..8u64 {
        submit_rating_for(&mut env, &pk, i, (i as u8 % 5) + 1).unwrap();
        env.svm.expire_blockhash();
    }

    // Check ALL invariants
    let p = read_profile(&env.svm, &profile_key);

    // completed_jobs >= successful_jobs
    assert!(p.completed_jobs >= p.successful_jobs);
    assert_eq!(p.completed_jobs, p.successful_jobs + p.cancelled_jobs);

    // average_rating in bounds
    assert!(p.average_rating <= 500);

    // reputation capped
    assert!(p.reputation_score <= 1000);

    // created_at <= updated_at
    assert!(p.created_at <= p.updated_at);

    // Earnings consistency (total_earnings should be sum of successful earnings)
    let expected_earnings: u64 = (0..15)
        .filter(|i| i % 4 != 0)
        .map(|i| (i as u64 + 1) * 100)
        .sum();
    assert_eq!(p.total_earnings, expected_earnings);

    // Off-chain verification
    assert_eq!(p.reputation_score, expected_reputation_score(&p));
}
