use soroban_sdk::{contracttype, Address};

use super::asset_price_key::AssetPriceKey;

#[contracttype]
pub enum DataKey {
    Admin,
    Price(AssetPriceKey),
    LastTimestamp,
    RetentionPeriod,
    Assets,
    BaseFee,
    Balance(Address)
}
