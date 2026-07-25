/// Regression tests for bugs discovered during development.
///
/// Each test documents a specific bug scenario and verifies it
/// does not recur.

mod common;

use common::*;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

/// Regression: Self-dealing check should not be bypassable by swapping
/// the freelancer and client accounts in the accounts struct.
#[test]
fn test_self_dealing_check_not_bypassed() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    env.svm.expire_blockhash();

    // Try to rate as the freelancer themselves (should fail self-dealing)
    // Try to rate as the freelancer themselves (using the freelancer keypair as client)
    let f_clone = freelancer.insecure_clone();
    let result = submit_rating_as(&mut env, &f_clone, &freelancer_pk, 1, 5, DEFAULT_REVIEW_HASH);
    expect_error(result.map(|_| ()), ERR_SELF_DEALING, "self-dealing should fail even with valid profile");

    // Also try with explicit same-key call
    let (profile, _) = profile_pda(&freelancer_pk);
    let (rating, _) = rating_pda(99);
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_submit_rating(
            &freelancer_pk,
            &freelancer_pk,
            &profile,
            &rating,
            99,
            5,
            DEFAULT_REVIEW_HASH,
        )],
        &[&env.payer.insecure_clone(), &freelancer],
    );
    expect_send_error(result.unwrap_err(), ERR_SELF_DEALING, "explicit self-dealing");
}

/// Regression: Duplicate prevention should not be bypassed by using different
/// instruction parameters while reusing the same PDA.
#[test]
fn test_duplicate_prevention_not_bypassed() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &pk, 1, 5).unwrap();

    env.svm.expire_blockhash();
    let client = env.client.insecure_clone();
    let result = submit_rating_as(&mut env, &client, &pk, 1, 4, ALT_REVIEW_HASH_1);
    assert!(result.is_err(), "duplicate rating with different review hash should fail");
}

/// Regression: Authority check should not be bypassable.
#[test]
fn test_authority_check_not_bypassed() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    // A different authority keypair should not be able to call update_completion
    let fake_authority = Keypair::new();
    env.svm.airdrop(&fake_authority.pubkey(), 10_000_000_000).unwrap();

    let (profile, _) = profile_pda(&pk);
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_update_completion(&fake_authority.pubkey(), &profile, true, 100)],
        &[&env.payer.insecure_clone(), &fake_authority],
    );
    expect_send_error(result.unwrap_err(), ERR_UNAUTHORIZED, "fake authority should be rejected");
}

/// Regression: Eligibility check should not be bypassable for non-attested badges.
#[test]
fn test_eligibility_check_not_bypassed() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    // No completions, try to get FirstGig
    let result = award_badge_for(&mut env, &pk, BadgeType::FirstGig);
    expect_error(result, ERR_BADGE_NOT_ELIGIBLE, "FirstGig requires >= 1 completion");

    update_completion_for(&mut env, &pk, true, 100).unwrap();
    env.svm.expire_blockhash();
    assert!(award_badge_for(&mut env, &pk, BadgeType::FirstGig).is_ok());
}

/// Regression: Range check for ratings should not be bypassable.
#[test]
fn test_range_check_not_bypassed() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    // Test all boundary values
    assert!(submit_rating_for(&mut env, &pk, 1, 1).is_ok(), "1 should be valid");
    env.svm.expire_blockhash();
    assert!(submit_rating_for(&mut env, &pk, 2, 5).is_ok(), "5 should be valid");

    // Values just outside bounds
    let result = submit_rating_for(&mut env, &pk, 3, 0);
    expect_error(result, ERR_INVALID_RATING, "0 should be rejected");
    let result = submit_rating_for(&mut env, &pk, 4, 6);
    expect_error(result, ERR_INVALID_RATING, "6 should be rejected");
}

/// Regression: Cancelled jobs should not add to total_earnings.
#[test]
fn test_cancelled_jobs_dont_add_earnings() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    // Successful: 100 earnings
    update_completion_for(&mut env, &pk, true, 100).unwrap();
    let p = read_profile(&env.svm, &profile_key);
    assert_eq!(p.total_earnings, 100);
    assert_eq!(p.completed_jobs, 1);
    assert_eq!(p.successful_jobs, 1);

    // Cancelled: 0 earnings added
    update_completion_for(&mut env, &pk, false, 9999).unwrap();
    let p = read_profile(&env.svm, &profile_key);
    assert_eq!(p.total_earnings, 100, "cancelled should not add earnings");
    assert_eq!(p.completed_jobs, 2);
    assert_eq!(p.cancelled_jobs, 1);
}

/// Regression: Badge metadata empty string should be accepted.
#[test]
fn test_empty_metadata_accepted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    update_completion_for(&mut env, &pk, true, 100).unwrap();

    let res = award_badge_for_with_metadata(&mut env, &pk, BadgeType::FirstGig, String::new());
    assert!(res.is_ok(), "empty metadata should be accepted");
}

/// Regression: Zero earnings for successful completion is valid.
#[test]
fn test_zero_earnings_successful() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &pk, true, 0).unwrap();
    let p = read_profile(&env.svm, &profile_key);
    assert_eq!(p.completed_jobs, 1);
    assert_eq!(p.successful_jobs, 1);
    assert_eq!(p.total_earnings, 0);
}

/// Regression: Ensuring the reputation score is recoverable after 100% cancellations.
#[test]
fn test_all_cancellations_minimum_score() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for _ in 0..10 {
        update_completion_for(&mut env, &pk, false, 0).unwrap();
        env.svm.expire_blockhash();
    }

    let p = read_profile(&env.svm, &profile_key);
    // Score can be 0 due to cancellations, but should not underflow or panic
    assert!(p.reputation_score <= 1000);
    assert_eq!(p.reputation_score, expected_reputation_score(&p));
}

/// Regression: Rapid successive calls should not corrupt state.
#[test]
fn test_rapid_successive_calls() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for i in 0..10u64 {
        update_completion_for(&mut env, &pk, true, 100).unwrap();
        env.svm.expire_blockhash();
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let p = read_profile(&env.svm, &profile_key);
    assert_eq!(p.completed_jobs, 10);
    assert_eq!(p.successful_jobs, 10);
    assert_eq!(p.rating_count, 10);
    assert_eq!(p.rating_sum, 50);
    assert_eq!(p.average_rating, 500);
    assert_invariants(&p);
}
