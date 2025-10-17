use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
// Standard contract errors
pub enum Error {
    // Contract already initialized
    AlreadyInitialized = 0,
    // Caller is not authorized to perform operation
    Unauthorized = 1,
    // Config asset list doesn't contain persistent asset
    AssetMissing = 2,
    // Asset is already exists in supported assets list
    AssetAlreadyExists = 3,
    // Config version is invalid
    InvalidConfigVersion = 4,
    // Price timestamp is invalid
    InvalidTimestamp = 5,
    // Maximum assets limit reached
    AssetLimitExceeded = 6,
    // Amount is invalid (negative or zero).
    InvalidAmount = 7,
    // Prices update is invalid
    InvalidPricesUpdate = 8,
}
