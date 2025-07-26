use crate::{
    define_instruction_with_metadata,
    state::{Platform, Position, Vote, PLATFORM_SEED, POSITION_SEED},
    utils::calculate_fees,
    PTokenProgramError,
};
use pinocchio::sysvars::{clock::Clock, Sysvar};

define_instruction_with_metadata!(
    discriminant: 4,
    UpdatePosition,
    accounts: {
        authority: signer => writable, desc: "Authority of the vault",
        platform: any, desc: "Platform pda key",
        vault: any, desc: "platforms fee vault pda",
        vote: any => writable, desc: "vote account",
        token: any, desc: "vote token",
        vote_vault: any => writable, desc: "votes vault pda",
        vote_vault_token_account: any => writable, desc: "votes token account for storing funds",
        authority_token_account: any => writable, desc: "authorities token account for storing funds",
        vault_token_account: any => writable, desc: "vault token account for storing funds",
        position: any => writable, desc: "position pda for voting on one side",
    },
    data: {
        amount: [u8; 8],
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

        let position_state = load_mut!(position, Position);
        // Validate position PDA
        assert_pda!(position,
            seeds: [POSITION_SEED, vote.key().as_ref(), authority.key().as_ref()],
            bump: position_state.bump,
            error: PTokenProgramError::PositionKeyIncorrect);

        // Don't let user create or update positions if the vote
        // has already ended
        let now = Clock::get()?.unix_timestamp;
        let vote_deadline = i64::from_le_bytes(vote_state.end_timestamp);
        if now > vote_deadline {
            return Err(PTokenProgramError::VoteHasAlreadyEnded.into());
        }

        // Transfer appropriate token and fees
        let update_amount = u64::from_be_bytes(amount);
        let fee_amount = calculate_fees(update_amount, u16::from_le_bytes(platform_state.fee));
        // Transfer tokens to vote vault
        transfer_tokens!(authority_token_account, vote_vault_token_account, authority, update_amount);
        // Take our fee
        transfer_tokens!(authority_token_account, vault_token_account, authority, fee_amount);

        position_state.amount =
            (u64::from_be_bytes(position_state.amount) + update_amount).to_be_bytes();

        if position_state.side == 0 {
            vote_state.false_votes =
                (u64::from_le_bytes(vote_state.false_votes) + update_amount).to_le_bytes();
        } else {
            vote_state.true_votes =
                (u64::from_le_bytes(vote_state.true_votes) + update_amount).to_le_bytes();
        }

        Ok(())
    }
);
