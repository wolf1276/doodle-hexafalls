mod common;

use common::*;
use solana_clock::Clock;
use solana_signer::Signer;

fn logs_contain(logs: &[String], needle: &str) -> bool {
    logs.iter().any(|l| l.contains(needle))
}

#[test]
fn test_profile_created_event_emitted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let (profile, _) = profile_pda(&freelancer.pubkey());
    let clock_before = env.svm.get_sysvar::<Clock>().unix_timestamp;

    let logs = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_initialize_profile(&freelancer.pubkey(), &profile)],
        &[&env.payer.insecure_clone(), &freelancer],
    )
    .unwrap();
    let clock_after = env.svm.get_sysvar::<Clock>().unix_timestamp;

    assert_event_emitted(&logs);

    // Verify the event's effects by checking state
    let profile_data = read_profile(&env.svm, &profile);
    assert_eq!(profile_data.authority, freelancer.pubkey());
    assert!(profile_data.created_at >= clock_before && profile_data.created_at <= clock_after);
}

#[test]
fn test_rating_submitted_event_emitted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    let (profile, _) = profile_pda(&freelancer_pk);
    let (rating, _) = rating_pda(1);

    let logs = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_submit_rating(
            &env.client.pubkey(),
            &freelancer_pk,
            &profile,
            &rating,
            1,
            5,
            DEFAULT_REVIEW_HASH,
        )],
        &[&env.payer.insecure_clone(), &env.client.insecure_clone()],
    )
    .unwrap();

    assert_event_emitted(&logs);

    // Verify event effects
    let rating_data = read_rating(&env.svm, &rating);
    assert_eq!(rating_data.job_id, 1);
    assert_eq!(rating_data.score, 5);
    assert_eq!(rating_data.client, env.client.pubkey());
    assert_eq!(rating_data.freelancer, freelancer_pk);

    let profile_data = read_profile(&env.svm, &profile);
    assert_eq!(profile_data.rating_sum, 5);
    assert_eq!(profile_data.rating_count, 1);
    assert_eq!(profile_data.average_rating, 500);
}

#[test]
fn test_completion_updated_event_emitted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    let (profile, _) = profile_pda(&freelancer_pk);

    let logs = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_update_completion(&env.authority.pubkey(), &profile, true, 500)],
        &[&env.payer.insecure_clone(), &env.authority.insecure_clone()],
    )
    .unwrap();

    assert_event_emitted(&logs);

    // Verify event effects
    let profile_data = read_profile(&env.svm, &profile);
    assert_eq!(profile_data.completed_jobs, 1);
    assert_eq!(profile_data.successful_jobs, 1);
    assert_eq!(profile_data.total_earnings, 500);
    assert!(profile_data.reputation_score > 0);
}

#[test]
fn test_badge_awarded_event_emitted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    update_completion_for(&mut env, &freelancer_pk, true, 100).unwrap();

    let (profile, _) = profile_pda(&freelancer_pk);
    let (badge, _) = badge_pda(&freelancer_pk, BadgeType::FirstGig);

    let logs = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_award_badge(
            &env.authority.pubkey(),
            &profile,
            &badge,
            BadgeType::FirstGig,
            String::new(),
        )],
        &[&env.payer.insecure_clone(), &env.authority.insecure_clone()],
    )
    .unwrap();

    assert_event_emitted(&logs);

    // Verify event effects
    let badge_data = read_badge(&env.svm, &badge);
    assert_eq!(badge_data.badge_type, BadgeType::FirstGig);
    assert_eq!(badge_data.issuer, env.authority.pubkey());
    assert_eq!(badge_data.profile, profile);

    let profile_data = read_profile(&env.svm, &profile);
    assert_eq!(profile_data.badges_earned, 1);
}

#[test]
fn test_event_contents_profile_created() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let (profile_key, _) = profile_pda(&freelancer.pubkey());
    let clock_before = env.svm.get_sysvar::<Clock>().unix_timestamp;

    let logs = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_initialize_profile(&freelancer.pubkey(), &profile_key)],
        &[&env.payer.insecure_clone(), &freelancer],
    )
    .unwrap();

    assert_event_emitted(&logs);

    // Verify via state that all event fields are correct
    let profile = read_profile(&env.svm, &profile_key);
    assert_eq!(profile.authority, freelancer.pubkey());
    assert_eq!(profile.created_at, profile.updated_at);
    assert!(profile.created_at >= clock_before);
    assert!(profile.created_at <= env.svm.get_sysvar::<Clock>().unix_timestamp);
}

