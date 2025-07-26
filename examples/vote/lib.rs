#![no_std]
#![allow(unexpected_cfgs)]

use pinocchio::entrypoint;

#[macro_use]
pub mod jiminy;
pub mod instructions;
pub mod state;
pub mod utils;

// Errors are now generated in generated.rs by the build script

pub use instructions::*;

pinocchio_pubkey::declare_id!("pVoTew8KNhq6rsrYq9jEUzKypytaLtQR62UbagWTCvu");

// Include the generated program code
pub mod generated;
pub use generated::*;


entrypoint!(process_instruction);
