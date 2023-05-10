use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetPriceKey {
    pub asset: Address,
    pub timestamp: u64,
}