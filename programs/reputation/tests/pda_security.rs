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
    let second = submit_rating_for(&mut env, &freelancer_pk, 1, 1);
    assert!(second.is_err());

    // Original rating is untouched.
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
    let result = award_badge_for(&mut env, &freelancer_pk, BadgeType::FirstGig);
    assert!(result.is_err());

    let (profile_key, _) = profile_pda(&freelancer_pk);
    assert_eq!(read_profile(&env.svm, &profile_key).badges_earned, 1);
}

#[test]
fn test_profile_pda_cannot_be_spoofed_with_wrong_seed() {
    let mut env = setup();
    let real_authority = env.freelancer.insecure_clone();
    init_profile(&mut env, &real_authority);

    // Attacker tries to submit a rating against an arbitrary account that is
    // NOT the PDA derived from `freelancer`, i.e. a fabricated "profile".
    let fake_profile = Keypair::new();
    let (rating, _) = rating_pda(99);

    let ix = Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::SubmitRating { job_id: 99, score: 5, review_hash: [0u8; 32] }.data(),
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
    assert!(result.is_err());
}

#[test]
fn test_badge_award_fails_without_eligibility() {
    let mut env = setup();
    let freelancer_pk = env.freelancer.pubkey();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);
    // No completions recorded yet: FirstGig requires >= 1 completed job.
    let result = award_badge_for(&mut env, &freelancer_pk, BadgeType::FirstGig);
    assert!(result.is_err());
}
