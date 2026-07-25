#![allow(dead_code)]

use anchor_lang::{
    solana_program::{instruction::Instruction, program_pack::Pack, system_instruction},
    InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_clock::Clock;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use spl_token_interface as spl_token;

pub const USDC_DECIMALS: u8 = 6;

pub struct Env {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub client: Keypair,
    pub freelancer: Keypair,
    pub mint: Keypair,
}

pub fn setup() -> Env {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../../target/deploy/escrow.so");
    svm.add_program(escrow::ID, bytes).unwrap();

    let payer = Keypair::new();
    let client = Keypair::new();
    let freelancer = Keypair::new();
    let mint = Keypair::new();

    for kp in [&payer, &client, &freelancer] {
        svm.airdrop(&kp.pubkey(), 10_000_000_000).unwrap();
    }

    create_mint(&mut svm, &payer, &mint, USDC_DECIMALS);

    Env {
        svm,
        payer,
        client,
        freelancer,
        mint,
    }
}

pub fn send(svm: &mut LiteSVM, payer: &Keypair, ixs: &[Instruction], signers: &[&Keypair]) -> Result<(), String> {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let mut all_signers = vec![payer];
    for s in signers {
        if s.pubkey() != payer.pubkey() {
            all_signers.push(s);
        }
    }
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &all_signers).unwrap();
    svm.send_transaction(tx).map(|_| ()).map_err(|e| e.err.to_string())
}

pub fn create_mint(svm: &mut LiteSVM, payer: &Keypair, mint: &Keypair, decimals: u8) {
    let rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN);
    let create_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint.pubkey(),
        rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::ID,
    );
    let init_ix = spl_token::instruction::initialize_mint2(
        &spl_token::ID,
        &mint.pubkey(),
        &payer.pubkey(),
        None,
        decimals,
    )
    .unwrap();
    send(svm, payer, &[create_ix, init_ix], &[payer, mint]).unwrap();
}

pub fn create_token_account(svm: &mut LiteSVM, payer: &Keypair, mint: &Pubkey, owner: &Pubkey) -> Pubkey {
    let account = Keypair::new();
    let rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Account::LEN);
    let create_ix = system_instruction::create_account(
        &payer.pubkey(),
        &account.pubkey(),
        rent,
        spl_token::state::Account::LEN as u64,
        &spl_token::ID,
    );
    let init_ix = spl_token::instruction::initialize_account3(&spl_token::ID, &account.pubkey(), mint, owner)
        .unwrap();
    send(svm, payer, &[create_ix, init_ix], &[payer, &account]).unwrap();
    account.pubkey()
}

pub fn mint_to(svm: &mut LiteSVM, payer: &Keypair, mint: &Pubkey, mint_authority: &Keypair, dest: &Pubkey, amount: u64) {
    let ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        mint,
        dest,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    send(svm, payer, &[ix], &[payer, mint_authority]).unwrap();
}

pub fn token_balance(svm: &LiteSVM, account: &Pubkey) -> u64 {
    let data = svm.get_account(account).unwrap().data;
    spl_token::state::Account::unpack(&data).unwrap().amount
}

pub fn warp_seconds(svm: &mut LiteSVM, seconds: i64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = clock.unix_timestamp.saturating_add(seconds);
    svm.set_sysvar::<Clock>(&clock);
}

pub fn gig_pda(id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"gig", id.to_le_bytes().as_ref()], &escrow::ID)
}

pub fn milestone_pda(gig: &Pubkey, index: u32) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"milestone", gig.as_ref(), index.to_le_bytes().as_ref()], &escrow::ID)
}

pub fn vault_pda(gig: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", gig.as_ref()], &escrow::ID)
}

pub fn vault_token_pda(gig: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", gig.as_ref(), b"token"], &escrow::ID)
}

pub fn ix_initialize_gig(
    client: &Pubkey,
    freelancer: &Pubkey,
    mint: &Pubkey,
    gig: &Pubkey,
    id: u64,
) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::InitializeGig { id }.data(),
        escrow::accounts::InitializeGig {
            client: *client,
            freelancer: *freelancer,
            mint: *mint,
            gig: *gig,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn ix_create_milestone(client: &Pubkey, gig: &Pubkey, milestone: &Pubkey, amount: u64) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::CreateMilestone { amount }.data(),
        escrow::accounts::CreateMilestone {
            client: *client,
            gig: *gig,
            milestone: *milestone,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub struct FundAccounts {
    pub client: Pubkey,
    pub gig: Pubkey,
    pub milestone: Pubkey,
    pub vault: Pubkey,
    pub vault_token_account: Pubkey,
    pub client_token_account: Pubkey,
    pub mint: Pubkey,
}

pub fn ix_fund_milestone(a: &FundAccounts) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::FundMilestone {}.data(),
        escrow::accounts::FundMilestone {
            client: a.client,
            gig: a.gig,
            milestone: a.milestone,
            vault: a.vault,
            vault_token_account: a.vault_token_account,
            client_token_account: a.client_token_account,
            mint: a.mint,
            token_program: spl_token::ID,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn ix_submit_delivery(freelancer: &Pubkey, gig: &Pubkey, milestone: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::SubmitDelivery {}.data(),
        escrow::accounts::SubmitDelivery {
            freelancer: *freelancer,
            gig: *gig,
            milestone: *milestone,
        }
        .to_account_metas(None),
    )
}

pub struct ReleaseAccounts {
    pub client: Pubkey,
    pub gig: Pubkey,
    pub milestone: Pubkey,
    pub vault: Pubkey,
    pub vault_token_account: Pubkey,
    pub freelancer: Pubkey,
    pub freelancer_token_account: Pubkey,
    pub mint: Pubkey,
}

pub fn ix_approve_milestone(a: &ReleaseAccounts) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::ApproveMilestone {}.data(),
        escrow::accounts::ApproveMilestone {
            client: a.client,
            gig: a.gig,
            milestone: a.milestone,
            vault: a.vault,
            vault_token_account: a.vault_token_account,
            freelancer: a.freelancer,
            freelancer_token_account: a.freelancer_token_account,
            mint: a.mint,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
    )
}

pub struct TimeoutAccounts {
    pub gig: Pubkey,
    pub milestone: Pubkey,
    pub vault: Pubkey,
    pub vault_token_account: Pubkey,
    pub freelancer_token_account: Pubkey,
    pub mint: Pubkey,
}

pub fn ix_partial_timeout_release(a: &TimeoutAccounts) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::PartialTimeoutRelease {}.data(),
        escrow::accounts::PartialTimeoutRelease {
            gig: a.gig,
            milestone: a.milestone,
            vault: a.vault,
            vault_token_account: a.vault_token_account,
            freelancer_token_account: a.freelancer_token_account,
            mint: a.mint,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
    )
}

pub fn ix_full_timeout_release(a: &TimeoutAccounts) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::FullTimeoutRelease {}.data(),
        escrow::accounts::FullTimeoutRelease {
            gig: a.gig,
            milestone: a.milestone,
            vault: a.vault,
            vault_token_account: a.vault_token_account,
            freelancer_token_account: a.freelancer_token_account,
            mint: a.mint,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
    )
}

pub fn ix_cancel_before_funding(client: &Pubkey, gig: &Pubkey, milestone: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::CancelBeforeFunding {}.data(),
        escrow::accounts::CancelBeforeFunding {
            client: *client,
            gig: *gig,
            milestone: *milestone,
        }
        .to_account_metas(None),
    )
}
