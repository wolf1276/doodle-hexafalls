mod common;

use common::*;
use solana_clock::Clock;
use solana_keypair::Keypair;
use solana_signer::Signer;

#[test]
fn test_rating_1_accepted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    assert!(submit_rating_for(&mut env, &freelancer_pk, 1, 1).is_ok());
    let (rating_key, _) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.score, 1);
}

#[test]
fn test_rating_2_accepted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    assert!(submit_rating_for(&mut env, &freelancer_pk, 1, 2).is_ok());
    let (rating_key, _) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.score, 2);
}

#[test]
fn test_rating_3_accepted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    assert!(submit_rating_for(&mut env, &freelancer_pk, 1, 3).is_ok());
    let (rating_key, _) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.score, 3);
}

#[test]
fn test_rating_4_accepted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    assert!(submit_rating_for(&mut env, &freelancer_pk, 1, 4).is_ok());
    let (rating_key, _) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.score, 4);
}

#[test]
fn test_rating_5_accepted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    assert!(submit_rating_for(&mut env, &freelancer_pk, 1, 5).is_ok());
    let (rating_key, _) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.score, 5);
}

#[test]
fn test_rating_0_rejected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    let result = submit_rating_for(&mut env, &freelancer_pk, 1, 0);
    expect_error(result, ERR_INVALID_RATING, "rating 0 should be rejected");
}

#[test]
fn test_rating_6_rejected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    let result = submit_rating_for(&mut env, &freelancer_pk, 1, 6);
    expect_error(result, ERR_INVALID_RATING, "rating 6 should be rejected");
}

#[test]
fn test_rating_255_rejected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    let result = submit_rating_for(&mut env, &freelancer_pk, 1, 255);
    expect_error(result, ERR_INVALID_RATING, "rating 255 should be rejected");
}

#[test]
fn test_duplicate_job_id_rejected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &freelancer_pk, 42, 5).unwrap();
    env.svm.expire_blockhash();
    let result = submit_rating_for(&mut env, &freelancer_pk, 42, 3);
    assert!(result.is_err(), "duplicate job_id rating should fail");
}

#[test]
fn test_rating_is_immutable() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &freelancer_pk, 1, 5).unwrap();
    env.svm.expire_blockhash();
    let (rating_key, _) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.score, 5);

    let result = submit_rating_for(&mut env, &freelancer_pk, 1, 1);
    assert!(result.is_err(), "cannot re-rate same job");
    let rating_after = read_rating(&env.svm, &rating_key);
    assert_eq!(rating_after.score, 5, "original rating preserved");
}

#[test]
fn test_rating_updates_profile_correctly() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &freelancer_pk, 1, 5).unwrap();
    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.rating_sum, 5);
    assert_eq!(profile.rating_count, 1);
    assert_eq!(profile.average_rating, 500);
    assert!(profile.reputation_score > 0);
    assert_invariants(&profile);
}

#[test]
fn test_rating_records_correct_fields() {
    let mut env = setup();
    let clock_before = env.svm.get_sysvar::<Clock>().unix_timestamp;
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let client_pk = env.client.pubkey();
    init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &freelancer_pk, 99, 4).unwrap();
    let clock_after = env.svm.get_sysvar::<Clock>().unix_timestamp;

    let (rating_key, expected_bump) = rating_pda(99);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.job_id, 99);
    assert_eq!(rating.client, client_pk);
    assert_eq!(rating.freelancer, freelancer_pk);
    assert_eq!(rating.score, 4);
    assert_eq!(rating.review_hash, DEFAULT_REVIEW_HASH);
    assert!(rating.submitted_at >= clock_before && rating.submitted_at <= clock_after);
    assert_eq!(rating.bump, expected_bump);
}

#[test]
fn test_multiple_ratings_different_jobs() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for job_id in 0..10 {
        submit_rating_for(&mut env, &freelancer_pk, job_id, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.rating_count, 10);
    assert_eq!(profile.rating_sum, 50);
    assert_eq!(profile.average_rating, 500);
    assert_invariants(&profile);
}

#[test]
fn test_rating_with_different_review_hashes() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    let client = env.client.insecure_clone();
    submit_rating_as(&mut env, &client, &freelancer_pk, 1, 5, ALT_REVIEW_HASH_1).unwrap();
    let (rating_key, _) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.review_hash, ALT_REVIEW_HASH_1);
}

#[test]
fn test_rating_rejects_nonexistent_profile() {
    let mut env = setup();
    let no_freelancer = Keypair::new();
    env.svm.airdrop(&no_freelancer.pubkey(), 10_000_000_000).unwrap();

    let (profile, _) = profile_pda(&no_freelancer.pubkey());
    let (rating, _) = rating_pda(1);
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_submit_rating(
            &env.client.pubkey(),
            &env.authority.pubkey(),
            &no_freelancer.pubkey(),
            &profile,
            &rating,
            1,
            5,
            DEFAULT_REVIEW_HASH,
        )],
        &[&env.payer.insecure_clone(), &env.client.insecure_clone(), &env.authority.insecure_clone()],
    );
    assert!(result.is_err(), "rating without profile should fail");
}

#[test]
fn test_rating_multiple_freelancers_independent() {
    let mut env = setup();
    let f1 = env.freelancer.insecure_clone();
    let f2 = Keypair::new();
    env.svm.airdrop(&f2.pubkey(), 10_000_000_000).unwrap();

    let f1_pda = init_profile(&mut env, &f1);
    let f2_pda = init_profile(&mut env, &f2);

    submit_rating_for(&mut env, &f1.pubkey(), 1, 5).unwrap();
    submit_rating_for(&mut env, &f2.pubkey(), 2, 3).unwrap();

    let p1 = read_profile(&env.svm, &f1_pda);
    let p2 = read_profile(&env.svm, &f2_pda);
    assert_eq!(p1.rating_sum, 5);
    assert_eq!(p1.average_rating, 500);
    assert_eq!(p2.rating_sum, 3);
    assert_eq!(p2.average_rating, 300);
}

#[test]
fn test_rating_timestamps_updated() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    warp_seconds(&mut env.svm, 3600);
    submit_rating_for(&mut env, &freelancer_pk, 1, 4).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    assert!(profile.updated_at > profile.created_at);
}
