use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum VoteError {
    #[error("User signature is required")]
    SignedRequired,

    #[error("Admin signature is required")]
    AdminRequired,

    #[error("Trying to create second vote counted")]
    DoubleCounter,

    #[error("Trying to create new vote when max count of votes already created")]
    MaxVote,

    #[error("Trying to define non-existed vote")]
    WrongVoteDefine,

    #[error("Trying to double participate in single vote")]
    DoubleParticipate,

    #[error("Trying to participate in closed vote")]
    CloseVoteParticipate,

    #[error("Wrong UserVote PDA")]
    WrongUserVotePDA,

    #[error("Wrong settings PDA")]
    WrongSettingsPDA,
}

impl From<VoteError> for ProgramError {
    fn from(e: VoteError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
