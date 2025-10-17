use soroban_sdk::{contracttype, Bytes, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
// Asset price data at specific timestamp
pub struct TimestampPrices {
    // Prices for assets that have been updated
    pub prices: Vec<i128>,
    // Bitmap of assets that have been updated
    pub mask: Bytes,
}
