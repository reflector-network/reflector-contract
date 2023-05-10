use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// The error codes for the contract.
pub enum Error {
    /// The caller is not authorized to perform the operation.
    Unauthorized = 1,
    /// The asset is already added to the contract's list of supported assets.
    AssetAlreadyPresented = 3,
    /// The updates length is not equal to the number of supported assets.
    InvalidUpdatesLength = 4,
    /// The price value is invalid (not i128)
    InvalidPriceValue = 5,
    /// If update price is 0, and there is no previous price
    NoPrevPrice = 6,
    /// Deposit in unsupported asset
    InvalidFeeAsset = 11,
    /// Deposit amount has negative value
    InvalidDepositAmount = 12,
    /// Consumer has insufficient balance to pay the fee
    InsufficientBalance = 13,
}