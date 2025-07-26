# Jiminy.rs - Pinocchio Solana Program Macros

A comprehensive macro system for building high-performance Solana programs with the pinocchio framework, designed to minimize boilerplate while maintaining zero-cost abstractions and security.

## Overview

Jiminy.rs provides a complete macro ecosystem for Solana program development, including:
- **State Management**: `define_state!` for on-chain state structs
- **Instruction Definition**: `define_instruction_with_metadata!` for instruction handlers
- **Account Operations**: Loading, validation, and PDA management macros
- **Token Operations**: Transfer and account management macros
- **Program Generation**: Automatic dispatch and IDL generation macros
- **Performance Utilities**: Unsafe optimized functions for critical paths

## State Definition Macro

### `define_state!`

Creates on-chain state structs with automatic `Pod`, `Zeroable`, and memory management implementations.

```rust
define_state! {
    pub struct Vote {
        pub token: [u8; 32],
        pub true_votes: [u8; 8],
        pub false_votes: [u8; 8],
        pub end_timestamp: [u8; 8],
        pub vault_bump: u8,
    }
}
```

**Generated Features:**
- `#[repr(C)]` for C-style memory layout
- `#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]` for efficient serialization
- `impl` block with `LEN` constant and `load()` method for account data loading

**Key Design Decisions:**
- Uses byte arrays (`[u8; 8]`) instead of primitive types for optimal on-chain sizing
- No padding fields - relies on proper field ordering for alignment
- Direct memory access for maximum performance

## Instruction Definition Macro

### `define_instruction_with_metadata!`

Creates complete instruction handlers with account parsing, data deserialization, and shank IDL metadata.

```rust
define_instruction_with_metadata!(
    discriminant: 2,
    InitializeVote,
    accounts: {
        authority: signer => writable, desc: "Authority of the vault",
        platform: program, desc: "Platform pda key",
        vote: signer => writable, desc: "new vote account",
        // ... more accounts
    },
    data: {
        time_to_add: [u8; 8],
    },
    process: {
        // Implementation code here
        Ok(())
    }
);
```

### Account Types

The macro supports several account type annotations:

- `signer`: Account must be a signer
- `program`: Account owned by our program  
- `token`: Account owned by token program
- `not_token`: Account NOT owned by token program (for ATAs)
- `uninitialized`: Account not yet initialized (automatically marked writable)
- `any`: Any account type

### Account Mutability

Add `=> writable` to mark accounts as mutable in the IDL:

```rust
vote: signer => writable, desc: "new vote account",
authority_token_account: token => writable, desc: "user's token account",
```

### Generated Components

The macro generates:
1. **Accounts struct**: For type-safe account access with validation
2. **Data struct**: For instruction data with bytemuck serialization  
3. **Instruction struct**: Combining accounts and data
4. **TryFrom implementations**: For parsing from raw account/data arrays
5. **Shank annotations**: For automatic IDL generation
6. **Metadata constants**: For build script integration

## Account Validation Macros

### `validate_account!`

Validates individual accounts based on type and mutability requirements:

```rust
validate_account!(account, signer);                    // Must be signer
validate_account!(account, signer => writable);        // Signer + writable
validate_account!(account, program);                   // Owned by program + initialized
validate_account!(account, program => writable);       // Program + writable
validate_account!(account, token);                     // Token program account
validate_account!(account, token => writable);         // Token + writable
validate_account!(account, uninitialized);             // System-owned, 0 lamports
validate_account!(account, not_token);                 // NOT token program
validate_account!(account, any);                       // No validation
```

### `assert_pda!`

Fast PDA validation without recomputing the address:

```rust
assert_pda!(account, 
    seeds: [PLATFORM_SEED], 
    bump: platform_state.platform_bump,
    error: PTokenProgramError::PlatformKeyIncorrect);
```

### `validate_pdas!`

Batch PDA validation for multiple accounts:

```rust
validate_pdas!(
    platform => seeds: [PLATFORM_SEED], bump: platform_state.platform_bump,
        error: PTokenProgramError::PlatformKeyIncorrect;
    vault => seeds: [platform.key().as_ref()], bump: platform_state.vault_bump,
        error: PTokenProgramError::VaultKeyIncorrect;
    vote_vault => seeds: [vote.key().as_ref()], bump: vote_state.vault_bump,
        error: PTokenProgramError::VoteVaultKeyIncorrect
);
```

## Account Loading Macros

### `load_mut!`

Load account data as mutable reference with zero-copy:

```rust
let vote_state = load_mut!(vote, Vote);
vote_state.true_votes = new_vote_count.to_be_bytes();
```

