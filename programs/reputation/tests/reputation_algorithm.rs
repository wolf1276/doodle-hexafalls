mod common;

use common::*;
use solana_keypair::Keypair;
use solana_signer::Signer;

#[test]
fn test_new_user_score_zero() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.reputation_score, 0);
    assert_eq!(expected_reputation_score(&profile), 0);
}

#[test]
fn test_ideal_profile_score_max() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for _ in 0..200 {
        update_completion_for(&mut env, &pk, true, 5_000_000).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..20u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.reputation_score, 1000);
    assert_eq!(expected_reputation_score(&profile), 1000);
}

#[test]
fn test_score_is_deterministic() {
    let mut env1 = setup();
    let f1 = env1.freelancer.insecure_clone();
    let pk1 = f1.pubkey();
    init_profile(&mut env1, &f1);
    update_completion_for(&mut env1, &pk1, true, 1_000).unwrap();
    submit_rating_for(&mut env1, &pk1, 1, 5).unwrap();
    let score = read_profile(&env1.svm, &profile_pda(&pk1).0).reputation_score;
    assert!(score > 0);

    let mut env2 = setup();
    let f2 = Keypair::new();
    env2.svm.airdrop(&f2.pubkey(), 10_000_000_000).unwrap();
    let pk2 = f2.pubkey();
    init_profile(&mut env2, &f2);
    update_completion_for(&mut env2, &pk2, true, 1_000).unwrap();
    submit_rating_for(&mut env2, &pk2, 1, 5).unwrap();
    let score2 = read_profile(&env2.svm, &profile_pda(&pk2).0).reputation_score;

    assert_eq!(score, score2, "identical inputs must produce identical scores");
}

#[test]
fn test_cancelled_jobs_penalty() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    // Clean profile: 10 completions, all successful
    for _ in 0..10 {
        update_completion_for(&mut env, &pk, true, 1_000).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..5u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }
    let clean_score = read_profile(&env.svm, &profile_key).reputation_score;

    // Build a new profile with cancellations
    let freelancer2 = Keypair::new();
    env.svm.airdrop(&freelancer2.pubkey(), 10_000_000_000).unwrap();
    let pk2 = freelancer2.pubkey();
    let profile_key2 = init_profile(&mut env, &freelancer2);

    for _ in 0..10 {
        update_completion_for(&mut env, &pk2, false, 0).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 5..10u64 {
        submit_rating_for(&mut env, &pk2, i, 5).unwrap();
        env.svm.expire_blockhash();
    }
    let cancelled_score = read_profile(&env.svm, &profile_key2).reputation_score;

    assert!(
        cancelled_score < clean_score,
        "cancelled profile score ({}) should be less than clean profile score ({})",
        cancelled_score,
        clean_score
    );
}

#[test]
fn test_large_earnings_capped() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &pk, true, u64::MAX).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    let expected = expected_reputation_score(&profile);
    assert_eq!(profile.reputation_score, expected);
    assert!(profile.reputation_score <= 1000);
}

#[test]
fn test_many_completions_capped() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    for _ in 0..200 {
        update_completion_for(&mut env, &pk, true, 1_000).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    // volume_score should be capped at 100, despite 200 completions
    let expected = expected_reputation_score(&profile);
    assert_eq!(profile.reputation_score, expected,
        "score should be capped properly even with 200 completions");
}

#[test]
fn test_poor_ratings_low_score() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &pk, true, 1_000).unwrap();
    submit_rating_for(&mut env, &pk, 1, 1).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    let expected = expected_reputation_score(&profile);
    assert_eq!(profile.reputation_score, expected);
    // Score should be well below max (1000) for a 1-star rating
    assert!(profile.reputation_score <= 500,
        "poor rating should result in moderate score, got {}", profile.reputation_score);
}

#[test]
fn test_mixed_scenarios_score_consistency() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    // Mix of successes and cancellations
    for i in 0..10 {
        let successful = i % 3 != 0;
        update_completion_for(
            &mut env,
            &pk,
            successful,
            if successful { 1_000 } else { 0 },
        )
        .unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..5u64 {
        submit_rating_for(&mut env, &pk, i, (i as u8 % 4) + 2).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    let expected = expected_reputation_score(&profile);
    assert_eq!(
        profile.reputation_score, expected,
        "off-chain and on-chain scores must match"
    );
    assert_invariants(&profile);
}

#[test]
fn test_score_does_not_overflow_with_extreme_values() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    // Use large but non-overflowing values
    for _ in 0..1000 {
        update_completion_for(&mut env, &pk, true, 10_000_000).unwrap();
        env.svm.expire_blockhash();
    }
    for i in 0..100u64 {
        submit_rating_for(&mut env, &pk, i, 5).unwrap();
        env.svm.expire_blockhash();
    }

    let profile = read_profile(&env.svm, &profile_key);
    assert!(profile.reputation_score <= 1000);
    assert!(profile.reputation_score > 0);
    assert_eq!(profile.reputation_score, expected_reputation_score(&profile));
}

#[test]
fn test_score_equal_for_identical_inputs() {
    let mut env1 = setup();
    let f1 = env1.freelancer.insecure_clone();
    let pk1 = f1.pubkey();
    init_profile(&mut env1, &f1);
    update_completion_for(&mut env1, &pk1, true, 500).unwrap();
    submit_rating_for(&mut env1, &pk1, 1, 4).unwrap();
    let score1 = read_profile(&env1.svm, &profile_pda(&pk1).0).reputation_score;

    let mut env2 = setup();
    let f2 = Keypair::new();
    env2.svm.airdrop(&f2.pubkey(), 10_000_000_000).unwrap();
    let pk2 = f2.pubkey();
    init_profile(&mut env2, &f2);
    update_completion_for(&mut env2, &pk2, true, 500).unwrap();
    submit_rating_for(&mut env2, &pk2, 1, 4).unwrap();
    let score2 = read_profile(&env2.svm, &profile_pda(&pk2).0).reputation_score;

    assert_eq!(score1, score2, "identical inputs must produce identical scores");
}

#[test]
fn test_zero_earnings_score() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    update_completion_for(&mut env, &pk, true, 0).unwrap();

    let profile = read_profile(&env.svm, &profile_key);
    let expected = expected_reputation_score(&profile);
    assert_eq!(profile.reputation_score, expected);
}
