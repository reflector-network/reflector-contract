use soroban_sdk::{contracttype, Address};

use super::asset::Asset;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]

// The configuration parameters for the contract.
pub struct ConfigData {
    // The admin address.
    pub admin: Address,
    // The retention period for the prices.
    pub period: u64,
    // The base asset for the prices.
    pub base_asset: Asset,
    // The number of decimals for the prices.
    pub decimals: u32,
    // The resolution of the prices.
    pub resolution: u32,
    // The FX oracle contract address (immutable after initialization).
    pub fx_oracle_address: Address,
    // Maximum allowed yield rate increase as a percentage (e.g., 1 = 1%, 10 = 10%)
    pub max_yield_deviation_percent: u32,
}