### `load!`

Load account data as immutable reference:

```rust
let vote_state = load!(vote, Vote);
let end_time = i64::from_le_bytes(vote_state.end_timestamp);
```

### `with_state!`

Load state within a closure for safer mutation patterns:

```rust
with_state!(vote, Vote, |vote_state| {
    vote_state.token = *token.key();
    vote_state.vault_bump = vote_vault_bump;
    vote_state.end_timestamp = (i64::from_le_bytes(time_to_add)
        + Clock::get()?.unix_timestamp)
        .to_be_bytes();
});
```

## Token Operations Macros

### `transfer_tokens!`

Transfer tokens with optional PDA signing:

```rust
// Simple transfer
transfer_tokens!(from_account, to_account, authority, amount);

// With PDA signing
let bump = [vote_state.vault_bump];
transfer_tokens!(vote_vault_token_account, authority_token_account, vote_vault, reward,
    seeds: [vote.key().as_ref(), &bump]);
```

### `transfer_sol!`

Transfer SOL between accounts:

```rust
transfer_sol!(authority, vote_vault, init_sol);
transfer_sol!(authority, vault, fee_sol);
```

## Account Management Macros

### `create_pda!`

Create PDA accounts with automatic bump calculation:

```rust
create_pda!(
    from: authority,
    to: new_account,
    space: StateStruct::LEN,
    seeds: [SEED_PREFIX, user.key().as_ref()],
    bump: bump_seed
);
```

### `close_account!`

Efficiently close accounts and transfer lamports:

```rust
close_account!(position, vault);
```

## Utility Macros

### `to_le_bytes!` and `to_be_bytes!`

Optimized byte array conversions:

```rust
let value = to_le_bytes!(stored_bytes);     // u64::from_le_bytes(stored_bytes)
let value = to_be_bytes!(stored_bytes);     // u64::from_be_bytes(stored_bytes)
```

## Build System Integration

The project uses `build.rs` to automatically generate shank-compatible code:

### Automatic Instruction Discovery
```rust
// Scans src/instructions/*.rs files for define_instruction_with_metadata! macros
let instructions = extract_instruction_metadata();
```

### IDL Generation
```rust
// Generates ShankInstruction enum with proper account annotations
#[repr(u8)]
#[derive(Clone, Debug, PartialEq, ShankInstruction)]
pub enum ProgramInstructions {
    #[account(0, signer, writable, name = "authority", desc = "Authority of the vault")]
    InitializePlatform { fee: [u8; 2] },
}
```

### Dispatch Generation  
```rust
// Automatically generates process_instruction function
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.first() {
        Some(0) => InitializePlatformInstruction::try_from((accounts, &instruction_data[1..]))?.process(),
        // ... other instructions
    }
}
```

## Program Generation Macros

### `jiminy_define_program!`

Complete program definition with shank enum generation:

```rust
jiminy_define_program!(
    0 => InitializePlatform {
        accounts: [
            0: authority(signer, writable) = "Authority of the vault",
            1: platform(writable) = "Platform pda key",
        ],
        data: {
            fee: [u8; 2],
            platform_bump: u8,
        }
    },
    1 => UpdatePlatform {
        // ... account and data definitions
    }
);
```

### `jiminy_program!`

Simple program dispatch without shank enum:

```rust
jiminy_program!(
    0 => InitializePlatform,
    1 => UpdatePlatform,
    2 => InitializeVote,
);
```

### `define_program_instructions!`

Generate shank enum from instruction variants:

```rust
define_program_instructions!(
    shank_instruction!(InitializePlatform {
        #[account(0, signer, writable, name = "authority", desc = "Authority")]
        data: { fee: [u8; 2] }
    }),
    shank_instruction!(UpdatePlatform { 
        data: { new_fee: [u8; 2] }
    })
);
```

## Performance Utilities

### `perf` Module

Unsafe optimized functions for critical performance paths:

```rust
use crate::jiminy::perf;

// Unsafe fast loading (use with caution)
let state = unsafe { perf::load_unchecked::<Vote>(vote_account)? };

// Fast memcpy for account data
unsafe { perf::fast_copy(&source_data, &mut target_data); }
```

## Complete Usage Examples

### 1. State Management

