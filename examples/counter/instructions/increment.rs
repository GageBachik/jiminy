use crate::{
    define_instruction_with_metadata,
    state::{Counter, COUNTER_SEED},
    CounterProgramError,
};

define_instruction_with_metadata!(
    discriminant: 1,
    Increment,
    accounts: {
        owner: signer, desc: "Owner of the counter",
        counter: program => writable, desc: "Counter PDA to increment",
    },
    data: {},
    process: {
        // Load the counter state
        let counter_state = load_mut!(counter, Counter);
        
        // Verify the owner
        if counter_state.owner != *owner.key() {
            return Err(CounterProgramError::Unauthorized.into());
        }
        
        // Validate the PDA
        assert_pda!(counter,
            seeds: [COUNTER_SEED, owner.key().as_ref()],
            bump: counter_state.bump,
            error: CounterProgramError::CounterKeyIncorrect
        );
        
        // Increment the counter
        let current_count = u64::from_le_bytes(counter_state.count);
        let new_count = current_count.saturating_add(1);
        counter_state.count = new_count.to_le_bytes();
        
        Ok(())
    }
);