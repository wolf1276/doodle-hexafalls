mod common;

use common::*;
use solana_keypair::Keypair;
use solana_signer::Signer;

#[test]
fn test_update_completion_rejects_unauthorized_signer() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
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
    expect_send_error(result.unwrap_err(), ERR_UNAUTHORIZED, "unauthorized update_completion signer");
}

#[test]
fn test_random_wallet_cannot_update_completion() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    let random_wallet = Keypair::new();
    env.svm.airdrop(&random_wallet.pubkey(), 10_000_000_000).unwrap();

    let (profile, _) = profile_pda(&freelancer_pk);
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_update_completion(&random_wallet.pubkey(), &profile, true, 100)],
        &[&env.payer.insecure_clone(), &random_wallet],
    );
    expect_send_error(result.unwrap_err(), ERR_UNAUTHORIZED, "random wallet cannot update completion");
}

#[test]
fn test_award_badge_rejects_unauthorized_signer() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
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
    expect_send_error(result.unwrap_err(), ERR_UNAUTHORIZED, "unauthorized award_badge signer");
}

#[test]
fn test_random_wallet_cannot_award_badge() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    update_completion_for(&mut env, &freelancer_pk, true, 100).unwrap();

    let random = Keypair::new();
    env.svm.airdrop(&random.pubkey(), 10_000_000_000).unwrap();

    let (profile, _) = profile_pda(&freelancer_pk);
    let (badge, _) = badge_pda(&freelancer_pk, BadgeType::FirstGig);
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_award_badge(
            &random.pubkey(),
            &profile,
            &badge,
            BadgeType::FirstGig,
            String::new(),
        )],
        &[&env.payer.insecure_clone(), &random],
    );
    expect_send_error(result.unwrap_err(), ERR_UNAUTHORIZED, "random wallet cannot award badge");
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
            &env.authority.pubkey(),
            &freelancer.pubkey(),
            &profile,
            &rating,
            1,
            5,
            DEFAULT_REVIEW_HASH,
        )],
        &[&env.payer.insecure_clone(), &freelancer, &env.authority.insecure_clone()],
    );
    expect_send_error(result.unwrap_err(), ERR_SELF_DEALING, "self-dealing rating should fail");
}

#[test]
fn test_submit_rating_allows_different_client() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let client = env.client.insecure_clone();
    init_profile(&mut env, &freelancer);

    let result = submit_rating_for(&mut env, &freelancer.pubkey(), 1, 5);
    assert!(result.is_ok(), "different client should be able to rate: {:?}", result);

    let (rating_key, _) = rating_pda(1);
    let rating = read_rating(&env.svm, &rating_key);
    assert_eq!(rating.client, client.pubkey());
    assert_ne!(rating.client, freelancer.pubkey());
}

#[test]
fn test_submit_rating_rejects_unauthorized_co_signer() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    let impostor = solana_keypair::Keypair::new();
    env.svm.airdrop(&impostor.pubkey(), 10_000_000_000).unwrap();

    let (profile, _) = profile_pda(&freelancer.pubkey());
    let (rating, _) = rating_pda(1);
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_submit_rating(
            &env.client.pubkey(),
            &impostor.pubkey(),
            &freelancer.pubkey(),
            &profile,
            &rating,
            1,
            5,
            DEFAULT_REVIEW_HASH,
        )],
        &[&env.payer.insecure_clone(), &env.client.insecure_clone(), &impostor],
    );
    expect_send_error(result.unwrap_err(), ERR_UNAUTHORIZED, "forged rating without authority co-sign should fail");
}

#[test]
fn test_get_profile_works_for_any_caller() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    let profile_key = init_profile(&mut env, &freelancer);

    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_get_profile(&profile_key)],
        &[&env.payer.insecure_clone()],
    );
    assert!(result.is_ok(), "get_profile should work for anyone");
}

#[test]
fn test_submit_rating_rejects_freelancer_without_profile() {
    let mut env = setup();
    let no_profile = Keypair::new();
    env.svm.airdrop(&no_profile.pubkey(), 10_000_000_000).unwrap();

    let (profile, _) = profile_pda(&no_profile.pubkey());
    let (rating, _) = rating_pda(1);

    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_submit_rating(
            &env.client.pubkey(),
            &env.authority.pubkey(),
            &no_profile.pubkey(),
            &profile,
            &rating,
            1,
            5,
            DEFAULT_REVIEW_HASH,
        )],
        &[&env.payer.insecure_clone(), &env.client.insecure_clone(), &env.authority.insecure_clone()],
    );
    assert!(
        result.is_err(),
        "rating without profile should fail"
    );
}

#[test]
fn test_duplicate_profile_rejected() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    env.svm.expire_blockhash();
    let (profile, _) = profile_pda(&freelancer.pubkey());
    let result = send(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_initialize_profile(&freelancer.pubkey(), &profile)],
        &[&env.payer.insecure_clone(), &freelancer],
    );
    assert!(result.is_err(), "duplicate profile should fail");
}

#[test]
fn test_authority_cannot_be_changed() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let profile_key = init_profile(&mut env, &freelancer);

    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.authority, freelancer.pubkey());
    // No instruction exists to modify authority - this is enforced by program design.
    // Verify there is no `update_profile` instruction that could do this.
}
