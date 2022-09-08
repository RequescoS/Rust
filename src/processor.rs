use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::{clock::Clock, rent::Rent, Sysvar};
use solana_program::{msg, system_instruction};

use crate::error::VoteError;
use crate::instruction::{Direction, VoteInstruction};
use crate::state::{UserVotes, Vote, VoteCounter, VoteStatus};
use crate::{id, SETTINGS_SEED, VOTE_SEED};

pub struct Processor;

pub const MAX_VOTES: u8 = 10;
pub const TIME_TO_LIVE: u64 = 10;

impl Processor {
    pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = VoteInstruction::try_from_slice(input)?;
        match instruction {
            VoteInstruction::Vote { direction } => Self::process_vote(direction, accounts),
            VoteInstruction::CreateVote { vote_seed } => Self::process_create(accounts, vote_seed),
            VoteInstruction::DeleteVote { admin } => Self::process_delete(accounts, admin),
            VoteInstruction::CreateVoteCounter => Self::process_create_counter(accounts),
        }
    }

    fn process_vote(direction: Direction, accounts: &[AccountInfo]) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let user_info = next_account_info(acc_iter)?;
        let participate_info = next_account_info(acc_iter)?;
        let vote_info = next_account_info(acc_iter)?;
        let rent_info = next_account_info(acc_iter)?;
        let system_program_info = next_account_info(acc_iter)?;

        if !user_info.is_signer {
            return Err(VoteError::SignedRequired.into());
        }

        let (participate_pubkey, bump_seed) =
            UserVotes::get_uservote_pubkey_with_bump(user_info.key, vote_info.key);

        if participate_pubkey != *participate_info.key {
            return Err(VoteError::WrongUserVotePDA.into());
        }

        if participate_info.data_is_empty() {
            let participate = UserVotes { is_voted: false };
            let space = participate.try_to_vec()?.len();
            let rent = &Rent::from_account_info(rent_info)?;
            let lamports = rent.minimum_balance(space);
            let signer_seeds: &[&[_]] = &[
                &user_info.key.to_bytes(),
                &vote_info.key.to_bytes(),
                VOTE_SEED.as_bytes(),
                &[bump_seed],
            ];
            invoke_signed(
                &system_instruction::create_account(
                    user_info.key,
                    &participate_pubkey,
                    lamports,
                    space as u64,
                    &id(),
                ),
                &[user_info.clone(), participate_info.clone(), system_program_info.clone()],
                &[signer_seeds],
            )?;
            let _ = participate.serialize(&mut &mut participate_info.data.borrow_mut()[..]);
        }

        let mut participation = UserVotes::try_from_slice(&participate_info.data.borrow())?;
        let mut vote = Vote::try_from_slice(&vote_info.data.borrow())?;

        if participation.is_voted {
            return Err(VoteError::DoubleParticipate.into());
        }

        if vote.status != VoteStatus::Alive {
            return Err(VoteError::CloseVoteParticipate.into());
        }

        participation.is_voted = true;

        match direction {
            Direction::For => vote.all_votes_for += 1,
            Direction::Against => vote.all_votes_against += 1,
        }

        let _ = participation.serialize(&mut &mut participate_info.data.borrow_mut()[..]);
        let _ = vote.serialize(&mut &mut vote_info.data.borrow_mut()[..]);

        Ok(())
    }

    fn process_create(accounts: &[AccountInfo], vote_seed: Pubkey) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let admin_info = next_account_info(acc_iter)?;
        let vote_info = next_account_info(acc_iter)?;
        let vote_counter_info = next_account_info(acc_iter)?;
        let rent_info = next_account_info(acc_iter)?;
        let system_program_info = next_account_info(acc_iter)?;
        let clock_sysvar_info = next_account_info(acc_iter)?;

        let (vote_pubkey, bump_seed) = Vote::get_vote_pubkey_with_bump(&vote_seed);

        if vote_pubkey != *vote_info.key {
            return Err(VoteError::WrongVoteDefine.into());
        }

        if !admin_info.is_signer {
            return Err(VoteError::AdminRequired.into());
        }

        let mut vote_counter = VoteCounter::try_from_slice(&vote_counter_info.data.borrow())?;
        let time = Clock::from_account_info(clock_sysvar_info)?.slot;

        if vote_counter.counter >= MAX_VOTES {
            return Err(VoteError::MaxVote.into());
        }

        let vote = Vote::new(admin_info.key.to_bytes(), time);
        let space = vote.try_to_vec()?.len();
        let rent = &Rent::from_account_info(rent_info)?;
        let lamports = rent.minimum_balance(space);
        let signer_seeds: &[&[_]] = &[&vote_seed.to_bytes(), &[bump_seed]];
        invoke_signed(
            &system_instruction::create_account(
                admin_info.key,
                &vote_pubkey,
                lamports,
                space as u64,
                &id(),
            ),
            &[admin_info.clone(), vote_info.clone(), system_program_info.clone()],
            &[signer_seeds],
        )?;
        vote_counter.counter += 1;
        let _ = vote_counter.serialize(&mut &mut vote_counter_info.data.borrow_mut()[..]);

        if vote.admin != admin_info.key.to_bytes() && vote.admin != [0; 32] {
            return Err(VoteError::AdminRequired.into());
        }

        let _ = vote.serialize(&mut &mut vote_info.data.borrow_mut()[..]);

        Ok(())
    }

    fn process_delete(accounts: &[AccountInfo], admin: [u8; 32]) -> ProgramResult {
        msg!("process_delete: admin={:?}", admin,);
        let acc_iter = &mut accounts.iter();
        let admin_info = next_account_info(acc_iter)?;
        let vote_info = next_account_info(acc_iter)?;
        let vote_counter_info = next_account_info(acc_iter)?;
        let clock_sysvar_info = next_account_info(acc_iter)?;

        if !admin_info.is_signer {
            return Err(VoteError::AdminRequired.into());
        }

        let mut vote = Vote::try_from_slice(&vote_info.data.borrow())?;
        let mut vote_counter = VoteCounter::try_from_slice(&vote_counter_info.data.borrow())?;
        let clock = Clock::from_account_info(clock_sysvar_info)?;

        if vote.admin != admin_info.key.to_bytes() {
            return Err(VoteError::AdminRequired.into());
        }
        msg!("clock.slot: {}, vote.clock: {}", clock.slot, vote.clock);
        if clock.slot - vote.clock >= TIME_TO_LIVE {
            vote.status = VoteStatus::Closed;
            vote_counter.counter -= 1;
        }

        let _ = vote.serialize(&mut &mut vote_info.data.borrow_mut()[..]);
        let _ = vote_counter.serialize(&mut &mut vote_counter_info.data.borrow_mut()[..]);

        msg!("process_delete: done");
        Ok(())
    }

    fn process_create_counter(accounts: &[AccountInfo]) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let admin_info = next_account_info(acc_iter)?;
        let vote_counter_info = next_account_info(acc_iter)?;
        let rent_info = next_account_info(acc_iter)?;
        let system_program_info = next_account_info(acc_iter)?;

        if !admin_info.is_signer {
            return Err(VoteError::AdminRequired.into());
        }

        let (vote_pubkey, bump_seed) = VoteCounter::get_vote_pubkey_with_bump();

        if !vote_counter_info.data_is_empty() {
            return Err(VoteError::DoubleCounter.into());
        }

        let vote_counter = VoteCounter { counter: 0 };
        let space = vote_counter.try_to_vec()?.len();
        let rent = &Rent::from_account_info(rent_info)?;
        let lamports = rent.minimum_balance(space);
        let signer_seeds: &[&[_]] = &[SETTINGS_SEED.as_bytes(), &[bump_seed]];
        invoke_signed(
            &system_instruction::create_account(
                admin_info.key,
                &vote_pubkey,
                lamports,
                space as u64,
                &id(),
            ),
            &[admin_info.clone(), vote_counter_info.clone(), system_program_info.clone()],
            &[signer_seeds],
        )?;

        let _ = vote_counter.serialize(&mut &mut vote_counter_info.data.borrow_mut()[..]);

        Ok(())
    }
}
