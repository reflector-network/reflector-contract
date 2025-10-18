use soroban_sdk::{contracterror, contracttype, Address, Bytes, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
// Quoted symbol descriptor
pub enum Asset {
    Stellar(Address), // Stellar asset contract address
    Other(Symbol),    // Symbol for all other external price sources
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
// Contract configuration parameters
pub struct ConfigData {
    // Admin address
    pub admin: Address,
    // Price history retention period
    pub history_retention_period: u64,
    // List of supported assets
    pub assets: Vec<Asset>,
    // Base asset
    pub base_asset: Asset,
    // Number of decimals for price records
    pub decimals: u32,
    // History timeframe resolution
    pub resolution: u32,
    // Number of rounds held in instance cache
    pub cache_size: u32,
    // Contract retention config
    pub fee_config: FeeConfig,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
// Oracle retention config containing fee asset and daily retention fee amount
pub enum FeeConfig {
    Some((Address, i128)),
    None,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
// Asset price data at specific timestamp
pub struct PriceData {
    // Price stored with configured decimals places
    pub price: i128,
    // Record timestamp
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
// Oracle price data at specific timestamp
pub struct PriceUpdate {
    // Prices for updated assets that have been updated
    pub prices: Vec<i128>,
    // Bitmap of updated asset positions
    pub mask: Bytes,
}

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
