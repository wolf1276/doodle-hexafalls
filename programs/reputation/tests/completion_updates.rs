mod common;

use common::*;
use solana_clock::Clock;
use solana_signer::Signer;

#[test]
fn test_successful_completion() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &freelancer_pk, true, 1_000).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.completed_jobs, 1);
    assert_eq!(profile.successful_jobs, 1);
    assert_eq!(profile.cancelled_jobs, 0);
    assert_eq!(profile.total_earnings, 1_000);
    assert!(profile.reputation_score > 0);
    assert_invariants(&profile);
}

#[test]
fn test_cancelled_completion() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &freelancer_pk, false, 0).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.completed_jobs, 1);
    assert_eq!(profile.successful_jobs, 0);
    assert_eq!(profile.cancelled_jobs, 1);
    assert_eq!(profile.total_earnings, 0);
    assert_invariants(&profile);
}

#[test]
fn test_multiple_completions() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for _ in 0..10 {
        update_completion_for(&mut env, &freelancer_pk, true, 500).unwrap();
        env.svm.expire_blockhash();
    }
    for _ in 0..3 {
        update_completion_for(&mut env, &freelancer_pk, false, 0).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.completed_jobs, 13);
    assert_eq!(profile.successful_jobs, 10);
    assert_eq!(profile.cancelled_jobs, 3);
    assert_eq!(profile.total_earnings, 5_000);
    assert_invariants(&profile);
}

#[test]
fn test_large_earnings() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &freelancer_pk, true, u64::MAX / 2).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.total_earnings, u64::MAX / 2);
    assert!(profile.reputation_score <= 1000);
    assert_invariants(&profile);
}

#[test]
fn test_earnings_accumulate() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &freelancer_pk, true, 100).unwrap();
    update_completion_for(&mut env, &freelancer_pk, true, 200).unwrap();
    update_completion_for(&mut env, &freelancer_pk, true, 300).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.total_earnings, 600);
}

#[test]
fn test_cancelled_earnings_not_added() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &freelancer_pk, false, 5000).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.total_earnings, 0);
    assert_eq!(profile.cancelled_jobs, 1);
}

#[test]
fn test_timestamps_update_on_completion() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    let created = read_profile(&env.svm, &profile_key).created_at;

    warp_seconds(&mut env.svm, 3600);
    update_completion_for(&mut env, &freelancer_pk, true, 100).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.created_at, created);
    assert!(profile.updated_at > created);
}

#[test]
fn test_reputation_recalculates_after_completion() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    let after_create = read_profile(&env.svm, &profile_key).reputation_score;

    update_completion_for(&mut env, &freelancer_pk, true, 1000).unwrap();
    let after_one = read_profile(&env.svm, &profile_key).reputation_score;
    assert!(after_one > after_create);

    submit_rating_for(&mut env, &freelancer_pk, 1, 5).unwrap();
    let after_rating = read_profile(&env.svm, &profile_key).reputation_score;
    assert!(after_rating > after_one);
}

#[test]
fn test_many_completions_with_ratings() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for i in 0..50u64 {
        update_completion_for(&mut env, &freelancer_pk, true, 10_000).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..20u64 {
        submit_rating_for(&mut env, &freelancer_pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.completed_jobs, 50);
    assert_eq!(profile.successful_jobs, 50);
    assert_eq!(profile.total_earnings, 500_000);
    assert_eq!(profile.rating_count, 20);
    assert_eq!(profile.average_rating, 500);
    assert!(profile.reputation_score > 0);
    assert!(profile.reputation_score <= 1000);
    assert_invariants(&profile);
}

#[test]
fn test_completed_jobs_never_decrease() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &freelancer_pk, true, 100).unwrap();
    let before = read_profile(&env.svm, &profile_key).completed_jobs;

    update_completion_for(&mut env, &freelancer_pk, true, 200).unwrap();
    let after = read_profile(&env.svm, &profile_key).completed_jobs;
    assert!(after >= before);
}

#[test]
fn test_alternating_success_and_cancellation() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for i in 0..10 {
        let successful = i % 2 == 0;
        update_completion_for(&mut env, &freelancer_pk, successful, if successful { 100 } else { 0 })
            .unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.completed_jobs, 10);
    assert_eq!(profile.successful_jobs, 5);
    assert_eq!(profile.cancelled_jobs, 5);
    assert_invariants(&profile);
}
