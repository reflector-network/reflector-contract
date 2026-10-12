use crate::settings;
use soroban_sdk::Env;

// UNIX timestamp of 3000-01-01T00:00:00Z (in milliseconds) - used as the expiration marker for the entries that never expire
pub const DISTANT_FUTURE: u64 = 32_503_680_000_000;

// Normalize timestamp trimming it to the timeframe resolution defined in settings
pub fn normalize(e: &Env, value: u64) -> u64 {
    let timeframe = settings::get_resolution(e) as u64;
    if value == 0 || timeframe == 0 {
        return 0;
    }
    (value / timeframe) * timeframe
}

// Whether the timestamp is valid
pub fn is_valid(e: &Env, value: u64) -> bool {
    value > 0 && value == normalize(e, value)
}

// Convert days to milliseconds
pub fn days_to_milliseconds(days: u32) -> u64 {
    (days as u64) * 24 * 60 * 60 * 1000
}

// Get timestamp for current ledger
pub fn ledger_timestamp(e: &Env) -> u64 {
    e.ledger().timestamp() * 1000 //convert to milliseconds
}
