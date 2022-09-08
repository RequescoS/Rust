#![cfg(feature = "test-bpf")]
use std::assert_eq;

use borsh::{BorshDeserialize};
use solana_program::{
    pubkey::Pubkey,
    system_instruction,
};
use solana_program_test::{
    processor,
    tokio::{
        self,
    },
    ProgramTest, ProgramTestContext,
};

use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use voting::state::{UserVotes, Vote, VoteCounter, VoteStatus};
use voting::{
    entrypoint::process_instruction,
    id,
    instruction::{Direction, VoteInstruction},
};

struct Env {
    ctx: ProgramTestContext,
    admin: Keypair,
    user_01: Keypair,
    user_02: Keypair,
    user_03: Keypair,
}

impl Env {
    async fn new() -> Self {
        let program_test = ProgramTest::new("voting", id(), processor!(process_instruction));
        let mut ctx = program_test.start_with_context().await;

        let admin = Keypair::new();
        let user_01 = Keypair::new();
        let user_02 = Keypair::new();
        let user_03 = Keypair::new();

        ctx.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[
                    system_instruction::transfer(
                        &ctx.payer.pubkey(),
                        &admin.pubkey(),
                        1_000_000_000,
                    ),
                    system_instruction::transfer(
                        &ctx.payer.pubkey(),
                        &user_01.pubkey(),
                        1_000_000_000,
                    ),
                    system_instruction::transfer(
                        &ctx.payer.pubkey(),
                        &user_02.pubkey(),
                        1_000_000_000,
                    ),
                    system_instruction::transfer(
                        &ctx.payer.pubkey(),
                        &user_03.pubkey(),
                        1_000_000_000,
                    ),
                ],
                Some(&ctx.payer.pubkey()),
                &[&ctx.payer],
                ctx.last_blockhash,
            ))
            .await
            .unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[VoteInstruction::create_vote_counter(&admin.pubkey())],
            Some(&admin.pubkey()),
            &[&admin],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();

        let acc =
            ctx.banks_client.get_account(VoteCounter::get_vote_pubkey()).await.unwrap().unwrap();
        let vote_counter = VoteCounter::try_from_slice(acc.data.as_slice()).unwrap();
        assert_eq!(vote_counter.counter, 0);

        Env { ctx, admin, user_01, user_02, user_03 }
    }
}

// test of 1 user vote
#[tokio::test]
async fn test_vote() {
    let mut env = Env::new().await;
    let vote_seed = Pubkey::new_unique();
    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::create_vote(&env.admin.pubkey(), &vote_seed)],
        Some(&env.admin.pubkey()),
        &[&env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::vote(
            &env.user_01.pubkey(),
            &Vote::get_vote_pubkey(&vote_seed),
            Direction::For,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01],
        env.ctx.last_blockhash,
    );
    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc = env
        .ctx
        .banks_client
        .get_account(UserVotes::get_uservote_pubkey(
            &env.user_01.pubkey(),
            &Vote::get_vote_pubkey(&vote_seed),
        ))
        .await
        .unwrap()
        .unwrap();
    let user_votes = UserVotes::try_from_slice(&acc.data).unwrap();

    let acc =
        env.ctx.banks_client.get_account(Vote::get_vote_pubkey(&vote_seed)).await.unwrap().unwrap();

    let vote = Vote::try_from_slice(acc.data.as_slice()).unwrap();
    assert_eq!(vote.all_votes_for, 1);
    assert_eq!(user_votes.is_voted, true);
}

// test of 3 users vote
#[tokio::test]
async fn test_vote_third() {
    let mut env = Env::new().await;
    let vote_seed = Pubkey::new_unique();
    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::create_vote(&env.admin.pubkey(), &vote_seed)],
        Some(&env.admin.pubkey()),
        &[&env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::vote(
            &env.user_01.pubkey(),
            &Vote::get_vote_pubkey(&vote_seed),
            Direction::For,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01],
        env.ctx.last_blockhash,
    );
    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::vote(
            &env.user_02.pubkey(),
            &Vote::get_vote_pubkey(&vote_seed),
            Direction::For,
        )],
        Some(&env.user_02.pubkey()),
        &[&env.user_02],
        env.ctx.last_blockhash,
    );
    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::vote(
            &env.user_03.pubkey(),
            &Vote::get_vote_pubkey(&vote_seed),
            Direction::Against,
        )],
        Some(&env.user_03.pubkey()),
        &[&env.user_03],
        env.ctx.last_blockhash,
    );
    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc =
        env.ctx.banks_client.get_account(Vote::get_vote_pubkey(&vote_seed)).await.unwrap().unwrap();

    let vote = Vote::try_from_slice(acc.data.as_slice()).unwrap();
    assert_eq!(vote.all_votes_for, 2);
    assert_eq!(vote.all_votes_against, 1);
}

