use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

/// Generates complete instruction handler with minimal boilerplate
/// Also generates metadata for automatic shank enum generation via build script
#[macro_export]
macro_rules! define_instruction_with_metadata {
    (
        discriminant: $disc:literal,
        $name:ident,
        // Accounts with their validation rules and descriptions
        accounts: {
            $(
                $account:ident: $account_type:tt $(=> $validation:tt)*, desc: $desc:literal
            ),* $(,)?
        },
        // Instruction data fields
        data: {
            $(
                $field:ident: $field_type:ty
            ),* $(,)?
        },
        // Process function body
        process: $process_body:block
    ) => {
        use bytemuck::{Pod, Zeroable};
        use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

        #[repr(C)]
        pub struct $name<'info> {
            $(pub $account: &'info AccountInfo,)*
        }

        ::paste::paste! {
            #[repr(C)]
            #[derive(Clone, Copy, Pod, Zeroable)]
            pub struct [<$name Data>] {
                $(pub $field: $field_type,)*
            }

            impl [<$name Data>] {
                pub const LEN: usize = core::mem::size_of::<Self>();
            }

            #[repr(C)]
            pub struct [<$name Instruction>]<'info> {
                pub accounts: $name<'info>,
                pub data: [<$name Data>],
            }
        }

        impl<'info> TryFrom<&'info [AccountInfo]> for $name<'info> {
            type Error = ProgramError;

            fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
                // Destructure accounts array
                let [$($account,)* ..] = accounts else {
                    return Err(ProgramError::NotEnoughAccountKeys);
                };

                // Apply validations
                $(
                    validate_account!($account, $account_type $(=> $validation)*);
                )*

                Ok(Self {
                    $($account,)*
                })
            }
        }

        ::paste::paste! {
            impl<'info> TryFrom<(&'info [AccountInfo], &'info [u8])> for [<$name Instruction>]<'info> {
                type Error = ProgramError;

                fn try_from((accounts, data): (&'info [AccountInfo], &'info [u8])) -> Result<Self, Self::Error> {
                    let accounts = $name::try_from(accounts)?;
                    let data = bytemuck::try_from_bytes::<[<$name Data>]>(data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    Ok(Self {
                        accounts,
                        data: *data,
                    })
                }
            }

            impl<'info> [<$name Instruction>]<'info> {
                pub fn process(self) -> ProgramResult {
                    // Destructure for easier access in process body
                    let Self { accounts, data } = self;
                    #[allow(unused_variables)]
                    let $name { $($account,)* } = accounts;
                    #[allow(unused_variables)]
                    let [<$name Data>] { $($field,)* } = data;

                    $process_body
                }
            }

            // Export metadata for build script parsing with auto-generated shank attributes
            #[doc(hidden)]
            #[allow(non_snake_case)]
            pub mod [<$name _METADATA>] {
                pub const DISCRIMINATOR: u8 = $disc;
                pub const NAME: &str = stringify!($name);

                // Account metadata with auto-assigned indices
                pub const ACCOUNTS: &[(&str, &str, usize, &str)] = &[
                    $(
                        (stringify!($account), stringify!($account_type), define_instruction_with_metadata!(@index_counter), $desc),
                    )*
                ];

                // Auto-generated shank attributes
                pub const SHANK_ATTRS: &[(&str, &[&str])] = &[
                    $(
                        (stringify!($account), define_instruction_with_metadata!(@shank_attrs $account_type $(=> $validation)*)),
                    )*
                ];

                pub const FIELDS: &[(&str, &str)] = &[
                    $(
                        (stringify!($field), stringify!($field_type)),
                    )*
                ];
            }
        }
    };

    // Helper to auto-assign indices (this is a simplified approach - build script will handle proper indexing)
    (@index_counter) => { 0 };

    // Helper to generate shank attributes from account type and validation
    (@shank_attrs signer => writable) => { &["signer", "writable"] };
    (@shank_attrs signer) => { &["signer"] };
    (@shank_attrs uninitialized => writable) => { &["writable"] };
    (@shank_attrs uninitialized) => { &["writable"] }; // uninitialized accounts are always writable
    (@shank_attrs $account_type:tt => writable) => { &["writable"] };
    (@shank_attrs $account_type:tt) => { &[] };
}

