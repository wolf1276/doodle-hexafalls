mod common;

use common::*;
use solana_keypair::Keypair;
use solana_signer::Signer;

#[test]
fn test_duplicate_profile_rejected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    let (profile, _) = profile_pda(&freelancer.pubkey());
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_initialize_profile(&freelancer.pubkey(), &profile)],
        &[&env.payer.insecure_clone(), &freelancer],
    );
    assert!(result.is_err());
}

#[test]
fn test_update_completion_rejects_unauthorized_signer() {
    let mut env = setup();
    let freelancer_pk = env.freelancer.pubkey();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    let impostor = Keypair::new();
    env.svm.airdrop(&impostor.pubkey(), 10_000_000_000).unwrap();

    let (profile, _) = profile_pda(&freelancer_pk);
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_update_completion(&impostor.pubkey(), &profile, true, 100)],
        &[&env.payer.insecure_clone(), &impostor],
    );
    assert!(result.is_err());
}

#[test]
fn test_award_badge_rejects_unauthorized_signer() {
    let mut env = setup();
    let freelancer_pk = env.freelancer.pubkey();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);
    update_completion_for(&mut env, &freelancer_pk, true, 100).unwrap();

    let impostor = Keypair::new();
    env.svm.airdrop(&impostor.pubkey(), 10_000_000_000).unwrap();

    let (profile, _) = profile_pda(&freelancer_pk);
    let (badge, _) = badge_pda(&freelancer_pk, BadgeType::FirstGig);
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_award_badge(
            &impostor.pubkey(),
            &profile,
            &badge,
            BadgeType::FirstGig,
            String::new(),
        )],
        &[&env.payer.insecure_clone(), &impostor],
    );
    assert!(result.is_err());
}

#[test]
fn test_submit_rating_rejects_self_dealing() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    let (profile, _) = profile_pda(&freelancer.pubkey());
    let (rating, _) = rating_pda(1);
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_submit_rating(
            &freelancer.pubkey(),
            &freelancer.pubkey(),
            &profile,
            &rating,
            1,
            5,
            [1u8; 32],
        )],
        &[&env.payer.insecure_clone(), &freelancer],
    );
    assert!(result.is_err());
}
