use shank::ShankInstruction;
use shank::ShankType;
use pinocchio::program_error::ProgramError;

// Generated error enum: PTokenProgramError
#[derive(Clone, PartialEq, ShankType)]
pub enum PTokenProgramError {
    InvalidDiscriminator = 6001,
    PlatformKeyIncorrect = 6002,
    VaultKeyIncorrect = 6003,
    VoteVaultKeyIncorrect = 6004,
    PositionKeyIncorrect = 6005,
    VoteVaultTokenAccountIncorrect = 6006,
    VoteHasAlreadyEnded = 6007,
    VoteIsStillRunning = 6008,
    VoteWasTied = 6009,
    DidNotVoteForWinningSide = 6010,
}

impl From<PTokenProgramError> for ProgramError {
    fn from(e: PTokenProgramError) -> Self {
        Self::Custom(e as u32)
    }
}

#[repr(u8)]
#[derive(Clone, Debug, PartialEq, ShankInstruction)]
pub enum ProgramInstructions {
    #[account(0, signer, writable, name = "authority", desc = "Authority of the vault")]
    #[account(1, writable, name = "platform", desc = "Platform pda key")]
    #[account(2, writable, name = "vault", desc = "platforms fee vault pda")]
    #[account(3, name = "system_program", desc = "System program")]
    InitializePlatform {
        fee: [u8; 2],
        platform_bump: u8,
        vault_bump: u8,
    },

    #[account(0, signer, writable, name = "authority", desc = "Authority of the vault")]
    #[account(1, name = "new_authority", desc = "New authority of the vault")]
    #[account(2, writable, name = "platform", desc = "Platform pda key")]
    #[account(3, name = "vault", desc = "platforms fee vault pda")]
    #[account(4, name = "rent", desc = "Rent program")]
    #[account(5, name = "system_program", desc = "System program")]
    UpdatePlatform {
        new_fee: [u8; 2],
    },

    #[account(0, signer, writable, name = "authority", desc = "Authority of the vault")]
    #[account(1, name = "platform", desc = "Platform pda key")]
    #[account(2, writable, name = "vault", desc = "platforms fee vault pda")]
    #[account(3, signer, writable, name = "vote", desc = "new vote account")]
    #[account(4, name = "token", desc = "vote token")]
    #[account(5, writable, name = "vote_vault", desc = "votes vault pda")]
    #[account(6, writable, name = "vote_vault_token_account", desc = "votes token account for storing funds")]
    #[account(7, name = "rent", desc = "Rent program")]
    #[account(8, name = "system_program", desc = "System program")]
    #[account(9, name = "token_program", desc = "Token program")]
    #[account(10, name = "associated_token_program", desc = "Associated Token program")]
    InitializeVote {
        time_to_add: [u8; 8],
    },

    #[account(0, signer, writable, name = "authority", desc = "Authority of the vault")]
    #[account(1, name = "platform", desc = "Platform pda key")]
    #[account(2, name = "vault", desc = "platforms fee vault pda")]
    #[account(3, writable, name = "vote", desc = "vote account")]
    #[account(4, name = "token", desc = "vote token")]
    #[account(5, name = "vote_vault", desc = "votes vault pda")]
    #[account(6, writable, name = "vote_vault_token_account", desc = "votes token account for storing funds")]
    #[account(7, writable, name = "authority_token_account", desc = "authorities token account for storing funds")]
    #[account(8, writable, name = "vault_token_account", desc = "vault token account for storing funds")]
    #[account(9, writable, name = "position", desc = "position pda for voting on one side")]
    InitializePosition {
        amount: [u8; 8],
        side: u8,
    },

    #[account(0, signer, writable, name = "authority", desc = "Authority of the vault")]
    #[account(1, name = "platform", desc = "Platform pda key")]
    #[account(2, name = "vault", desc = "platforms fee vault pda")]
    #[account(3, writable, name = "vote", desc = "vote account")]
    #[account(4, name = "token", desc = "vote token")]
    #[account(5, writable, name = "vote_vault", desc = "votes vault pda")]
    #[account(6, writable, name = "vote_vault_token_account", desc = "votes token account for storing funds")]
    #[account(7, writable, name = "authority_token_account", desc = "authorities token account for storing funds")]
    #[account(8, writable, name = "vault_token_account", desc = "vault token account for storing funds")]
    #[account(9, writable, name = "position", desc = "position pda for voting on one side")]
    UpdatePosition {
        amount: [u8; 8],
    },

    #[account(0, signer, writable, name = "authority", desc = "Authority of the vault")]
    #[account(1, name = "platform", desc = "Platform pda key")]
    #[account(2, name = "vault", desc = "platforms fee vault pda")]
    #[account(3, writable, name = "vote", desc = "vote account")]
    #[account(4, name = "token", desc = "vote token")]
    #[account(5, name = "vote_vault", desc = "votes vault pda")]
    #[account(6, writable, name = "vote_vault_token_account", desc = "votes token account for storing funds")]
    #[account(7, writable, name = "authority_token_account", desc = "authorities token account for storing funds")]
    #[account(8, writable, name = "vault_token_account", desc = "vault token account for storing funds")]
    #[account(9, writable, name = "position", desc = "position pda for voting on one side")]
    RedeemWinnings {
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
            crate::instructions::InitializePlatformInstruction::try_from((accounts, &instruction_data[1..]))?.process()
        }
        Some(1) => {
            crate::instructions::UpdatePlatformInstruction::try_from((accounts, &instruction_data[1..]))?.process()
        }
        Some(2) => {
            crate::instructions::InitializeVoteInstruction::try_from((accounts, &instruction_data[1..]))?.process()
        }
        Some(3) => {
            crate::instructions::InitializePositionInstruction::try_from((accounts, &instruction_data[1..]))?.process()
        }
        Some(4) => {
            crate::instructions::UpdatePositionInstruction::try_from((accounts, &instruction_data[1..]))?.process()
        }
        Some(5) => {
            crate::instructions::RedeemWinningsInstruction::try_from((accounts, &instruction_data[1..]))?.process()
        }
        _ => Err(PTokenProgramError::InvalidDiscriminator.into()),
    }
}