```rust
define_state! {
    pub struct Platform {
        pub authority: [u8; 32],
        pub fee: [u8; 2],
        pub platform_bump: u8,
        pub vault_bump: u8,
    }
    
    pub struct Vote {
        pub token: [u8; 32],
        pub true_votes: [u8; 8],
        pub false_votes: [u8; 8],
        pub end_timestamp: [u8; 8],
        pub vault_bump: u8,
    }
}

// Usage in instruction
let platform_state = load_mut!(platform, Platform);
assert_pda!(platform, seeds: [PLATFORM_SEED], bump: platform_state.platform_bump,
    error: PTokenProgramError::PlatformKeyIncorrect);
```

### 2. Complex Instruction Implementation

```rust
define_instruction_with_metadata!(
    discriminant: 5,
    RedeemWinnings,
    accounts: {
        authority: signer => writable, desc: "Authority of the vault",
        platform: program, desc: "Platform pda key",
        vote: program => writable, desc: "vote account",
        position: program => writable, desc: "position pda for voting on one side",
        authority_token_account: token => writable, desc: "user's token account",
        vote_vault_token_account: token => writable, desc: "vote's token account",
    },
    data: {},
    process: {
        // Load and validate multiple state accounts
        let platform_state = load_mut!(platform, Platform);
        let vote_state = load_mut!(vote, Vote);
        let position_state = load_mut!(position, Position);

        // Batch PDA validation
        validate_pdas!(
            platform => seeds: [PLATFORM_SEED], bump: platform_state.platform_bump,
                error: PTokenProgramError::PlatformKeyIncorrect;
            position => seeds: [POSITION_SEED, vote.key().as_ref(), authority.key().as_ref()],
                bump: position_state.bump,
                error: PTokenProgramError::PositionKeyIncorrect
        );

        // Business logic
        let now = Clock::get()?.unix_timestamp;
        let vote_deadline = i64::from_le_bytes(vote_state.end_timestamp);
        if now < vote_deadline {
            return Err(PTokenProgramError::VoteIsStillRunning.into());
        }

        // Calculate winnings
        let position_amount = u64::from_le_bytes(position_state.amount);
        let reward = calculate_reward(position_amount, vote_state);

        // Transfer with PDA signing
        let bump = [vote_state.vault_bump];
        transfer_tokens!(vote_vault_token_account, authority_token_account, vote_vault, reward,
            seeds: [vote.key().as_ref(), &bump]);

        // Close position account
        close_account!(position, vault);

        Ok(())
    }
);
```

### 3. PDA Creation and Management

```rust
// Manual PDA creation for signer accounts
pinocchio_system::instructions::CreateAccount {
    from: authority,
    to: vote,
    space: Vote::LEN as u64,
    lamports: Rent::get()?.minimum_balance(Vote::LEN),
    owner: &crate::ID,
}.invoke()?;

// Automatic PDA creation with macro
create_pda!(
    from: authority,
    to: position_account,
    space: Position::LEN,
    seeds: [POSITION_SEED, vote.key().as_ref(), authority.key().as_ref()],
    bump: position_bump
);
```

## Best Practices

### 1. Data Types and Memory Layout
- **Use byte arrays**: `[u8; N]` for all numeric data to avoid endianness issues and ensure consistent sizing
- **Conversion patterns**: Use `u64::from_le_bytes()` for storage, `u64::from_be_bytes()` for wire format
- **Alignment**: Keep structs minimal and properly aligned - no padding fields
- **Fixed sizes**: All state structs must have predictable, fixed sizes

```rust
// Good: predictable byte layout
pub struct Vote {
    pub token: [u8; 32],        // Always 32 bytes
    pub true_votes: [u8; 8],    // Always 8 bytes
    pub vault_bump: u8,         // Always 1 byte
}

// Bad: unpredictable sizing
pub struct BadVote {
    pub token: Pubkey,          // Size varies by platform
    pub votes: u64,             // Endianness issues
}
```

### 2. Security Validations

**Always validate PDAs using macros:**
```rust
// Single PDA validation
assert_pda!(platform, seeds: [PLATFORM_SEED], bump: platform_state.platform_bump,
    error: PTokenProgramError::PlatformKeyIncorrect);

// Batch validation for efficiency
validate_pdas!(
    platform => seeds: [PLATFORM_SEED], bump: platform_state.platform_bump,
        error: PTokenProgramError::PlatformKeyIncorrect;
    vault => seeds: [platform.key().as_ref()], bump: platform_state.vault_bump,
        error: PTokenProgramError::VaultKeyIncorrect
);
```

**Account validation is automatic:**
```rust
// The macro automatically validates account types
accounts: {
    authority: signer => writable, desc: "Must be signer and writable",
    platform: program, desc: "Must be owned by program and initialized",
    token_account: token => writable, desc: "Must be token account and writable",
}
```

