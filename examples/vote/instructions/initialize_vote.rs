use crate::{
    define_instruction_with_metadata,
    state::{Platform, Vote, PLATFORM_SEED},
    utils::calculate_fees,
    PTokenProgramError,
};
use pinocchio::{
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    pubkey,
    sysvars::{clock::Clock, Sysvar},
};
use pinocchio_log::log;

define_instruction_with_metadata!(
    discriminant: 2,
    InitializeVote,
    accounts: {
        authority: signer => writable, desc: "Authority of the vault",
        platform: program, desc: "Platform pda key",
        vault: any => writable, desc: "platforms fee vault pda",
        vote: signer => writable, desc: "new vote account",
        token: token, desc: "vote token",
        vote_vault: any => writable, desc: "votes vault pda",
        vote_vault_token_account: uninitialized, desc: "votes token account for storing funds",
        rent: any, desc: "Rent program",
        system_program: any, desc: "System program",
        token_program: any, desc: "Token program",
        associated_token_program: any, desc: "Associated Token program",
    },
    data: {
        time_to_add: [u8; 8],
    },
    process: {

        // Handle extra checks here
        // mainly that platform, vault, and vote_vault are correct
        let platform_state = load_mut!(platform, Platform);
        assert_pda!(platform, seeds: [PLATFORM_SEED], bump: platform_state.platform_bump,
            error: PTokenProgramError::PlatformKeyIncorrect);
        // cant use derive_address yet for security concerns
        // find the vault PDA
        let (vote_vault_pda, vote_vault_bump) =
            pubkey::find_program_address(&[vote.key().as_ref()], &crate::ID);
        // check that it matches what the user supplied:
        if vote_vault.key().ne(&vote_vault_pda) {
            return Err(PTokenProgramError::VoteVaultKeyIncorrect.into());
        }
        // make sure the token account is correct for the vault and then make it
        let (vote_vault_token_account_pda, _vote_vault_token_account_bump) =
            pubkey::find_program_address(
                &[
                    vote_vault.key().as_ref(),
                    pinocchio_token::ID.as_ref(),
                    token.key().as_ref(),
                ],
                &pinocchio_associated_token_account::ID,
            );
        // check that it matches what the user supplied:
        if vote_vault_token_account
            .key()
            .ne(&vote_vault_token_account_pda)
        {
            return Err(PTokenProgramError::VoteVaultTokenAccountIncorrect.into());
        }

        // Initialize the vote account using create_pda macro
        // Note: vote is a signer account, so we can't use create_pda here
        // Keep the manual CreateAccount for signer accounts
        pinocchio_system::instructions::CreateAccount {
            from: authority,
            to: vote,
            space: Vote::LEN as u64,
            lamports: pinocchio::sysvars::rent::Rent::get()?.minimum_balance(Vote::LEN),
            owner: &crate::ID,
        }
        .invoke()?;
        log!("the vote account was made");

        let create_ata_account_infos = [
            authority,
            vote_vault_token_account,
            vote_vault,
            token,
            system_program,
            token_program,
            associated_token_program,
        ];
        let create_ata_account_metas = [
            AccountMeta::new(authority.key(), true, true),
            AccountMeta::new(vote_vault_token_account.key(), true, false),
            AccountMeta::readonly(vote_vault.key()),
            AccountMeta::readonly(token.key()),
            AccountMeta::readonly(system_program.key()),
            AccountMeta::readonly(token_program.key()),
        ];
        let create_ata_ix = Instruction {
            program_id: &pinocchio_associated_token_account::ID,
            accounts: &create_ata_account_metas,
            data: &[0]
        };

        invoke(&create_ata_ix, &create_ata_account_infos)?;
        log!("the ata was made");

        // set vote account data
        with_state!(vote, Vote, |vote_state| {
            vote_state.token = *token.key();
            vote_state.vault_bump = vote_vault_bump;
            // get the current timestamp onchain and add however long the user wants for the vote to it.
            // dont let the user arbitratily choose a timestamp for safety.
            vote_state.end_timestamp = (i64::from_le_bytes(time_to_add)
                + Clock::get()?.unix_timestamp)
                .to_be_bytes();
        });

        let init_sol = (0.01 * 1e9) as u64;
        let fee_sol = calculate_fees(init_sol, u16::from_le_bytes(platform_state.fee));
        // Initialize the vote vault by sending it some sol
        transfer_sol!(authority, vote_vault, init_sol);
        // Take our fee
        transfer_sol!(authority, vault, fee_sol);

        Ok(())
    }
);
