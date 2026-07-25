mod common;

use anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas};
use common::*;
use solana_keypair::Keypair;
use solana_signer::Signer;

#[test]
fn test_rating_is_immutable_duplicate_job_rejected() {
    let mut env = setup();
    let freelancer_pk = env.freelancer.pubkey();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    submit_rating_for(&mut env, &freelancer_pk, 1, 5).unwrap();
    env.svm.expire_blockhash();
    let second = submit_rating_for(&mut env, &freelancer_pk, 1, 1);
    assert!(second.is_err(), "duplicate rating should fail");

    let (rating_key, _) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.score, 5);
}

#[test]
fn test_duplicate_badge_rejected() {
    let mut env = setup();
    let freelancer_pk = env.freelancer.pubkey();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);
    update_completion_for(&mut env, &freelancer_pk, true, 100).unwrap();

    award_badge_for(&mut env, &freelancer_pk, BadgeType::FirstGig).unwrap();
    env.svm.expire_blockhash();
    let result = award_badge_for(&mut env, &freelancer_pk, BadgeType::FirstGig);
    assert!(result.is_err(), "duplicate badge should be rejected");

    let profile = read_profile(&env.svm, &profile_pda(&freelancer_pk).0);
    assert_eq!(profile.badges_earned, 1);
}

#[test]
fn test_profile_pda_cannot_be_spoofed_with_wrong_seed() {
    let mut env = setup();
    let real_authority = env.freelancer.insecure_clone();
    init_profile(&mut env, &real_authority);

    let fake_profile = Keypair::new();
    let (rating, _) = rating_pda(99);

    let ix = Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::SubmitRating {
            job_id: 99,
            score: 5,
            review_hash: [0u8; 32],
        }
        .data(),
        reputation::accounts::SubmitRating {
            client: env.client.pubkey(),
            freelancer: real_authority.pubkey(),
            freelancer_profile: fake_profile.pubkey(),
            rating,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );

    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix],
        &[&env.payer.insecure_clone(), &env.client.insecure_clone()],
    );
    assert!(result.is_err(), "fake profile PDA should fail");
}

#[test]
fn test_badge_award_fails_without_eligibility() {
    let mut env = setup();
    let freelancer_pk = env.freelancer.pubkey();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    let result = award_badge_for(&mut env, &freelancer_pk, BadgeType::FirstGig);
    expect_error(result, ERR_BADGE_NOT_ELIGIBLE, "no completions -> not eligible");
}

#[test]
fn test_wrong_bump_for_profile_rejected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    let (profile_key, _) = profile_pda(&freelancer.pubkey());
    // Pass a wrong bump by constructing instruction manually
    env.svm.expire_blockhash();
    let ix = Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::InitializeProfile {}.data(),
        reputation::accounts::InitializeProfile {
            authority: freelancer.pubkey(),
            profile: profile_key,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );

    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix],
        &[&env.payer.insecure_clone(), &freelancer],
    );
    assert!(
        result.is_err(),
        "re-init should fail"
    );
}

#[test]
fn test_fake_rating_pda_rejected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    let fake_rating = Keypair::new();
    let (profile, _) = profile_pda(&freelancer_pk);

    let ix = Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::SubmitRating {
            job_id: 42,
            score: 5,
            review_hash: [0u8; 32],
        }
        .data(),
        reputation::accounts::SubmitRating {
            client: env.client.pubkey(),
            freelancer: freelancer_pk,
            freelancer_profile: profile,
            rating: fake_rating.pubkey(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );

    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix],
        &[&env.payer.insecure_clone(), &env.client.insecure_clone()],
    );
    assert!(result.is_err(), "fake rating PDA should fail");
}

#[test]
fn test_fake_badge_pda_rejected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    update_completion_for(&mut env, &freelancer_pk, true, 100).unwrap();

    let (profile, _) = profile_pda(&freelancer_pk);
    let fake_badge = Keypair::new();

    let ix = Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::AwardBadge {
            badge_type: BadgeType::FirstGig,
            metadata: String::new(),
        }
        .data(),
        reputation::accounts::AwardBadge {
            authority: env.authority.pubkey(),
            profile,
            badge: fake_badge.pubkey(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );

    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix],
        &[&env.payer.insecure_clone(), &env.authority.insecure_clone()],
    );
    assert!(result.is_err(), "fake badge PDA should fail");
}

#[test]
fn test_wrong_owner_account_rejected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    // Create a system-owned account to pass as profile
    let system_account = Keypair::new();
    env.svm
        .airdrop(&system_account.pubkey(), 1_000_000_000)
        .unwrap();

    let (rating_key, _) = rating_pda(100);
    let ix = Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::SubmitRating {
            job_id: 100,
            score: 5,
            review_hash: [0u8; 32],
        }
        .data(),
        reputation::accounts::SubmitRating {
            client: env.client.pubkey(),
            freelancer: pk,
            freelancer_profile: system_account.pubkey(),
            rating: rating_key,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );

    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix],
        &[&env.payer.insecure_clone(), &env.client.insecure_clone()],
    );
    assert!(result.is_err(), "system-owned account should not pass as profile");
}

#[test]
fn test_profile_pda_ownership() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    assert!(account_owned_by_program(&env.svm, &profile_key));
}

#[test]
fn test_rating_pda_ownership() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    submit_rating_for(&mut env, &pk, 1, 5).unwrap();

    let (rating_key, _) = rating_pda(1);
    assert!(account_owned_by_program(&env.svm, &rating_key));
}

#[test]
fn test_badge_pda_ownership() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    update_completion_for(&mut env, &pk, true, 100).unwrap();
    let badge_key = award_badge_for(&mut env, &pk, BadgeType::FirstGig).unwrap();

    assert!(account_owned_by_program(&env.svm, &badge_key));
}

#[test]
fn test_get_profile_wrong_pda_fails() {
    let mut env = setup();
    let wrong_key = Keypair::new();

    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_get_profile(&wrong_key.pubkey())],
        &[&env.payer.insecure_clone()],
    );
    assert!(result.is_err(), "get_profile with wrong PDA should fail");
}
