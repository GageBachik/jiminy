#![no_std]
#![allow(unexpected_cfgs)]

use pinocchio::entrypoint;

#[macro_use]
pub mod jiminy;
pub mod instructions;
pub mod state;

// Errors are now generated in generated.rs by the build script

pub use instructions::*;

pinocchio_pubkey::declare_id!("Cntrt7BXEtNAnSo9ecGs9n9KkHGDF73Shr3xqFvsvQTJ");

// Include the generated program code
pub mod generated;
pub use generated::*;

entrypoint!(process_instruction);
