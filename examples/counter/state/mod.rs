use crate::define_state;

// Seeds
pub const COUNTER_SEED: &[u8; 7] = b"counter";

define_state! {
    pub struct Counter {
        pub owner: [u8; 32],
        pub count: [u8; 8],
        pub bump: u8,
    }
}