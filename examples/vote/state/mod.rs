use crate::define_state;

// Seeds
pub const PLATFORM_SEED: &[u8; 6] = b"config";
pub const POSITION_SEED: &[u8; 8] = b"position";

define_state! {
    pub struct Platform {
        pub authority: [u8; 32],
        pub fee: [u8; 2],
        pub platform_bump: u8,
        pub vault_bump: u8,
    }

    pub struct Vote {
        pub token: [u8; 32],
        pub true_votes: [u8; 8],
        pub false_votes: [u8; 8],
        pub end_timestamp: [u8; 8],
        pub vault_bump: u8,
    }

    pub struct Position {
        pub amount: [u8; 8],
        pub side: u8,
        pub bump: u8,
    }
}
