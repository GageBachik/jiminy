use crate::{
    define_instruction_with_metadata,
    state::{Platform, Position, Vote, PLATFORM_SEED, POSITION_SEED},
    utils::calculate_fees,
    PTokenProgramError,
};
use pinocchio::sysvars::{clock::Clock, Sysvar};

define_instruction_with_metadata!(
    discriminant: 5,
    RedeemWinnings,
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
        position: program => writable, desc: "position pda for voting on one side",
    },
    data: {},
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

        // Don't let users redeem if the vote is still going on
        let now = Clock::get()?.unix_timestamp;
        let vote_deadline = i64::from_le_bytes(vote_state.end_timestamp);
        // purposely non-inclusive to allow flashloan exploit for learning purposes
        // I should be able to sway the votes and redeem all on the vote deadline.
        if now < vote_deadline {
            return Err(PTokenProgramError::VoteIsStillRunning.into());
        }

        // Redeem winnings

        let voted_true = position_state.side != 0;
        let total_true = u64::from_be_bytes(vote_state.true_votes);
        let total_false = u64::from_le_bytes(vote_state.false_votes);
        let winning_side = if total_true > total_false {
            Some(true)
        } else if total_false > total_true {
            Some(false)
        } else {
            None // it's a tie
        };

        // make sure user voted correctly otherwise they can't redeem.
        if let Some(winner) = winning_side {
            if voted_true != winner {
                return Err(PTokenProgramError::DidNotVoteForWinningSide.into());
            }
        } else {
            return Err(PTokenProgramError::VoteWasTied.into());
        }

        let winning_side = winning_side.unwrap(); // safe now

        let winning_total = if winning_side {
            total_true
        } else {
            total_false
        };
        let losing_total = if winning_side {
            total_false
        } else {
            total_true
        };

        let position_amount = u64::from_le_bytes(position_state.amount);
        let reward = position_amount + (position_amount * losing_total) / winning_total;

        // Transfer appropriate token and fees
        let fee_amount = calculate_fees(reward, u16::from_le_bytes(platform_state.fee));

        // Transfer reward with PDA signing
        let bump = [vote_state.vault_bump];
        transfer_tokens!(vote_vault_token_account, authority_token_account, vote_vault, reward,
            seeds: [vote.key().as_ref(), &bump]);
        // Take our fee (no signing needed as vote_vault can authorize)
        transfer_tokens!(vote_vault_token_account, vault_token_account, vote_vault, fee_amount,
            seeds: [vote.key().as_ref(), &bump]);

        // lastly close the position account data so it can no longer be redeemed.
        close_account!(position, vault);

        Ok(())
    }
);
