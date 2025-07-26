use crate::{
    define_instruction_with_metadata,
    state::{Platform, PLATFORM_SEED},
    PTokenProgramError,
};

define_instruction_with_metadata!(
    discriminant: 1,
    UpdatePlatform,
    accounts: {
        authority: signer => writable, desc: "Authority of the vault",
        new_authority: any, desc: "New authority of the vault",
        platform: program => writable, desc: "Platform pda key",
        vault: any, desc: "platforms fee vault pda",
        rent: any, desc: "Rent program",
        system_program: any, desc: "System program",
    },
    data: {
        new_fee: [u8; 2],
    },
    process: {
        // Load platform state
        let platform_state = load_mut!(platform, Platform);

        // Validate platform PDA
        assert_pda!(platform, seeds: [PLATFORM_SEED], bump: platform_state.platform_bump,
            error: PTokenProgramError::PlatformKeyIncorrect);

        // Verify current authority
        if platform_state.authority != *authority.key() {
            return Err(pinocchio::program_error::ProgramError::IncorrectAuthority);
        }

        // Update platform state - change authority to new_authority
        platform_state.authority = *new_authority.key();
        platform_state.fee = new_fee;

        Ok(())
    }
);
