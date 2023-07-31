use soroban_sdk::contracttype;

use super::asset::Asset;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceUpdateItem {
    pub asset: Asset,
    pub price: i128,
}