// Define errors using the define_errors! macro
// This will be parsed by the build script and generated in generated.rs
define_errors! {
    PTokenProgramError,
    InvalidDiscriminator = 6001,
    PlatformKeyIncorrect = 6002,
    VaultKeyIncorrect = 6003,
    VoteVaultKeyIncorrect = 6004,
    PositionKeyIncorrect = 6005,
    VoteVaultTokenAccountIncorrect = 6006,
    VoteHasAlreadyEnded = 6007,
    VoteIsStillRunning = 6008,
    VoteWasTied = 6009,
    DidNotVoteForWinningSide = 6010,
}
