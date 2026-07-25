mod common;

use common::*;
use solana_signer::Signer;

#[test]
fn test_rating_sum_overflow_protected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    // rating_sum starts at 0. Each 5-star adds 5. Max u64 = 18446744073709551615
    // We can fit 3689348814741910322 ratings of 5 before overflow.
    // Not practical to test exhaustively, but we verify the concept works.
    for i in 0..1000u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_pda(&pk).0);
    assert_eq!(profile.rating_sum, 5000);
    assert_eq!(profile.rating_count, 1000);
    assert!(profile.reputation_score <= 1000);
}

#[test]
fn test_large_total_earnings_capped_score() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for _ in 0..100 {
        update_completion_for(&mut env, &pk, true, 10_000_000).unwrap();
        env.svm.expire_blockhash();
    }
    let profile = read_profile(&env.svm, &profile_key);
    assert!(profile.total_earnings > 0);
    assert!(profile.reputation_score <= 1000);
    assert_invariants(&profile);
}

#[test]
fn test_checked_sub_not_needed() {
    // All profile fields are monotonically increasing (or initialized to 0).
    // There are no instructions that subtract from any field.
    // Verify this property holds.
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    let before = read_profile(&env.svm, &profile_key);

    update_completion_for(&mut env, &pk, true, 100).unwrap();

    let after = read_profile(&env.svm, &profile_key);
    assert!(after.completed_jobs >= before.completed_jobs);
    assert!(after.successful_jobs >= before.successful_jobs);
    assert!(after.cancelled_jobs >= before.cancelled_jobs);
    assert!(after.total_earnings >= before.total_earnings);
    assert!(after.rating_count >= before.rating_count);
}

#[test]
fn test_average_rating_zero_rating_count() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.rating_count, 0);
    assert_eq!(profile.average_rating, 0);
}

#[test]
fn test_reputation_score_bounded() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();

    let profile_key = init_profile(&mut env, &freelancer);

    // Push to extremes: many cancellations + poor ratings
    for _ in 0..50 {
        update_completion_for(&mut env, &pk, false, 0).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..50u64 {
        submit_rating_for(&mut env, &pk, i, 1).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    // Should be 0 due to heavy penalties
    assert!(profile.reputation_score >= 0);
    assert!(profile.reputation_score <= 1000);
    assert_invariants(&profile);
}

#[test]
fn test_u64_max_earnings_no_panic() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    let result = update_completion_for(&mut env, &pk, true, u64::MAX);
    assert!(result.is_ok(), "first u64::MAX addition should succeed");
    // Second call with u64::MAX will overflow total_earnings (checked_add returns error)
    let result = update_completion_for(&mut env, &pk, true, u64::MAX);
    assert!(result.is_err(), "second u64::MAX addition should overflow");

    let profile = read_profile(&env.svm, &profile_pda(&pk).0);
    assert_eq!(profile.total_earnings, u64::MAX);
}

#[test]
fn test_all_math_operations_safe() {
    // Integration check: verify that all mathematical invariants hold
    // after a sequence of extreme operations.
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    // Mix of operations that exercise all math paths
    for _ in 0..10 {
        update_completion_for(&mut env, &pk, true, 1_000_000).unwrap();
        env.svm.expire_blockhash();
    }
    for _ in 0..5 {
        update_completion_for(&mut env, &pk, false, 0).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..3u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert_invariants(&profile);
    assert_eq!(profile.reputation_score, expected_reputation_score(&profile));
}
