#![allow(dead_code)]
#![allow(unused_imports)]

pub mod fixtures;
pub mod helpers;

pub use fixtures::*;
pub use helpers::*;

use anchor_lang::{
    solana_program::instruction::Instruction, AccountDeserialize, InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_clock::Clock;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

pub use reputation::state::*;

pub struct Env {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub client: Keypair,
    pub freelancer: Keypair,
    pub authority: Keypair,
}

pub fn setup() -> Env {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../../target/deploy/reputation.so");
    svm.add_program(reputation::ID, bytes).unwrap();

    let payer = Keypair::new();
    let client = Keypair::new();
    let freelancer = Keypair::new();
    let authority_bytes: [u8; 64] = {
        let json = include_str!("../fixtures/authority-keypair.json");
        let parsed: Vec<u8> = serde_json::from_str(json).unwrap();
        parsed.try_into().unwrap()
    };
    let authority = Keypair::try_from(&authority_bytes[..]).unwrap();

    for kp in [&payer, &client, &freelancer, &authority] {
        svm.airdrop(&kp.pubkey(), 10_000_000_000).unwrap();
    }

    Env { svm, payer, client, freelancer, authority }
}

pub fn send(
    svm: &mut LiteSVM,
    payer: &Keypair,
    ixs: &[Instruction],
    signers: &[&Keypair],
) -> Result<(), String> {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let mut all_signers = vec![payer];
    for s in signers {
        if s.pubkey() != payer.pubkey() {
            all_signers.push(s);
        }
    }
    let tx =
        VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &all_signers).unwrap();
    svm.send_transaction(tx)
        .map(|_| ())
        .map_err(|e| e.err.to_string())
}

pub fn send_logs(
    svm: &mut LiteSVM,
    payer: &Keypair,
    ixs: &[Instruction],
    signers: &[&Keypair],
) -> Result<Vec<String>, String> {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let mut all_signers = vec![payer];
    for s in signers {
        if s.pubkey() != payer.pubkey() {
            all_signers.push(s);
        }
    }
    let tx =
        VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &all_signers).unwrap();
    let meta = svm.send_transaction(tx).map_err(|e| e.err.to_string())?;
    Ok(meta.logs)
}

pub fn warp_seconds(svm: &mut LiteSVM, seconds: i64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = clock.unix_timestamp.saturating_add(seconds);
    svm.set_sysvar::<Clock>(&clock);
}

// ── Account deserialization ──

pub fn read_profile(svm: &LiteSVM, key: &Pubkey) -> UserProfile {
    let data = svm.get_account(key).unwrap().data;
    UserProfile::try_deserialize(&mut &data[..]).unwrap()
}

pub fn read_rating(svm: &LiteSVM, key: &Pubkey) -> Rating {
    let data = svm.get_account(key).unwrap().data;
    Rating::try_deserialize(&mut &data[..]).unwrap()
}

pub fn read_badge(svm: &LiteSVM, key: &Pubkey) -> Badge {
    let data = svm.get_account(key).unwrap().data;
    Badge::try_deserialize(&mut &data[..]).unwrap()
}

pub fn account_exists(svm: &LiteSVM, key: &Pubkey) -> bool {
    svm.get_account(key).is_some()
}

pub fn account_owned_by_program(svm: &LiteSVM, key: &Pubkey) -> bool {
    svm.get_account(key)
        .map(|a| a.owner == reputation::ID)
        .unwrap_or(false)
}

// ── PDA derivation ──

pub fn profile_pda(authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"profile", authority.as_ref()], &reputation::ID)
}

pub fn rating_pda(job_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"rating", job_id.to_le_bytes().as_ref()], &reputation::ID)
}

pub fn badge_pda(authority: &Pubkey, badge_type: BadgeType) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"badge", authority.as_ref(), &[badge_type.as_seed()]],
        &reputation::ID,
    )
}

// ── Instruction builders ──

