use soroban_sdk::{contracttype, Address};

use super::asset_price_key::AssetPriceKey;

#[contracttype]
pub enum DataKey {
    Admin,
    Price(AssetPriceKey),
    ConfigVersion,
    LastTimestamp,
    RetentionPeriod,
    Assets,
    BaseFee,
    Balance(Address)
}