// test delete with time wait
#[tokio::test]
async fn test_delete() {
    let mut env = Env::new().await;

    let vote_seed = Pubkey::new_unique();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::create_vote(&env.admin.pubkey(), &vote_seed)],
        Some(&env.admin.pubkey()),
        &[&env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    env.ctx.warp_to_slot(11);

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::delete(&env.admin.pubkey(), &Vote::get_vote_pubkey(&vote_seed))],
        Some(&env.admin.pubkey()),
        &[&env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc =
        env.ctx.banks_client.get_account(Vote::get_vote_pubkey(&vote_seed)).await.unwrap().unwrap();
    let vote = Vote::try_from_slice(acc.data.as_slice()).unwrap();

    assert_eq!(vote.status, VoteStatus::Closed);
}

// test delete without time wait
#[tokio::test]
async fn test_delete_no_wait() {
    let mut env = Env::new().await;

    let vote_seed = Pubkey::new_unique();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::create_vote(&env.admin.pubkey(), &vote_seed)],
        Some(&env.admin.pubkey()),
        &[&env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::delete(&env.admin.pubkey(), &Vote::get_vote_pubkey(&vote_seed))],
        Some(&env.admin.pubkey()),
        &[&env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc =
        env.ctx.banks_client.get_account(Vote::get_vote_pubkey(&vote_seed)).await.unwrap().unwrap();
    let vote = Vote::try_from_slice(acc.data.as_slice()).unwrap();

    assert_eq!(vote.status, VoteStatus::Alive);
}

// test user to double participate in 1 vote
#[should_panic]
#[tokio::test]
async fn double_vote() {
    let mut env = Env::new().await;
    let vote_seed = Pubkey::new_unique();
    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::create_vote(&env.admin.pubkey(), &vote_seed)],
        Some(&env.admin.pubkey()),
        &[&env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::vote(
            &env.user_01.pubkey(),
            &Vote::get_vote_pubkey(&vote_seed),
            Direction::For,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01],
        env.ctx.last_blockhash,
    );
    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::vote(
            &env.user_01.pubkey(),
            &Vote::get_vote_pubkey(&vote_seed),
            Direction::Against,
        )],
        Some(&env.user_01.pubkey()),
        &[&env.user_01],
        env.ctx.last_blockhash,
    );
    env.ctx.banks_client.process_transaction(tx).await.unwrap();
}

// test user (not admin) try to delete vote
#[should_panic]
#[tokio::test]
async fn not_admin() {
    let mut env = Env::new().await;
    let vote_seed = Pubkey::new_unique();
    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::create_vote(&env.admin.pubkey(), &vote_seed)],
        Some(&env.admin.pubkey()),
        &[&env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::delete(&env.user_01.pubkey(), &Vote::get_vote_pubkey(&vote_seed))],
        Some(&env.user_01.pubkey()),
        &[&env.user_01],
        env.ctx.last_blockhash,
    );
    env.ctx.banks_client.process_transaction(tx).await.unwrap();
}

// test of vote create
#[tokio::test]
async fn test_create_vote() {
    let mut env = Env::new().await;

    let vote_seed = Pubkey::new_unique();

    let tx = Transaction::new_signed_with_payer(
        &[VoteInstruction::create_vote(&env.admin.pubkey(), &vote_seed)],
        Some(&env.admin.pubkey()),
        &[&env.admin],
        env.ctx.last_blockhash,
    );

    env.ctx.banks_client.process_transaction(tx).await.unwrap();

    let acc =
        env.ctx.banks_client.get_account(Vote::get_vote_pubkey(&vote_seed)).await.unwrap().unwrap();
    let vote = Vote::try_from_slice(acc.data.as_slice()).unwrap();
    assert_eq!(vote.admin, env.admin.pubkey().to_bytes());
}
