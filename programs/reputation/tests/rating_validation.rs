mod common;

use common::*;
use solana_signer::Signer;

#[test]
fn test_single_rating_average() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &freelancer_pk, 1, 4).unwrap();
    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.rating_count, 1);
    assert_eq!(profile.rating_sum, 4);
    assert_eq!(profile.average_rating, 400);
    assert_eq!(
        profile.average_rating,
        expected_average_rating(profile.rating_sum, profile.rating_count)
    );
}

#[test]
fn test_two_ratings_average() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &freelancer_pk, 1, 5).unwrap();
    submit_rating_for(&mut env, &freelancer_pk, 2, 3).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.rating_count, 2);
    assert_eq!(profile.rating_sum, 8);
    assert_eq!(profile.average_rating, 400);
    assert_eq!(
        profile.average_rating,
        expected_average_rating(profile.rating_sum, profile.rating_count)
    );
}

#[test]
fn test_many_ratings_average() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for i in 0..100u64 {
        submit_rating_for(&mut env, &freelancer_pk, i, 4).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.rating_count, 100);
    assert_eq!(profile.rating_sum, 400);
    assert_eq!(profile.average_rating, 400);
    assert_eq!(
        profile.average_rating,
        expected_average_rating(profile.rating_sum, profile.rating_count)
    );
    assert_invariants(&profile);
}

#[test]
fn test_precision_correctness() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    // 5 + 5 + 4 = 14, / 3 = 4.666... -> scaled: 466
    submit_rating_for(&mut env, &freelancer_pk, 1, 5).unwrap();
    submit_rating_for(&mut env, &freelancer_pk, 2, 5).unwrap();
    submit_rating_for(&mut env, &freelancer_pk, 3, 4).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.rating_sum, 14);
    assert_eq!(profile.rating_count, 3);
    // 14 * 100 / 3 = 466.666... truncated to 466
    assert_eq!(profile.average_rating, 466);
    assert_eq!(
        profile.average_rating,
        expected_average_rating(profile.rating_sum, profile.rating_count)
    );
}

#[test]
fn test_average_converges_to_five_star() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for i in 0..50u64 {
        submit_rating_for(&mut env, &freelancer_pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.average_rating, 500);
}

#[test]
fn test_average_converges_to_one_star() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for i in 0..50u64 {
        submit_rating_for(&mut env, &freelancer_pk, i, 1).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.average_rating, 100);
}

#[test]
fn test_mixed_ratings_average() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    let scores = [5u8, 1, 3, 4, 2, 5, 1, 3, 4, 2];
    let expected_sum: u64 = scores.iter().map(|&s| s as u64).sum();
    let expected_count = scores.len() as u64;
    for (i, &score) in scores.iter().enumerate() {
        submit_rating_for(&mut env, &freelancer_pk, i as u64, score).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.rating_sum, expected_sum);
    assert_eq!(profile.rating_count, expected_count);
    assert_eq!(
        profile.average_rating,
        expected_average_rating(expected_sum, expected_count)
    );
    assert_invariants(&profile);
}

#[test]
fn test_average_is_unchanged_by_duplicate_attempt() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &freelancer_pk, 1, 5).unwrap();
    let before = read_profile(&env.svm, &profile_key);

    env.svm.expire_blockhash();
    let _ = submit_rating_for(&mut env, &freelancer_pk, 1, 1);
    let after = read_profile(&env.svm, &profile_key);

    assert_eq!(before.rating_sum, after.rating_sum);
    assert_eq!(before.rating_count, after.rating_count);
    assert_eq!(before.average_rating, after.average_rating);
}

#[test]
fn test_average_consistency_with_sum_and_count() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for i in 0..25u64 {
        let score = ((i % 5) + 1) as u8;
        submit_rating_for(&mut env, &freelancer_pk, i, score).unwrap();
        env.svm.expire_blockhash();

        let profile = read_profile(&env.svm, &profile_key);
        let computed = expected_average_rating(profile.rating_sum, profile.rating_count);
        assert_eq!(
            profile.average_rating, computed,
            "consistency check failed after {} ratings", i + 1
        );
    }
}
