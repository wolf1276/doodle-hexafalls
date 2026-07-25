mod common;

use common::*;
use solana_signer::Signer;

#[test]
fn test_rating_out_of_range_rejected() {
    let mut env = setup();
    let freelancer_pk = env.freelancer.pubkey();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    assert!(submit_rating_for(&mut env, &freelancer_pk, 1, 0).is_err());
    assert!(submit_rating_for(&mut env, &freelancer_pk, 2, 6).is_err());
    assert!(submit_rating_for(&mut env, &freelancer_pk, 3, 5).is_ok());
}

#[test]
fn test_average_rating_across_multiple_submissions() {
    let mut env = setup();
    let freelancer_pk = env.freelancer.pubkey();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &freelancer_pk, 1, 5).unwrap();
    submit_rating_for(&mut env, &freelancer_pk, 2, 3).unwrap();
    submit_rating_for(&mut env, &freelancer_pk, 3, 4).unwrap();

    let (profile_key, _) = profile_pda(&freelancer_pk);
    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.rating_count, 3);
    assert_eq!(profile.rating_sum, 12);
    // (5 + 3 + 4) / 3 = 4.00 -> scaled by 100 = 400
    assert_eq!(profile.average_rating, 400);
}

#[test]
fn test_reputation_score_does_not_overflow_with_large_earnings() {
    let mut env = setup();
    let freelancer_pk = env.freelancer.pubkey();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    let result = update_completion_for(&mut env, &freelancer_pk, true, u64::MAX / 2);
    assert!(result.is_ok());

    let (profile_key, _) = profile_pda(&freelancer_pk);
    let profile = read_profile(&env.svm, &profile_key);
    assert!(profile.reputation_score <= 1000);
}

#[test]
fn test_reputation_score_capped_after_many_completions() {
    let mut env = setup();
    let freelancer_pk = env.freelancer.pubkey();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    for _ in 0..30 {
        update_completion_for(&mut env, &freelancer_pk, true, 10_000).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..10u64 {
        submit_rating_for(&mut env, &freelancer_pk, i, 5).unwrap();
    }

    let (profile_key, _) = profile_pda(&freelancer_pk);
    let profile = read_profile(&env.svm, &profile_key);
    assert!(profile.reputation_score <= 1000);
    assert_eq!(profile.completed_jobs, 30);
}
