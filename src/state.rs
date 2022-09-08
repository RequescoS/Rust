use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::{id, SETTINGS_SEED, VOTE_SEED};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserVotes {
    pub is_voted: bool,
}

impl UserVotes {
    pub fn get_uservote_pubkey_with_bump(user: &Pubkey, vote: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&user.to_bytes(), &vote.to_bytes(), VOTE_SEED.as_bytes()],
            &id(),
        )
    }

    pub fn get_uservote_pubkey(user: &Pubkey, vote: &Pubkey) -> Pubkey {
        let (pubkey, _) = Self::get_uservote_pubkey_with_bump(user, vote);
        pubkey
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum VoteStatus {
    Alive,
    Closed,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Vote {
    pub admin: [u8; 32],

    pub all_votes_for: u32,

    pub all_votes_against: u32,

    pub clock: u64,

    pub status: VoteStatus,
}

impl Vote {
    pub fn get_vote_pubkey_with_bump(vote_seed: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[&vote_seed.to_bytes()], &id())
    }

    pub fn get_vote_pubkey(vote_seed: &Pubkey) -> Pubkey {
        let (pubkey, _) = Self::get_vote_pubkey_with_bump(vote_seed);
        pubkey
    }

    pub fn new(admin: [u8; 32], clock: u64) -> Self {
        Self { admin, all_votes_for: 0, all_votes_against: 0, clock, status: VoteStatus::Alive }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VoteCounter {
    pub counter: u8,
}

impl VoteCounter {
    pub fn get_vote_pubkey_with_bump() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[SETTINGS_SEED.as_bytes()], &id())
    }

    pub fn get_vote_pubkey() -> Pubkey {
        let (pubkey, _) = Self::get_vote_pubkey_with_bump();
        pubkey
    }

    pub fn is_ok_vote_pubkey(vote_pubkey: &Pubkey) -> bool {
        let (pubkey, _) = Self::get_vote_pubkey_with_bump();
        pubkey.to_bytes() == vote_pubkey.to_bytes()
    }
}
