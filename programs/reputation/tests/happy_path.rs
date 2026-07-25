mod common;

use common::*;
use solana_clock::Clock;
use solana_signer::Signer;

#[test]
fn test_initialize_profile() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();

    let clock_before = env.svm.get_sysvar::<Clock>().unix_timestamp;
    let profile_key = init_profile(&mut env, &freelancer);
    let clock_after = env.svm.get_sysvar::<Clock>().unix_timestamp;

    let (expected_pda, expected_bump) = profile_pda(&freelancer_pk);
    assert_eq!(profile_key, expected_pda);

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.authority, freelancer_pk);
    assert_eq!(profile.completed_jobs, 0);
    assert_eq!(profile.successful_jobs, 0);
    assert_eq!(profile.cancelled_jobs, 0);
    assert_eq!(profile.total_earnings, 0);
    assert_eq!(profile.average_rating, 0);
    assert_eq!(profile.reputation_score, 0);
    assert_eq!(profile.badges_earned, 0);
    assert!(profile.created_at >= clock_before && profile.created_at <= clock_after);
    assert_eq!(profile.updated_at, profile.created_at);
    assert_eq!(profile.bump, expected_bump);
}

#[test]
fn test_submit_rating() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &freelancer_pk, 1, 5).unwrap();

    let (rating_key, expected_bump) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.job_id, 1);
    assert_eq!(rating.client, env.client.pubkey());
    assert_eq!(rating.freelancer, freelancer_pk);
    assert_eq!(rating.score, 5);
    assert_eq!(rating.review_hash, [7u8; 32]);
    assert_eq!(rating.bump, expected_bump);

    let (profile_key, _) = profile_pda(&freelancer_pk);
    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.rating_sum, 5);
    assert_eq!(profile.rating_count, 1);
    assert_eq!(profile.average_rating, 500);
}

#[test]
fn test_update_completion_successful() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &freelancer_pk, true, 1_000).unwrap();

    let (profile_key, _) = profile_pda(&freelancer_pk);
    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.completed_jobs, 1);
    assert_eq!(profile.successful_jobs, 1);
    assert_eq!(profile.cancelled_jobs, 0);
    assert_eq!(profile.total_earnings, 1_000);
    assert!(profile.reputation_score > 0);
}

#[test]
fn test_update_completion_cancelled() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &freelancer_pk, false, 0).unwrap();

    let (profile_key, _) = profile_pda(&freelancer_pk);
    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.completed_jobs, 1);
    assert_eq!(profile.successful_jobs, 0);
    assert_eq!(profile.cancelled_jobs, 1);
    assert_eq!(profile.total_earnings, 0);
}

#[test]
fn test_award_badge_first_gig() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    update_completion_for(&mut env, &freelancer_pk, true, 500).unwrap();

    let badge_key = award_badge_for(&mut env, &freelancer_pk, BadgeType::FirstGig).unwrap();

    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.badge_type, BadgeType::FirstGig);
    assert_eq!(badge.issuer, env.authority.pubkey());

    let (profile_key, _) = profile_pda(&freelancer_pk);
    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.badges_earned, 1);
}

#[test]
fn test_reputation_score_improves_after_completion_and_rating() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    let (profile_key, _) = profile_pda(&freelancer_pk);
    let before = read_profile(&env.svm, &profile_key).reputation_score;

    update_completion_for(&mut env, &freelancer_pk, true, 500).unwrap();
    submit_rating_for(&mut env, &freelancer_pk, 42, 5).unwrap();

    let after = read_profile(&env.svm, &profile_key).reputation_score;
    assert!(after > before);
}
