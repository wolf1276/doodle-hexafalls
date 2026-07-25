mod common;

use common::*;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1000);
fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[test]
fn test_init_gig_requires_budget_gt_zero() {
    let mut env = setup();
    let id = next_id();
    let (gig, _) = gig_pda(id);
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_initialize_gig(&env.client.pubkey(), &env.mint.pubkey(), &gig, id, TEST_TITLE.to_string(), TEST_DESCRIPTION.to_string(), TEST_CATEGORY.to_string(), 0, TEST_DEADLINE)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_err());
}

#[test]
fn test_init_gig_requires_deadline_far_enough() {
    let mut env = setup();
    let id = next_id();
    let (gig, _) = gig_pda(id);
    let clock = env.svm.get_sysvar::<solana_clock::Clock>();
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_initialize_gig(&env.client.pubkey(), &env.mint.pubkey(), &gig, id, TEST_TITLE.to_string(), TEST_DESCRIPTION.to_string(), TEST_CATEGORY.to_string(), TEST_BUDGET, clock.unix_timestamp)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_err());
}

#[test]
fn test_init_gig_empty_title_fails() {
    let mut env = setup();
    let id = next_id();
    let (gig, _) = gig_pda(id);
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_initialize_gig(&env.client.pubkey(), &env.mint.pubkey(), &gig, id, String::new(), TEST_DESCRIPTION.to_string(), TEST_CATEGORY.to_string(), TEST_BUDGET, TEST_DEADLINE)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_err());
}

#[test]
fn test_init_gig_title_max_len() {
    let mut env = setup();
    let id = next_id();
    let (gig, _) = gig_pda(id);
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_initialize_gig(&env.client.pubkey(), &env.mint.pubkey(), &gig, id, "x".repeat(100), TEST_DESCRIPTION.to_string(), TEST_CATEGORY.to_string(), TEST_BUDGET, TEST_DEADLINE)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_ok());
}

#[test]
fn test_init_gig_title_over_max() {
    let mut env = setup();
    let id = next_id();
    let (gig, _) = gig_pda(id);
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_initialize_gig(&env.client.pubkey(), &env.mint.pubkey(), &gig, id, "x".repeat(101), TEST_DESCRIPTION.to_string(), TEST_CATEGORY.to_string(), TEST_BUDGET, TEST_DEADLINE)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_err());
}

#[test]
fn test_init_gig_desc_over_max() {
    let mut env = setup();
    let id = next_id();
    let (gig, _) = gig_pda(id);
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_initialize_gig(&env.client.pubkey(), &env.mint.pubkey(), &gig, id, TEST_TITLE.to_string(), "x".repeat(501), TEST_CATEGORY.to_string(), TEST_BUDGET, TEST_DEADLINE)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_err());
}

#[test]
fn test_init_gig_category_over_max() {
    let mut env = setup();
    let id = next_id();
    let (gig, _) = gig_pda(id);
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_initialize_gig(&env.client.pubkey(), &env.mint.pubkey(), &gig, id, TEST_TITLE.to_string(), TEST_DESCRIPTION.to_string(), "x".repeat(51), TEST_BUDGET, TEST_DEADLINE)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_err());
}

#[test]
fn test_create_milestone_fails_in_draft() {
    let mut env = setup();
    let id = next_id();
    let gig = init_gig(&mut env, id);
    let (m, _) = milestone_pda(&gig, 0);
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_create_milestone(&env.client.pubkey(), &gig, &m, STANDARD_AMOUNT)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_err());
}

#[test]
fn test_create_milestone_fails_in_published() {
    let mut env = setup();
    let id = next_id();
    let gig = init_gig(&mut env, id);
    publish_gig(&mut env, &gig);
    let (m, _) = milestone_pda(&gig, 0);
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_create_milestone(&env.client.pubkey(), &gig, &m, STANDARD_AMOUNT)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_err());
}

#[test]
fn test_create_milestone_succeeds_in_assigned() {
    let mut env = setup();
    let id = next_id();
    let gig = init_gig(&mut env, id);
    publish_and_assign(&mut env, &gig);
    create_milestone_for(&mut env, &gig, 0, STANDARD_AMOUNT);
    let ms = read_milestone(&env.svm, &milestone_pda(&gig, 0).0);
    assert_eq!(ms.amount, STANDARD_AMOUNT);
}

#[test]
fn test_create_milestone_fails_after_complete() {
    let mut env = setup();
    let id = next_id();
    let gig = init_gig(&mut env, id);
    publish_and_assign(&mut env, &gig);
    complete_gig_for(&mut env, &gig);
    let (m, _) = milestone_pda(&gig, 0);
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_create_milestone(&env.client.pubkey(), &gig, &m, STANDARD_AMOUNT)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_err());
}

#[test]
fn test_create_milestone_fails_after_cancel() {
    let mut env = setup();
    let id = next_id();
    let gig = init_gig(&mut env, id);
    publish_and_assign(&mut env, &gig);
    cancel_gig_for(&mut env, &gig);
    let (m, _) = milestone_pda(&gig, 0);
    let r = send(
        &mut env.svm, &env.payer,
        &[ix_create_milestone(&env.client.pubkey(), &gig, &m, STANDARD_AMOUNT)],
        &[&env.payer, &env.client],
    );
    assert!(r.is_err());
}