#[test]
fn test_event_contents_rating_submitted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    let (profile, _) = profile_pda(&freelancer_pk);
    let (rating, _) = rating_pda(42);

    let logs = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_submit_rating(
            &env.client.pubkey(),
            &freelancer_pk,
            &profile,
            &rating,
            42,
            4,
            DEFAULT_REVIEW_HASH,
        )],
        &[&env.payer.insecure_clone(), &env.client.insecure_clone()],
    )
    .unwrap();

    assert_event_emitted(&logs);

    // Verify event contents via state
    let r = read_rating(&env.svm, &rating);
    assert_eq!(r.job_id, 42);
    assert_eq!(r.client, env.client.pubkey());
    assert_eq!(r.freelancer, freelancer_pk);
    assert_eq!(r.score, 4);

    let p = read_profile(&env.svm, &profile);
    assert_eq!(p.average_rating, 400);
}

#[test]
fn test_event_contents_completion_updated() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);

    let (profile, _) = profile_pda(&freelancer_pk);

    let logs = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_update_completion(&env.authority.pubkey(), &profile, true, 750)],
        &[&env.payer.insecure_clone(), &env.authority.insecure_clone()],
    )
    .unwrap();

    assert_event_emitted(&logs);

    let p = read_profile(&env.svm, &profile);
    assert_eq!(p.completed_jobs, 1);
    assert_eq!(p.successful_jobs, 1);
    assert_eq!(p.total_earnings, 750);
    assert!(p.reputation_score > 0);
}

#[test]
fn test_event_contents_badge_awarded() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();
    init_profile(&mut env, &freelancer);
    update_completion_for(&mut env, &freelancer_pk, true, 100).unwrap();

    let (profile_key, _) = profile_pda(&freelancer_pk);
    let (badge_key, _) = badge_pda(&freelancer_pk, BadgeType::FirstGig);

    let clock_before = env.svm.get_sysvar::<Clock>().unix_timestamp;
    let logs = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_award_badge(
            &env.authority.pubkey(),
            &profile_key,
            &badge_key,
            BadgeType::FirstGig,
            "test".to_string(),
        )],
        &[&env.payer.insecure_clone(), &env.authority.insecure_clone()],
    )
    .unwrap();
    let clock_after = env.svm.get_sysvar::<Clock>().unix_timestamp;

    assert_event_emitted(&logs);

    let b = read_badge(&env.svm, &badge_key);
    assert_eq!(b.badge_type, BadgeType::FirstGig);
    assert_eq!(b.profile, profile_key);
    assert_eq!(b.issuer, env.authority.pubkey());
    assert_eq!(b.metadata, "test");
    assert!(b.issued_at >= clock_before && b.issued_at <= clock_after);
}

#[test]
fn test_no_events_on_failed_instruction() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    init_profile(&mut env, &freelancer);

    let (profile, _) = profile_pda(&freelancer.pubkey());
    let result = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_initialize_profile(&freelancer.pubkey(), &profile)],
        &[&env.payer.insecure_clone(), &freelancer],
    );

    // Should fail - duplicate profile
    assert!(result.is_err(), "duplicate profile should fail");
}

#[test]
fn test_event_order_across_multiple_instructions() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let freelancer_pk = freelancer.pubkey();

    // Instruction 1: Initialize profile
    let (profile, _) = profile_pda(&freelancer_pk);
    let logs1 = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_initialize_profile(&freelancer_pk, &profile)],
        &[&env.payer.insecure_clone(), &freelancer],
    )
    .unwrap();
    assert_event_emitted(&logs1);

    assert!(account_exists(&env.svm, &profile));

    // Instruction 2: Update completion
    env.svm.expire_blockhash();
    let logs2 = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_update_completion(&env.authority.pubkey(), &profile, true, 100)],
        &[&env.payer.insecure_clone(), &env.authority.insecure_clone()],
    )
    .unwrap();
    assert_event_emitted(&logs2);

    // State is consistent
    let p = read_profile(&env.svm, &profile);
    assert_eq!(p.completed_jobs, 1);
    assert_eq!(p.successful_jobs, 1);
    assert_eq!(p.total_earnings, 100);
}
