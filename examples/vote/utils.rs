pub fn calculate_fees(amount: u64, bps: u16) -> u64 {
    amount * bps as u64 / 10_000
}
