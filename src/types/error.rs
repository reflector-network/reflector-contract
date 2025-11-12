use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
// The error codes for the contract.
pub enum Error {
    // The contract is already initialized.
    AlreadyInitialized = 0,
    // The caller is not authorized to perform the operation.
    Unauthorized = 1,
    // The config assets doen't contain persistent asset. Delete assets is not supported.
    AssetMissing = 2,
    // The asset is already added to the contract's list of supported assets.
    AssetAlreadyExists = 3,
    // The config version is invalid
    InvalidConfigVersion = 4,
    // The prices timestamp is invalid
    InvalidTimestamp = 5,
    // The assets update length or prices update length is invalid
    InvalidUpdateLength = 6,
    // The assets storage is full
    AssetLimitExceeded = 7,
    // The fxs storage is full
    FxLimitExceeded = 8,
    // The fx is already added to the contract's list of supported fxs.
    FxAlreadyExists = 9,
    // The fx price is stale
    StaleFxPrice = 10,
    // The assets and fxs arrays have mismatched lengths
    FxArrayLengthMismatch = 11,
    // The yield rate is invalid (must be >= 1.0 with matching decimals)
    InvalidYieldRate = 12,
    // The fx price is invalid (must be positive and non-zero)
    InvalidFxPrice = 13,
    // The fx oracle is unavailable or cannot be accessed
    FxOracleUnavailable = 14,
    // Integer overflow occurred during price calculation
    IntegerOverflow = 15,
    // The FX oracle timestamp has drifted too far from the contract's timestamp
    FxOracleTimestampDrift = 16,
    // The yield rate decreased when it should only increase or stay the same
    YieldRateDecreased = 17,
    // The yield rate increased by more than the maximum allowed deviation
    YieldRateDeviationExceeded = 18,
}
