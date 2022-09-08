pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

pub const VOTE_SEED: &str = "vote";
pub const SETTINGS_SEED: &str = "settings";
solana_program::declare_id!("78yZvMzqAFzSHJrLNVWfqLRFFQ5ZCGzNXB4PBxmp6z5Y");
