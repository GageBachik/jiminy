use crate::{
    define_instruction_with_metadata,
    state::{Platform, Position, Vote, PLATFORM_SEED, POSITION_SEED},
    utils::calculate_fees,
    PTokenProgramError,
};
use pinocchio::{
    pubkey,
    sysvars::{clock::Clock, Sysvar},
};

define_instruction_with_metadata!(
    discriminant: 3,
    InitializePosition,
    accounts: {
        authority: signer => writable, desc: "Authority of the vault",
        platform: program, desc: "Platform pda key",
        vault: any, desc: "platforms fee vault pda",
        vote: program => writable, desc: "vote account",
        token: token, desc: "vote token",
        vote_vault: any, desc: "votes vault pda",
        vote_vault_token_account: token => writable, desc: "votes token account for storing funds",
        authority_token_account: token => writable, desc: "authorities token account for storing funds",
        vault_token_account: token => writable, desc: "vault token account for storing funds",
        position: uninitialized, desc: "position pda for voting on one side",
    },
    data: {
        amount: [u8; 8],
        side: u8,
    },
    process: {
        // Handle extra security checks here
        // mainly that platform, vault, vote_vault, and position_pda are correct
        let platform_state = load_mut!(platform, Platform);
        let vote_state = load_mut!(vote, Vote);

        // Validate all PDAs at once
        validate_pdas!(
            platform => seeds: [PLATFORM_SEED], bump: platform_state.platform_bump,
                error: PTokenProgramError::PlatformKeyIncorrect;
            vault => seeds: [platform.key().as_ref()], bump: platform_state.vault_bump,
                error: PTokenProgramError::VaultKeyIncorrect;
            vote_vault => seeds: [vote.key().as_ref()], bump: vote_state.vault_bump,
                error: PTokenProgramError::VoteVaultKeyIncorrect
        );

        // cant use derive_address yet for security concerns
        // find the vault PDA
        let (position_pda, position_bump) = pubkey::find_program_address(
            &[
                POSITION_SEED,
                vote.key().as_ref(),
                authority.key().as_ref(),
            ],
            &crate::ID,
        );
        // check that it matches what the user supplied:
        if position.key().ne(&position_pda) {
            return Err(PTokenProgramError::PositionKeyIncorrect.into());
        }

        // Don't let user create or update positions if the vote
        // has already ended
        let now = Clock::get()?.unix_timestamp;
        let vote_deadline = i64::from_le_bytes(vote_state.end_timestamp);
        if now > vote_deadline {
            return Err(PTokenProgramError::VoteHasAlreadyEnded.into());
        }

        // Initialize the position account
        create_pda!(
            from: authority,
            to: position,
            space: Position::LEN,
            seeds: [POSITION_SEED, vote.key().as_ref(), authority.key().as_ref()],
            bump: position_bump
        );

        // Transfer appropriate token and fees
        let init_amount = u64::from_be_bytes(amount);
        let fee_amount = calculate_fees(init_amount, u16::from_le_bytes(platform_state.fee));
        // Initialize the position vault by sending it some tokens
        transfer_tokens!(authority_token_account, vote_vault_token_account, authority, init_amount);
        // Take our fee
        transfer_tokens!(authority_token_account, vault_token_account, authority, fee_amount);

        // lastly set position account data
        with_state!(position, Position, |position_state| {
            position_state.amount = amount;
            position_state.side = side;
            position_state.bump = position_bump;
        });

        if side == 0 {
            vote_state.false_votes =
                (u64::from_le_bytes(vote_state.false_votes) + init_amount).to_le_bytes();
        } else {
            vote_state.true_votes =
                (u64::from_le_bytes(vote_state.true_votes) + init_amount).to_le_bytes();
        }

        Ok(())
    }
);
