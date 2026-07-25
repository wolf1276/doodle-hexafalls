mod common;

use common::*;
use solana_signer::Signer;

fn logs_contain(logs: &[String], needle: &str) -> bool {
    logs.iter().any(|l| l.contains(needle))
}

#[test]
fn test_profile_created_event_emitted() {
    let mut env = setup();
    let freelancer = env.freelancer.insecure_clone();
    let (profile, _) = profile_pda(&freelancer.pubkey());
    let logs = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_initialize_profile(&freelancer.pubkey(), &profile)],
        &[&env.payer.insecure_clone(), &freelancer],
    )
    .unwrap();
    assert!(logs_contain(&logs, "Program data:"));
}

#[test]
fn test_rating_submitted_and_completion_updated_events_emitted() {
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
            [1u8; 32],
        )],
        &[&env.payer.insecure_clone(), &env.client.insecure_clone()],
    )
    .unwrap();
    assert!(logs_contain(&logs, "Program data:"));

    let logs = send_logs(
        &mut env.svm,
        &env.payer.insecure_clone(),
        &[ix_update_completion(&env.authority.pubkey(), &profile, true, 500)],
        &[&env.payer.insecure_clone(), &env.authority.insecure_clone()],
    )
    .unwrap();
    assert!(logs_contain(&logs, "Program data:"));
}
