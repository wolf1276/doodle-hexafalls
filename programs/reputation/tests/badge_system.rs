mod common;

use common::*;
use solana_clock::Clock;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

fn setup_freelancer(env: &mut Env) -> (Keypair, Pubkey) {
    let f = env.freelancer.insecure_clone();
    let pk = f.pubkey();
    init_profile(env, &f);
    (f, pk)
}

#[test]
fn test_first_gig_badge() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    update_completion_for(&mut env, &pk, true, 500).unwrap();

    let badge_key = award_badge_for(&mut env, &pk, BadgeType::FirstGig).unwrap();
    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.badge_type, BadgeType::FirstGig);
    assert_eq!(badge.issuer, env.authority.pubkey());

    let (profile_pda_key, _) = profile_pda(&pk);
    let profile = read_profile(&env.svm, &profile_pda_key);
    assert_eq!(profile.badges_earned, 1);
}

#[test]
fn test_ten_completed_jobs_badge() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    for _ in 0..10 {
        update_completion_for(&mut env, &pk, true, 100).unwrap();
        env.svm.expire_blockhash();
    }

    let badge_key = award_badge_for(&mut env, &pk, BadgeType::TenCompletedJobs).unwrap();
    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.badge_type, BadgeType::TenCompletedJobs);
}

#[test]
fn test_hundred_completed_jobs_badge() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    for _ in 0..100 {
        update_completion_for(&mut env, &pk, true, 100).unwrap();
        env.svm.expire_blockhash();
    }

    let badge_key = award_badge_for(&mut env, &pk, BadgeType::HundredCompletedJobs).unwrap();
    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.badge_type, BadgeType::HundredCompletedJobs);
}

#[test]
fn test_five_star_performer_badge() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    for i in 0..5u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let badge_key = award_badge_for(&mut env, &pk, BadgeType::FiveStarPerformer).unwrap();
    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.badge_type, BadgeType::FiveStarPerformer);
}

#[test]
fn test_top_rated_badge() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    for i in 0..10u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let badge_key = award_badge_for(&mut env, &pk, BadgeType::TopRated).unwrap();
    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.badge_type, BadgeType::TopRated);
}

#[test]
fn test_trusted_freelancer_badge() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);

    let badge_key = award_badge_for(&mut env, &pk, BadgeType::TrustedFreelancer).unwrap();
    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.badge_type, BadgeType::TrustedFreelancer);
}

#[test]
fn test_fast_deliverer_badge() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);

    let badge_key = award_badge_for(&mut env, &pk, BadgeType::FastDeliverer).unwrap();
    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.badge_type, BadgeType::FastDeliverer);
}

#[test]
fn test_duplicate_badge_rejected() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    update_completion_for(&mut env, &pk, true, 100).unwrap();

    award_badge_for(&mut env, &pk, BadgeType::FirstGig).unwrap();
    env.svm.expire_blockhash();
    let result = award_badge_for(&mut env, &pk, BadgeType::FirstGig);
    assert!(result.is_err(), "duplicate badge type should fail");

    let (profile_pda_key, _) = profile_pda(&pk);
    let profile = read_profile(&env.svm, &profile_pda_key);
    assert_eq!(profile.badges_earned, 1);
}

#[test]
fn test_different_badge_types_allowed() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    for _ in 0..100 {
        update_completion_for(&mut env, &pk, true, 100).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..10u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    award_badge_for(&mut env, &pk, BadgeType::FirstGig).unwrap();
    award_badge_for(&mut env, &pk, BadgeType::TenCompletedJobs).unwrap();
    award_badge_for(&mut env, &pk, BadgeType::HundredCompletedJobs).unwrap();
    award_badge_for(&mut env, &pk, BadgeType::FiveStarPerformer).unwrap();
    award_badge_for(&mut env, &pk, BadgeType::TopRated).unwrap();

    let (profile_pda_key, _) = profile_pda(&pk);
    let profile = read_profile(&env.svm, &profile_pda_key);
    assert_eq!(profile.badges_earned, 5);
}

#[test]
fn test_badge_not_eligible_yet() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);

    let result = award_badge_for(&mut env, &pk, BadgeType::FirstGig);
    expect_error(result, ERR_BADGE_NOT_ELIGIBLE, "no completions -> not eligible for FirstGig");
}

