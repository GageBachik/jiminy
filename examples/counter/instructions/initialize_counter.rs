use crate::{
    define_instruction_with_metadata,
    state::{Counter, COUNTER_SEED},
    CounterProgramError,
};
use pinocchio::pubkey;

define_instruction_with_metadata!(
    discriminant: 0,
    InitializeCounter,
    accounts: {
        owner: signer => writable, desc: "Owner of the counter",
        counter: uninitialized, desc: "Counter PDA to be initialized",
        system_program: any, desc: "System program",
    },
    data: {},
    process: {
        // Derive the counter PDA
        let (counter_pda, counter_bump) = pubkey::find_program_address(
            &[
                COUNTER_SEED,
                owner.key().as_ref(),
            ],
            &crate::ID,
        );
        
        // Verify the counter PDA matches
        if counter.key().ne(&counter_pda) {
            return Err(CounterProgramError::CounterKeyIncorrect.into());
        }
        
        // Create the counter PDA
        create_pda!(
            from: owner,
            to: counter,
            space: Counter::LEN,
            seeds: [COUNTER_SEED, owner.key().as_ref()],
            bump: counter_bump
        );
        
        // Initialize the counter state
        with_state!(counter, Counter, |counter_state| {
            counter_state.owner = *owner.key();
            counter_state.count = 0u64.to_le_bytes();
            counter_state.bump = counter_bump;
        });
        
        Ok(())
    }
);