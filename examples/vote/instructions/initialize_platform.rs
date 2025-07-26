use crate::{
    define_instruction_with_metadata,
    state::{Platform, PLATFORM_SEED},
};

define_instruction_with_metadata!(
    discriminant: 0,
    InitializePlatform,
    accounts: {
        authority: signer => writable, desc: "Authority of the vault",
        platform: uninitialized, desc: "Platform pda key",
        vault: any => writable, desc: "platforms fee vault pda",
        system_program: any, desc: "System program",
    },
    data: {
        fee: [u8; 2],
        platform_bump: u8,
        vault_bump: u8,
    },
    process: {
        // Create platform account
        create_pda!(
            from: authority,
            to: platform,
            space: Platform::LEN,
            seeds: [PLATFORM_SEED],
            bump: platform_bump
        );

        // Initialize platform state
        with_state!(platform, Platform, |state| {
            state.authority = *authority.key();
            state.fee = fee;
            state.platform_bump = platform_bump;
            state.vault_bump = vault_bump;
        });

        // Initialize vault
        transfer_sol!(authority, vault, (0.01 * 1e9) as u64);

        Ok(())
    }
);
