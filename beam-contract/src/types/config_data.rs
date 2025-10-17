use shared::types::{asset::Asset, fee_config::FeeConfig};
use soroban_sdk::{contracttype, Address, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
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
    pub retention_config: FeeConfig,
    /// Invocation fee
    pub invocation_config: FeeConfig,
}
