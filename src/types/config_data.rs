use soroban_sdk::{contracttype, Address, Vec};

use super::asset::Asset;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]

/// The configuration parameters for the contract.
pub struct ConfigData {
    /// The admin address.
    pub admin: Address,
    /// The retention period for the prices.
    pub period: u64,
    /// The assets supported by the contract.
    pub assets: Vec<Asset>,
    /// The base asset for the prices.
    pub base_asset: Asset,
    /// The number of decimals for the prices.
    pub decimals: u32,
    /// The resolution of the prices.
    pub resolution: u32
}
