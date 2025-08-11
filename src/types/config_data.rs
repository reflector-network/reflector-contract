use soroban_sdk::{contracttype, Address, Vec};

use crate::types::retention_config::RetentionConfig;

use super::asset::Asset;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]

// The configuration parameters for the contract.
pub struct ConfigData {
    // The admin address.
    pub admin: Address,
    // The history retention period for the prices.
    pub history_retention_period: u64,
    // The assets supported by the contract.
    pub assets: Vec<Asset>,
    // The base asset for the prices.
    pub base_asset: Asset,
    // The number of decimals for the prices.
    pub decimals: u32,
    // The resolution of the prices.
    pub resolution: u32,
    // The cache size for the prices.
    pub cache_size: u32,
    // The retention config for the contract. Token address and fee amount.
    pub retention_config: RetentionConfig,
}