### 3. Performance Optimizations

**Use efficient loading patterns:**
```rust
// Preferred: direct mutable loading
let vote_state = load_mut!(vote, Vote);
vote_state.true_votes = new_count.to_be_bytes();

// Alternative: closure pattern for complex updates
with_state!(vote, Vote, |vote_state| {
    vote_state.token = *token.key();
    vote_state.end_timestamp = deadline.to_be_bytes();
});

// Critical path: unsafe optimized loading
let state = unsafe { perf::load_unchecked::<Vote>(vote_account)? };
```

**Batch operations when possible:**
```rust
// Good: batch token transfers
let bump = [vote_state.vault_bump];
transfer_tokens!(vault_account, user_account, vault_pda, reward_amount,
    seeds: [vote.key().as_ref(), &bump]);
transfer_tokens!(vault_account, fee_account, vault_pda, fee_amount,
    seeds: [vote.key().as_ref(), &bump]);
```

### 4. Error Handling Patterns

**Use specific error types:**
```rust
// Custom error enum with shank integration
#[derive(Clone, PartialEq, ShankType)]
pub enum PTokenProgramError {
    PlatformKeyIncorrect = 6002,
    VoteHasAlreadyEnded = 6007,
    DidNotVoteForWinningSide = 6010,
}

// Error propagation in macros
if now < vote_deadline {
    return Err(PTokenProgramError::VoteIsStillRunning.into());
}
```

### 5. Account Management

**PDA creation patterns:**
```rust
// For signer accounts: manual creation
pinocchio_system::instructions::CreateAccount {
    from: authority,
    to: vote,
    space: Vote::LEN as u64,
    lamports: Rent::get()?.minimum_balance(Vote::LEN),
    owner: &crate::ID,
}.invoke()?;

// For PDA accounts: use macro
create_pda!(
    from: authority,
    to: position_account,
    space: Position::LEN,
    seeds: [POSITION_SEED, vote.key().as_ref(), authority.key().as_ref()],
    bump: position_bump
);
```

**Account closing:**
```rust
// Efficient account closing with lamport transfer
close_account!(position, vault);
```

## IDL Integration

The macros provide automatic IDL generation through shank integration:

### Automatic Shank Enum Generation
```rust
// Generated in src/generated.rs
#[repr(u8)]
#[derive(Clone, Debug, PartialEq, ShankInstruction)]
pub enum ProgramInstructions {
    #[account(0, signer, writable, name = "authority", desc = "Authority")]
    #[account(1, writable, name = "platform", desc = "Platform pda key")]
    InitializePlatform {
        fee: [u8; 2],
        platform_bump: u8,
        vault_bump: u8,
    },
}
```

### ShankAccount Integration
```rust
// Automatically generated for state structs
#[repr(C)]
#[derive(Clone, shank::ShankAccount)]
pub struct Vote {
    pub token: [u8; 32],
    pub true_votes: [u8; 8],
    pub false_votes: [u8; 8],
    pub end_timestamp: [u8; 8],
    pub vault_bump: u8,
}
```

### IDL Generation Commands
```bash
# Generate IDL for the program
shank idl -p pVoTew8KNhq6rsrYq9jEUzKypytaLtQR62UbagWTCvu

# Build program and generate IDL in one step
cargo build-sbf
```

## Complete Macro Reference

### Core Macros
- `define_instruction_with_metadata!` - Main instruction definition
- `define_state!` - State struct definition

### Validation Macros
- `validate_account!` - Individual account validation
- `assert_pda!` - Single PDA validation  
- `validate_pdas!` - Batch PDA validation

### Loading Macros
- `load_mut!` - Mutable account loading
- `load!` - Immutable account loading
- `with_state!` - Closure-based state loading

### Operation Macros
- `create_pda!` - PDA creation with bump
- `transfer_tokens!` - Token transfers (with/without PDA signing)
- `transfer_sol!` - SOL transfers
- `close_account!` - Account closing with lamport transfer

### Utility Macros
- `to_le_bytes!` - Little endian conversion
- `to_be_bytes!` - Big endian conversion

### Program Generation Macros
- `jiminy_define_program!` - Complete program with shank enum
- `jiminy_program!` - Simple dispatch generation
- `define_program_instructions!` - Shank enum generation
- `shank_instruction!` - Individual instruction variants

## Migration from Traditional Patterns

