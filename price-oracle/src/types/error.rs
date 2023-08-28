use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// The error codes for the contract.
pub enum Error {
    /// The contract is already initialized.
    AlreadyInitialized = 0,
    /// The caller is not authorized to perform the operation.
    Unauthorized = 1,
    /// The config assets doen't contain persistent asset. Delete assets is not supported.
    AssetMissing = 2,
    /// The asset is already added to the contract's list of supported assets.
    AssetAlreadyPresented = 3,
    /// The config version is invalid
    InvalidConfigVersion = 4,
    /// Deposit in unsupported asset
    InvalidFeeAsset = 11,
    /// Deposit amount has negative value
    InvalidDepositAmount = 12,
    /// Consumer has insufficient balance to pay the fee
    InsufficientBalance = 13,
}