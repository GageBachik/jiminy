// Define errors using the define_errors! macro
// This will be parsed by the build script and generated in generated.rs
define_errors! {
    CounterProgramError,
    InvalidDiscriminator = 6001,
    Unauthorized = 6002,
    CounterKeyIncorrect = 6003,
    CounterAlreadyInitialized = 6004,
    CounterNotInitialized = 6005,
    CounterUnderflow = 6006,
}