/// Validates accounts based on type and additional rules
#[macro_export]
macro_rules! validate_account {
    // Signer validation
    ($account:expr, signer) => {{
        if !$account.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
    }};

    // Signer + writable
    ($account:expr, signer => writable) => {{
        if !$account.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if !$account.is_writable() {
            return Err(ProgramError::InvalidAccountData);
        }
    }};

    // Program account (owned by program + initialized)
    ($account:expr, program) => {{
        if !$account.is_owned_by(&$crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        if $account.lamports() == 0 {
            return Err(ProgramError::UninitializedAccount);
        }
    }};

    // Program account + writable
    ($account:expr, program => writable) => {{
        $crate::validate_account!($account, program);
        if !$account.is_writable() {
            return Err(ProgramError::InvalidAccountData);
        }
    }};

    // Uninitialized system account
    ($account:expr, uninitialized) => {{
        if !$account.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        if $account.lamports() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
    }};

    // Uninitialized system account + writable
    ($account:expr, uninitialized => writable) => {{
        $crate::validate_account!($account, uninitialized);
        if !$account.is_writable() {
            return Err(ProgramError::InvalidAccountData);
        }
    }};

    // Token account
    ($account:expr, token) => {{
        if !$account.is_owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        if $account.lamports() == 0 {
            return Err(ProgramError::UninitializedAccount);
        }
    }};

    // Token account + writable
    ($account:expr, token => writable) => {{
        $crate::validate_account!($account, token);
        if !$account.is_writable() {
            return Err(ProgramError::InvalidAccountData);
        }
    }};

    // Token account (but NOT owned by token program - for ATAs)
    ($account:expr, not_token) => {{
        if $account.is_owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
    }};

    // Token account (but NOT owned by token program - for ATAs) + writable
    ($account:expr, not_token => writable) => {{
        $crate::validate_account!($account, not_token);
        if !$account.is_writable() {
            return Err(ProgramError::InvalidAccountData);
        }
    }};

    // Any account + writable
    ($account:expr, any => writable) => {{
        if !$account.is_writable() {
            return Err(ProgramError::InvalidAccountData);
        }
    }};

    // Any account type
    ($account:expr, any) => {{
        // No validation needed for any type
    }};

    // Custom validation
    ($account:expr, any, custom($validation:expr)) => {{
        if !$validation($account) {
            return Err(ProgramError::InvalidAccountData);
        }
    }};
}

/// Fast PDA validation without recomputing
#[macro_export]
macro_rules! assert_pda {
    ($account:expr, seeds: [$($seed:expr),*], bump: $bump:expr, error: $error:expr) => {{
        use pinocchio_pubkey::derive_address;
        let expected = derive_address(&[$($seed),*], Some($bump), &$crate::ID);
        if $account.key() != &expected {
            return Err($error.into());
        }
    }};
}

/// Load account data with zero-copy
#[macro_export]
macro_rules! load_mut {
    ($account:expr, $type:ty) => {{
        let data = unsafe { $account.borrow_mut_data_unchecked() };
        bytemuck::try_from_bytes_mut::<$type>(data).map_err(|_| ProgramError::InvalidAccountData)?
    }};
}

/// Load account data immutably
#[macro_export]
macro_rules! load {
    ($account:expr, $type:ty) => {{
        unsafe {
            let data = $account.borrow_data_unchecked();
            bytemuck::try_from_bytes::<$type>(&data)
                .map_err(|_| ProgramError::InvalidAccountData)?
        }
    }};
}

/// Create PDA with automatic bump calculation
#[macro_export]
macro_rules! create_pda {
    (
        from: $from:expr,
        to: $to:expr,
        space: $space:expr,
        seeds: [$($seed:expr),*],
        bump: $bump:expr
    ) => {{
        use pinocchio::{
            instruction::{Seed, Signer},
            sysvars::{rent::Rent, Sysvar},
        };

        let bump_seed = [$bump];
        let seeds = [$(Seed::from($seed),)* Seed::from(&bump_seed)];
        let signer = Signer::from(&seeds);

        pinocchio_system::instructions::CreateAccount {
            from: $from,
            to: $to,
            space: $space as u64,
            lamports: Rent::get()?.minimum_balance($space),
            owner: &$crate::ID,
        }
        .invoke_signed(&[signer])?;
    }};
}

/// Transfer tokens with optional PDA signing
#[macro_export]
macro_rules! transfer_tokens {
    ($from:expr, $to:expr, $authority:expr, $amount:expr) => {{
        pinocchio_token::instructions::Transfer {
            from: $from,
            to: $to,
            authority: $authority,
            amount: $amount,
        }
        .invoke()?;
    }};

    ($from:expr, $to:expr, $authority:expr, $amount:expr, seeds: [$($seed:expr),*]) => {{
        use pinocchio::instruction::{Seed, Signer};
        let seeds = [$(Seed::from($seed),)*];
        let signer = Signer::from(&seeds);

        pinocchio_token::instructions::Transfer {
            from: $from,
            to: $to,
            authority: $authority,
            amount: $amount,
        }
        .invoke_signed(&[signer])?;
    }};
}

/// Transfer SOL
#[macro_export]
macro_rules! transfer_sol {
    ($from:expr, $to:expr, $amount:expr) => {{
        pinocchio_system::instructions::Transfer {
            from: $from,
            to: $to,
            lamports: $amount,
        }
        .invoke()?;
    }};
}

/// Close account efficiently
#[macro_export]
macro_rules! close_account {
    ($account:expr, $receiver:expr) => {{
        // Transfer lamports
        *$receiver.try_borrow_mut_lamports()? += *$account.try_borrow_lamports()?;

        // Mark as closed and resize
        {
            let mut data = $account.try_borrow_mut_data()?;
            if !data.is_empty() {
                data[0] = 0xff;
            }
        }
        $account.resize(1)?;
        $account.close()?;
    }};
}

/// Optimized byte array conversions
#[macro_export]
macro_rules! to_le_bytes {
    ($arr:expr) => {
        u64::from_le_bytes($arr)
    };
}

#[macro_export]
macro_rules! to_be_bytes {
    ($arr:expr) => {
        u64::from_be_bytes($arr)
    };
}

/// Fast state loading pattern
#[macro_export]
macro_rules! with_state {
    ($account:expr, $type:ty, |$state:ident| $body:block) => {{
        let account_clone = $account.clone();
        let $state = $crate::load_mut!(account_clone, $type);
        $body
    }};
}

/// Batch PDA validation
#[macro_export]
macro_rules! validate_pdas {
    (
        $(
            $account:expr => seeds: [$($seed:expr),*], bump: $bump:expr, error: $error:expr
        );* $(;)?
    ) => {
        $(
            $crate::assert_pda!($account, seeds: [$($seed),*], bump: $bump, error: $error);
        )*
    };
}

/// Define state structs with automatic load methods and ShankAccount for IDL
#[macro_export]
macro_rules! define_state {
    (
        $(
            pub struct $name:ident {
                $(pub $field:ident: $field_type:ty),* $(,)?
            }
        )*
    ) => {


        $(
            #[repr(C)]
            #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
            pub struct $name {
                $(pub $field: $field_type,)*
            }

            impl $name {
                pub const LEN: usize = ::core::mem::size_of::<Self>();
            }
        )*
    };
}

/// Performance utilities
pub mod perf {
    use super::*;
    use bytemuck::Pod;

    /// Load account data as mutable reference (no_std compatible)
    /// Documentation
    ///
    /// # Safety
    ///
    /// Ensure the account data is initialized and matches the expected type
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn load_unchecked<T: Pod>(account: &AccountInfo) -> Result<&mut T, ProgramError> {
        let data = account.borrow_mut_data_unchecked();
        bytemuck::try_from_bytes_mut::<T>(data).map_err(|_| ProgramError::InvalidAccountData)
    }

    /// Fast memcpy for account data (no_std compatible)
    /// Documentation
    ///
    /// # Safety
    ///
    /// Ensure the source and destination slices are of the same length
    #[inline(always)]
    pub unsafe fn fast_copy(src: &[u8], dst: &mut [u8]) {
        if src.len() != dst.len() {
            panic!("Length mismatch in fast_copy");
        }
        core::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}

/// Define program errors with automatic InvalidDiscriminator
/// 
/// This macro generates error enums that are compatible with shank IDL generation.
/// The generated enum will appear in the IDL types section when shank processes the crate.
/// 
/// Usage:
/// ```rust
/// define_program_errors! {
///     pub enum MyProgramError {
///         #[doc = "Custom error message"]
///         CustomError = 6002,
///         AnotherError = 6003,
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_program_errors {
    (
        pub enum $error_name:ident {
            $(
                $(#[doc = $doc:literal])*
                $variant:ident = $code:literal
            ),* $(,)?
        }
    ) => {
        /// Program error codes  
        #[derive(Clone, PartialEq, Eq, ::shank::ShankType)]
        #[repr(u32)]
        pub enum $error_name {
            /// Invalid instruction discriminator
            InvalidDiscriminator = 6001,
            $(
                $(#[doc = $doc])*
                $variant = $code,
            )*
        }

        impl From<$error_name> for ::pinocchio::program_error::ProgramError {
            fn from(e: $error_name) -> Self {
                Self::Custom(e as u32)
            }
        }

        impl core::fmt::Display for $error_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $error_name::InvalidDiscriminator => write!(f, "Invalid instruction discriminator"),
                    $(
                        $error_name::$variant => write!(f, "{}", stringify!($variant)),
                    )*
                }
            }
        }

        // Re-export InvalidDiscriminator for jiminy macros
        pub use $error_name::InvalidDiscriminator;
    };
}

/// Re-export common items
pub use paste::paste;

/// Macro that generates both shank enum and dispatch from concise definition
#[macro_export]
macro_rules! jiminy_define_program {
    (
        $(
            $disc:literal => $instruction:ident {
                accounts: [
                    $(
                        $idx:literal: $name:ident($($attrs:ident),*) = $desc:literal
                    ),* $(,)?
                ],
                data: {
                    $(
                        $field:ident: $field_type:ty
                    ),* $(,)?
                }
            }
        ),* $(,)?
    ) => {
        use shank::ShankInstruction;
        use $crate::jiminy::paste::paste;

        // Generate the shank instruction enum
        #[repr(u8)]
        #[derive(Clone, Debug, PartialEq, ShankInstruction)]
        pub enum ProgramInstructions {
            $(
                $(
                    #[account($idx, $($attrs,)* name = stringify!($name), desc = $desc)]
                )*
                $instruction {
                    $(
                        #[allow(dead_code)]
                        $field: $field_type,
                    )*
                }
            ),*
        }

        // Generate the dispatch function
        pub fn process_instruction(
            program_id: &Pubkey,
            accounts: &[AccountInfo],
            instruction_data: &[u8],
        ) -> ProgramResult {
            // Validate program ID
            if program_id != &$crate::ID {
                return Err(ProgramError::IncorrectProgramId);
            }

            // Dispatch to instruction handlers
            match instruction_data.first() {
                $(
                    Some($disc) => {
                        paste! {
                            [<$instruction Instruction>]::try_from((accounts, &instruction_data[1..]))?.process()
                        }
                    }
                )*
                _ => Err(crate::error::InvalidDiscriminator.into()),
            }
        }
    };
}

/// Simple program definition that generates dispatch and references external shank enum
#[macro_export]
macro_rules! jiminy_program {
    (
        $(
            $disc:literal => $instruction:ident
        ),* $(,)?
    ) => {
        pub fn process_instruction(
            program_id: &Pubkey,
            accounts: &[AccountInfo],
            instruction_data: &[u8],
        ) -> ProgramResult {
            // Validate program ID
            if program_id != &$crate::ID {
                return Err(ProgramError::IncorrectProgramId);
            }

            // Dispatch to instruction handlers
            match instruction_data.first() {
                $(
                    Some($disc) => {
                        ::paste::paste! {
                            [<$instruction Instruction>]::try_from((accounts, &instruction_data[1..]))?.process()
                        }
                    }
                )*
                _ => Err(crate::error::InvalidDiscriminator.into()),
            }
        }
    };
}

/// Macro to define shank instruction enum variants
#[macro_export]
macro_rules! shank_instruction {
    (
        $name:ident {
            $(
                #[account($idx:literal, $($account_attr:tt)*)]
            )*
            data: {
                $(
                    $field:ident: $field_type:ty
                ),* $(,)?
            }
        }
    ) => {
        $(
            #[account($idx, $($account_attr)*)]
        )*
        $name {
            $(
                $field: $field_type,
            )*
        }
    };
}

/// Generate complete shank enum from instruction list  
#[macro_export]
macro_rules! define_program_instructions {
    (
        $(
            $variant:tt
        ),* $(,)?
    ) => {
        use shank::ShankInstruction;

        /// Program instructions for IDL generation
        #[repr(u8)]
        #[derive(Clone, Debug, PartialEq, ShankInstruction)]
        pub enum ProgramInstructions {
            $(
                $variant,
            )*
        }
    };
}
