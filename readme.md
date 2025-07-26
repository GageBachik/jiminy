<h1 align="center">
  <code>jiminy</code>
</h1>
<p align="center">
  <img width="400" height="400" alt="jiminy logo" src="https://github.com/user-attachments/assets/a3f172c9-692c-4363-881f-d6288f806623" />
</p>
<p align="center">
    Pinocchio Solana Program Macros
</p>


A comprehensive macro system for building high-performance Solana programs with the pinocchio framework, designed to minimize boilerplate while maintaining zero-cost abstractions and security.

## Usage

    To use just drop jiminy.rs in your src directory and build.rs in your root directory
    Checkout the cargo.toml for dependencies and this file for usage examples
    Explore the exmaples directory for some pinocchio programs using jiminy

## Table of Contents

1. [Overview](#overview)
2. [Error Definition System](#error-definition-system)
3. [State Definition Macros](#state-definition-macros)
4. [Instruction Definition System](#instruction-definition-system)
5. [Account Validation Macros](#account-validation-macros)
6. [Account Loading Macros](#account-loading-macros)
7. [Token Operations](#token-operations)
8. [Account Management](#account-management)
9. [Program Generation](#program-generation)
10. [Build System Integration](#build-system-integration)
11. [Best Practices](#best-practices)
12. [Migration Guide](#migration-guide)
13. [Development Workflow](#development-workflow)
14. [Performance Characteristics](#performance-characteristics)

## Overview

Jiminy.rs provides a complete macro ecosystem for Solana program development, including:
- **Error Management**: `define_errors!` for ShankType-compatible error definitions
- **State Management**: `define_state!` for on-chain state structs
- **Instruction Definition**: `define_instruction_with_metadata!` for instruction handlers
- **Account Operations**: Loading, validation, and PDA management macros
- **Token Operations**: Transfer and account management macros
- **Program Generation**: Automatic dispatch and IDL generation macros
- **Performance Utilities**: Unsafe optimized functions for critical paths

## Error Definition System

### `define_errors!` Macro

The `define_errors!` macro provides seamless integration with Shank IDL generation by parsing macro calls during build time and generating the actual error enum in `generated.rs` where Shank can see it.

#### Usage

Define errors using the `define_errors!` macro in your `error.rs` file:

```rust
// src/error.rs
define_errors! {
    PTokenProgramError,
    InvalidDiscriminator = 6001,
    PlatformKeyIncorrect = 6002,
    VaultKeyIncorrect = 6003,
    InsufficientFunds = 6004,
    Unauthorized = 6005,
}
```

#### How It Works

1. **Build Script Parsing**: The build script parses `define_errors!` macro calls from `error.rs`
2. **Code Generation**: Generates the actual enum with `#[derive(ShankType)]` in `generated.rs`
3. **Shank Compatibility**: Shank sees the generated enum directly in source code during static analysis
4. **IDL Generation**: The errors appear correctly in the generated IDL

#### Generated Code

The macro generates code equivalent to:

```rust
// In generated.rs (auto-generated)
use shank::ShankType;
use pinocchio::program_error::ProgramError;

#[derive(Clone, PartialEq, ShankType)]
pub enum PTokenProgramError {
    InvalidDiscriminator = 6001,
    PlatformKeyIncorrect = 6002,
    VaultKeyIncorrect = 6003,
    InsufficientFunds = 6004,
    Unauthorized = 6005,
}

impl From<PTokenProgramError> for ProgramError {
    fn from(e: PTokenProgramError) -> Self {
        Self::Custom(e as u32)
    }
}
```

#### Multiple Error Types

You can define multiple error enums in the same project:

```rust
// src/error.rs
define_errors! {
    ValidationError,
    InvalidInput = 7001,
    MissingRequiredField = 7002,
}

define_errors! {
    BusinessLogicError,  
    InsufficientBalance = 8001,
    OperationNotAllowed = 8002,
}
```

All error types will be generated in `generated.rs` and available in the IDL.

## State Definition Macros

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

## Instruction Definition System

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

## Token Operations

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

## Account Management

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

## Program Generation

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
    error_type: PTokenProgramError,
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

## Build System Integration

The enhanced build system automatically discovers and generates code for both instructions and errors:

### Error Discovery and Generation

```rust
// Scans src/error.rs for define_errors! macros
let errors = extract_error_metadata();

// Generates ShankType enums in generated.rs
#[derive(Clone, PartialEq, ShankType)]
pub enum PTokenProgramError {
    InvalidDiscriminator = 6001,
    // ... variants
}
```

### Instruction Discovery

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
        _ => Err(PTokenProgramError::InvalidDiscriminator.into()),
    }
}
```

## Making Jiminy Generic

To use Jiminy in any Pinocchio project:

1. **Copy Core Files**:
   ```
   src/jiminy.rs    # Core macros and utilities  
   build.rs         # Enhanced build script with error parsing
   ```

2. **Define Errors**:
   ```rust
   // src/error.rs
   define_errors! {
       YourProgramError,
       InvalidDiscriminator = 6001,
       InvalidAccount = 6002,
       InsufficientFunds = 6003,
       Unauthorized = 6004,
   }
   ```

3. **Update lib.rs**:
   ```rust
   #[macro_use]
   pub mod jiminy;
   pub mod instructions;
   pub mod state;
   pub mod utils;

   // Errors are generated in generated.rs by the build script
   pub use instructions::*;

   // Include the generated program code
   pub mod generated;
   pub use generated::*;
   ```

### IDL Generation

After defining errors with `define_errors!`, run:

```bash
cargo build  # Generates errors in generated.rs
shank idl -o idl/  # Creates IDL with errors included
```

Your errors will appear in the generated IDL under the `types` section, making them available to client-side code generation.

### Example Project Structure

```
my-pinocchio-project/
├── src/
│   ├── jiminy.rs      # Core Jiminy utilities
│   ├── error.rs       # Error definitions (using define_errors! macro)
│   ├── generated.rs   # Auto-generated (contains errors + instructions)
│   ├── lib.rs         # Main library file
│   └── instructions/  # Instruction handlers (using macros)
├── build.rs           # Enhanced build script
└── idl/
    └── program.json   # Generated IDL with errors and instructions
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

### 4. Error Handling Patterns

**Use generated error types:**
```rust
// Error handling with define_errors! macro
define_errors! {
    PTokenProgramError,
    PlatformKeyIncorrect = 6002,
    VoteHasAlreadyEnded = 6007,
    DidNotVoteForWinningSide = 6010,
}

// Error propagation in instructions
if now < vote_deadline {
    return Err(PTokenProgramError::VoteIsStillRunning.into());
}
```

## Migration Guide

### From Manual Implementation

**Before (Manual Implementation)**
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
```

**After (Jiminy Macros)**
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

### 2. Define Error Types
```rust
// src/error.rs
define_errors! {
    YourProgramError,
    InvalidDiscriminator = 6001,
    PlatformKeyIncorrect = 6002,
    InsufficientFunds = 6003,
}
```

### 3. Define State Structs
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

### 4. Create Instructions
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

### 5. Build and Test
```bash
# Build program with automatic IDL generation
cargo build-sbf

# Generate IDL explicitly
shank idl -o idl/

# Run tests
cargo test-sbf
```

## Performance Characteristics

The jiminy macro system provides:
- **Zero-cost abstractions**: Macros expand to efficient pinocchio calls
- **Compile-time validation**: Account types validated at compile time
- **Minimal runtime overhead**: Direct memory access with bytemuck
- **Optimized PDA validation**: Fast validation without recomputation
- **Batch operations**: Efficient bulk validation and operations
- **Build-time code generation**: Errors and instructions generated for Shank compatibility

## Complete Macro Reference

### Core Macros
- `define_errors!` - Error enum definition with ShankType
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

This system delivers the developer experience of high-level frameworks with the performance characteristics of hand-optimized pinocchio code, now including seamless error definition and IDL generation capabilities.
