use soroban_sdk::{contracttype, BytesN};

use super::asset_price_key::AssetPriceKey;

#[contracttype]
pub enum DataKey {
    Admin,
    Price(AssetPriceKey),
    LastTimestamp,
    RetentionPeriod,
    Assets,
    BaseFee,
    Balance(BytesN<32>)
}