### Before (Manual Implementation)
```rust
// Manual account struct definition
#[repr(C)]
pub struct InitializePlatformAccounts<'info> {
    pub authority: &'info AccountInfo,
    pub platform: &'info AccountInfo,
    pub vault: &'info AccountInfo,
}

// Manual validation logic
impl<'info> TryFrom<&'info [AccountInfo]> for InitializePlatformAccounts<'info> {
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, platform, vault, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        // ... more manual validation
        
        Ok(Self { authority, platform, vault })
    }
}

// Manual data struct
#[repr(C)]
#[derive(Pod, Zeroable)]
pub struct InitializePlatformData {
    pub fee: [u8; 2],
    pub platform_bump: u8,
    pub vault_bump: u8,
}

// Manual instruction processing
pub fn process_initialize_platform(
    accounts: InitializePlatformAccounts,
    data: InitializePlatformData,
) -> ProgramResult {
    // Implementation...
    Ok(())
}
```

### After (Jiminy Macros)
```rust
define_instruction_with_metadata!(
    discriminant: 0,
    InitializePlatform,
    accounts: {
        authority: signer => writable, desc: "Authority of the vault",
        platform: any => writable, desc: "Platform pda key",
        vault: any => writable, desc: "Platforms fee vault pda",
        system_program: any, desc: "System program",
    },
    data: {
        fee: [u8; 2],
        platform_bump: u8,
        vault_bump: u8,
    },
    process: {
        // Direct implementation with automatic validation
        // All accounts pre-validated, data pre-parsed
        
        create_pda!(
            from: authority,
            to: platform,
            space: Platform::LEN,
            seeds: [PLATFORM_SEED],
            bump: platform_bump
        );
        
        Ok(())
    }
);
```

## Common Gotchas and Solutions

### 1. Byte Order Consistency
```rust
// Problem: mixing endianness
vote_state.true_votes = total_true.to_le_bytes();  // Storage
let network_value = total_true.to_be_bytes();      // Wire format

// Solution: consistent patterns
vote_state.true_votes = total_true.to_be_bytes();  // Always big-endian for storage
let stored_value = u64::from_be_bytes(vote_state.true_votes);
```

### 2. Account Size Calculation
```rust
// Problem: incorrect size calculation
pub struct Vote {
    pub token: Pubkey,      // Size varies by platform!
    pub votes: u64,         // Padding issues
}

// Solution: fixed-size byte arrays
pub struct Vote {
    pub token: [u8; 32],    // Always 32 bytes
    pub votes: [u8; 8],     // Always 8 bytes
}
```

### 3. PDA Validation
```rust
// Problem: manual PDA validation
let (expected_pda, _) = Pubkey::find_program_address(&[seeds], &program_id);
if account.key() != &expected_pda {
    return Err(ProgramError::InvalidAccountData);
}

// Solution: use validation macros
assert_pda!(account, seeds: [PLATFORM_SEED], bump: bump,
    error: PTokenProgramError::PlatformKeyIncorrect);
```

### 4. Uninitialized Account Handling
```rust
// The macro automatically marks uninitialized accounts as writable
accounts: {
    new_account: uninitialized, desc: "Will be created",  // Automatically writable
}
```

## Development Workflow

### 1. Project Setup
```bash
# Add dependencies to Cargo.toml
[dependencies]
pinocchio = { version = "0.1.0" }
bytemuck = { version = "1.0" }
shank = { version = "0.1.0" }
paste = "1.0"
```

### 2. Define State Structs
```rust
// src/state/mod.rs
define_state! {
    pub struct Platform {
        pub authority: [u8; 32],
        pub fee: [u8; 2],
        pub platform_bump: u8,
        pub vault_bump: u8,
    }
}
```

### 3. Create Instructions
```rust
// src/instructions/initialize_platform.rs
define_instruction_with_metadata!(
    discriminant: 0,
    InitializePlatform,
    accounts: { /* ... */ },
    data: { /* ... */ },
    process: { /* ... */ }
);
```

### 4. Build and Test
```bash
# Build program with automatic IDL generation
cargo build-sbf

# Generate IDL explicitly
shank idl -p <program_id>

# Run tests
cargo test-sbf
```

### 5. Deploy
```bash
# Deploy to devnet
solana program deploy target/deploy/program.so --program-id keypair.json
```

## Performance Characteristics

The jiminy macro system provides:
- **Zero-cost abstractions**: Macros expand to efficient pinocchio calls
- **Compile-time validation**: Account types validated at compile time
- **Minimal runtime overhead**: Direct memory access with bytemuck
- **Optimized PDA validation**: Fast validation without recomputation
- **Batch operations**: Efficient bulk validation and operations

This system delivers the developer experience of high-level frameworks with the performance characteristics of hand-optimized pinocchio code.