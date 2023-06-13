use soroban_sdk::{contracttype};

use super::asset::Asset;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetPriceKey {
    pub asset: Asset,
    pub timestamp: u64,
}