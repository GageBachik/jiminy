use shank::ShankInstruction;
use shank::ShankType;
use pinocchio::program_error::ProgramError;

// Generated error enum: CounterProgramError
#[derive(Clone, PartialEq, ShankType)]
pub enum CounterProgramError {
    InvalidDiscriminator = 6001,
    Unauthorized = 6002,
    CounterKeyIncorrect = 6003,
    CounterAlreadyInitialized = 6004,
    CounterNotInitialized = 6005,
    CounterUnderflow = 6006,
}

impl From<CounterProgramError> for ProgramError {
    fn from(e: CounterProgramError) -> Self {
        Self::Custom(e as u32)
    }
}

#[repr(u8)]
#[derive(Clone, Debug, PartialEq, ShankInstruction)]
pub enum ProgramInstructions {
    #[account(0, signer, writable, name = "owner", desc = "Owner of the counter")]
    #[account(1, writable, name = "counter", desc = "Counter PDA to be initialized")]
    #[account(2, name = "system_program", desc = "System program")]
    InitializeCounter {
    },

    #[account(0, signer, name = "owner", desc = "Owner of the counter")]
    #[account(1, writable, name = "counter", desc = "Counter PDA to increment")]
    Increment {
    },

    #[account(0, signer, name = "owner", desc = "Owner of the counter")]
    #[account(1, writable, name = "counter", desc = "Counter PDA to decrement")]
    Decrement {
    },

}

// ShankAccount definitions for state structs
// These are generated for IDL compatibility
#[repr(C)]
#[derive(Clone, shank::ShankAccount)]
pub struct Vote {
    pub token: [u8; 32],
    pub true_votes: [u8; 8],
    pub false_votes: [u8; 8],
    pub end_timestamp: [u8; 8],
    pub vault_bump: u8,
}

#[repr(C)]
#[derive(Clone, shank::ShankAccount)]
pub struct Position {
    pub amount: [u8; 8],
    pub side: u8,
    pub bump: u8,
}

pub fn process_instruction(
    program_id: &pinocchio::pubkey::Pubkey,
    accounts: &[pinocchio::account_info::AccountInfo],
    instruction_data: &[u8],
) -> pinocchio::ProgramResult {
    if program_id != &crate::ID {
        return Err(pinocchio::program_error::ProgramError::IncorrectProgramId);
    }

    match instruction_data.first() {
        Some(0) => {
            crate::instructions::InitializeCounterInstruction::try_from((accounts, &instruction_data[1..]))?.process()
        }
        Some(1) => {
            crate::instructions::IncrementInstruction::try_from((accounts, &instruction_data[1..]))?.process()
        }
        Some(2) => {
            crate::instructions::DecrementInstruction::try_from((accounts, &instruction_data[1..]))?.process()
        }
        _ => Err(CounterProgramError::InvalidDiscriminator.into()),
    }
}