#[test]
fn test_badge_eligibility_thresholds() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);

    for _ in 0..9 {
        update_completion_for(&mut env, &pk, true, 100).unwrap();
        env.svm.expire_blockhash();
    }
    let result = award_badge_for(&mut env, &pk, BadgeType::TenCompletedJobs);
    expect_error(result, ERR_BADGE_NOT_ELIGIBLE, "9 completions not enough for TenCompletedJobs");

    update_completion_for(&mut env, &pk, true, 100).unwrap();
    env.svm.expire_blockhash();
    assert!(award_badge_for(&mut env, &pk, BadgeType::TenCompletedJobs).is_ok());
}

#[test]
fn test_badge_pda_correctness() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    update_completion_for(&mut env, &pk, true, 100).unwrap();

    let badge_key = award_badge_for(&mut env, &pk, BadgeType::FirstGig).unwrap();
    let (expected_key, expected_bump) = badge_pda(&pk, BadgeType::FirstGig);
    assert_eq!(badge_key, expected_key);

    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.bump, expected_bump);
}

#[test]
fn test_badge_metadata_recorded() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    update_completion_for(&mut env, &pk, true, 100).unwrap();

    let metadata = "ipfs://QmTestBadge".to_string();
    let badge_key =
        award_badge_for_with_metadata(&mut env, &pk, BadgeType::FirstGig, metadata.clone()).unwrap();

    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.metadata, metadata);
}

#[test]
fn test_badge_metadata_too_long_rejected() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    update_completion_for(&mut env, &pk, true, 100).unwrap();

    let long_metadata = "x".repeat(129);
    let result = award_badge_for_with_metadata(&mut env, &pk, BadgeType::FirstGig, long_metadata);
    expect_error(result, ERR_METADATA_TOO_LONG, "metadata > 128 bytes should fail");
}

#[test]
fn test_badge_timestamps() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    update_completion_for(&mut env, &pk, true, 100).unwrap();

    let clock_before = env.svm.get_sysvar::<Clock>().unix_timestamp;
    let badge_key = award_badge_for(&mut env, &pk, BadgeType::FirstGig).unwrap();
    let clock_after = env.svm.get_sysvar::<Clock>().unix_timestamp;

    let badge = read_badge(&env.svm, &badge_key);
    assert!(badge.issued_at >= clock_before && badge.issued_at <= clock_after);
}

#[test]
fn test_badge_issuer_recorded() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    update_completion_for(&mut env, &pk, true, 100).unwrap();

    let badge_key = award_badge_for(&mut env, &pk, BadgeType::FirstGig).unwrap();
    let (expected_profile_pda, _) = profile_pda(&pk);
    let badge = read_badge(&env.svm, &badge_key);
    assert_eq!(badge.issuer, env.authority.pubkey());
    assert_eq!(badge.profile, expected_profile_pda);
}

#[test]
fn test_five_star_performer_needs_enough_ratings() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);

    for i in 0..4u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }
    let result = award_badge_for(&mut env, &pk, BadgeType::FiveStarPerformer);
    expect_error(result, ERR_BADGE_NOT_ELIGIBLE, "4 ratings not enough for FiveStarPerformer");

    submit_rating_for(&mut env, &pk, 4, 5).unwrap();
    env.svm.expire_blockhash();
    assert!(award_badge_for(&mut env, &pk, BadgeType::FiveStarPerformer).is_ok());
}

#[test]
fn test_badge_count_increments_correctly() {
    let mut env = setup();
    let (freelancer, pk) = setup_freelancer(&mut env);
    for _ in 0..100 {
        update_completion_for(&mut env, &pk, true, 100).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..10u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let (profile_pda_key, _) = profile_pda(&pk);
    let profile_before = read_profile(&env.svm, &profile_pda_key);
    assert_eq!(profile_before.badges_earned, 0);

    award_badge_for(&mut env, &pk, BadgeType::FirstGig).unwrap();
    let profile = read_profile(&env.svm, &profile_pda_key);
    assert_eq!(profile.badges_earned, 1);

    award_badge_for(&mut env, &pk, BadgeType::TenCompletedJobs).unwrap();
    let profile = read_profile(&env.svm, &profile_pda_key);
    assert_eq!(profile.badges_earned, 2);
}