pub fn ix_initialize_profile(authority: &Pubkey, profile: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::InitializeProfile {}.data(),
        reputation::accounts::InitializeProfile {
            authority: *authority,
            profile: *profile,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn ix_submit_rating(
    client: &Pubkey,
    authority: &Pubkey,
    freelancer: &Pubkey,
    freelancer_profile: &Pubkey,
    rating: &Pubkey,
    job_id: u64,
    score: u8,
    review_hash: [u8; 32],
) -> Instruction {
    Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::SubmitRating { job_id, score, review_hash }.data(),
        reputation::accounts::SubmitRating {
            client: *client,
            authority: *authority,
            freelancer: *freelancer,
            freelancer_profile: *freelancer_profile,
            rating: *rating,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn ix_update_completion(
    authority_pk: &Pubkey,
    profile: &Pubkey,
    successful: bool,
    earnings: u64,
) -> Instruction {
    Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::UpdateCompletion { successful, earnings }.data(),
        reputation::accounts::UpdateCompletion {
            authority: *authority_pk,
            profile: *profile,
        }
        .to_account_metas(None),
    )
}

pub fn ix_award_badge(
    authority_pk: &Pubkey,
    profile: &Pubkey,
    badge: &Pubkey,
    badge_type: BadgeType,
    metadata: String,
) -> Instruction {
    Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::AwardBadge { badge_type, metadata }.data(),
        reputation::accounts::AwardBadge {
            authority: *authority_pk,
            profile: *profile,
            badge: *badge,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn ix_get_profile(profile: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        reputation::ID,
        &reputation::instruction::GetProfile {}.data(),
        reputation::accounts::GetProfile {
            profile: *profile,
        }
        .to_account_metas(None),
    )
}

// ── Test harness helpers ──

pub fn init_profile(env: &mut Env, authority: &Keypair) -> Pubkey {
    let (profile, _) = profile_pda(&authority.pubkey());
    send(
        &mut env.svm,
        &env.payer,
        &[ix_initialize_profile(&authority.pubkey(), &profile)],
        &[&env.payer, authority],
    )
    .unwrap();
    profile
}

pub fn submit_rating_for(
    env: &mut Env,
    freelancer: &Pubkey,
    job_id: u64,
    score: u8,
) -> Result<(), String> {
    let (profile, _) = profile_pda(freelancer);
    let (rating, _) = rating_pda(job_id);
    let signer = env.authority.insecure_clone();
    send(
        &mut env.svm,
        &env.payer,
        &[ix_submit_rating(
            &env.client.pubkey(),
            &env.authority.pubkey(),
            freelancer,
            &profile,
            &rating,
            job_id,
            score,
            DEFAULT_REVIEW_HASH,
        )],
        &[&env.payer, &env.client, &signer],
    )
}

pub fn submit_rating_as(
    env: &mut Env,
    client_key: &Keypair,
    freelancer: &Pubkey,
    job_id: u64,
    score: u8,
    review_hash: [u8; 32],
) -> Result<(), String> {
    let (profile, _) = profile_pda(freelancer);
    let (rating, _) = rating_pda(job_id);
    let signer = env.authority.insecure_clone();
    send(
        &mut env.svm,
        &env.payer,
        &[ix_submit_rating(
            &client_key.pubkey(),
            &env.authority.pubkey(),
            freelancer,
            &profile,
            &rating,
            job_id,
            score,
            review_hash,
        )],
        &[&env.payer, client_key, &signer],
    )
}

pub fn update_completion_for(
    env: &mut Env,
    authority_pk: &Pubkey,
    successful: bool,
    earnings: u64,
) -> Result<(), String> {
    let (profile, _) = profile_pda(authority_pk);
    let signer = env.authority.insecure_clone();
    send(
        &mut env.svm,
        &env.payer,
        &[ix_update_completion(
            &env.authority.pubkey(),
            &profile,
            successful,
            earnings,
        )],
        &[&env.payer, &signer],
    )
}

pub fn award_badge_for(
    env: &mut Env,
    authority_pk: &Pubkey,
    badge_type: BadgeType,
) -> Result<Pubkey, String> {
    award_badge_for_with_metadata(env, authority_pk, badge_type, String::new())
}

pub fn award_badge_for_with_metadata(
    env: &mut Env,
    authority_pk: &Pubkey,
    badge_type: BadgeType,
    metadata: String,
) -> Result<Pubkey, String> {
    let (profile, _) = profile_pda(authority_pk);
    let (badge, _) = badge_pda(authority_pk, badge_type);
    let signer = env.authority.insecure_clone();
    send(
        &mut env.svm,
        &env.payer,
        &[ix_award_badge(
            &env.authority.pubkey(),
            &profile,
            &badge,
            badge_type,
            metadata,
        )],
        &[&env.payer, &signer],
    )?;
    Ok(badge)
}

// ── Error code constants ──

pub const ERR_PROFILE_ALREADY_EXISTS: &str = "0x1770";
pub const ERR_PROFILE_NOT_FOUND: &str = "0x1771";
pub const ERR_INVALID_RATING: &str = "0x1772";
pub const ERR_DUPLICATE_RATING: &str = "0x1773";
pub const ERR_BADGE_ALREADY_OWNED: &str = "0x1774";
pub const ERR_BADGE_NOT_ELIGIBLE: &str = "0x1775";
pub const ERR_UNAUTHORIZED: &str = "0x1776";
pub const ERR_MATH_OVERFLOW: &str = "0x1777";
pub const ERR_SELF_DEALING: &str = "0x1778";
pub const ERR_METADATA_TOO_LONG: &str = "0x177a";

pub const ERR_CONSTRAINT_SEEDS: &str = "0x7d6";
pub const ERR_ACCOUNT_NOT_INITIALIZED: &str = "0xbc4";

pub const DEFAULT_REVIEW_HASH: [u8; 32] = [7u8; 32];
