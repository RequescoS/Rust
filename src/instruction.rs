use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    id,
    state::{UserVotes, Vote, VoteCounter},
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum VoteInstruction {
    /// Participate in vote.
    /// Accounts:
    /// 0. `[signer]` want to vote
    /// 1. `[writable]` contain info about vote that this user participate in, PDA
    /// 2. `[]` concrete vote, PDA
    /// 3. `[]` Rent sysvar
    /// 4. `[]` System program
    Vote { direction: Direction },

    /// Create a vote.
    /// Accounts:
    /// 0. `[signer]` admin
    /// 1. '[writable]' vote to create, PDA
    /// 2. '[writable]' vote counter, PDA
    /// 3. `[]` Rent sysvar, PDA
    /// 4. `[]` System program, PDA
    /// 5. '[]' Clock, PDA
    CreateVote { vote_seed: Pubkey },

    /// Delete a vote.
    /// Accounts:
    /// 0. `[signer]` admin
    /// 1. `[writable]` vote to delete, PDA
    /// 2. '[writable]' vote counter, PDA
    /// 3. '[]' Clock, PDA
    DeleteVote { admin: [u8; 32] },

    /// Create vote counter.
    /// Accounts:
    /// 0. `[signer]` admin
    /// 2. '[writable]' vote counter, PDA
    /// 3. `[]` Rent sysvar, PDA
    /// 4. `[]` System program, PDA
    CreateVoteCounter,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum Direction {
    For,
    Against,
}

impl VoteInstruction {
    pub fn delete(admin: &Pubkey, vote: &Pubkey) -> Instruction {
        let (vote_counter_pubkey, _) = VoteCounter::get_vote_pubkey_with_bump();
        Instruction::new_with_borsh(
            id(),
            &VoteInstruction::DeleteVote { admin: admin.to_bytes() },
            vec![
                AccountMeta::new_readonly(*admin, true),
                AccountMeta::new(*vote, false),
                AccountMeta::new(vote_counter_pubkey, false),
                AccountMeta::new_readonly(sysvar::clock::id(), false),
            ],
        )
    }

    pub fn vote(user: &Pubkey, vote: &Pubkey, direction: Direction) -> Instruction {
        let user_votes_pubkey = UserVotes::get_uservote_pubkey(user, vote);
        Instruction::new_with_borsh(
            id(),
            &VoteInstruction::Vote { direction: (direction) },
            vec![
                AccountMeta::new_readonly(*user, true),
                AccountMeta::new(user_votes_pubkey, false),
                AccountMeta::new(*vote, false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )
    }

    pub fn create_vote_counter(admin: &Pubkey) -> Instruction {
        let (vote_counter_pubkey, _) = VoteCounter::get_vote_pubkey_with_bump();
        Instruction::new_with_borsh(
            id(),
            &VoteInstruction::CreateVoteCounter,
            vec![
                AccountMeta::new(*admin, true),
                AccountMeta::new(vote_counter_pubkey, false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )
    }

    pub fn create_vote(admin: &Pubkey, vote_seed: &Pubkey) -> Instruction {
        let (vote_pubkey, _) = Vote::get_vote_pubkey_with_bump(vote_seed);
        let (vote_counter_pubkey, _) = VoteCounter::get_vote_pubkey_with_bump();
        Instruction::new_with_borsh(
            id(),
            &VoteInstruction::CreateVote { vote_seed: *vote_seed },
            vec![
                AccountMeta::new(*admin, true),
                AccountMeta::new(vote_pubkey, false),
                AccountMeta::new(vote_counter_pubkey, false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(sysvar::clock::id(), false),
            ],
        )
    }
}
