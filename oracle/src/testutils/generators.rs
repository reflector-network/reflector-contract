extern crate alloc;
extern crate std;

use super::constants::RESOLUTION;
use crate::{
    mapping,
    types::{Asset, ConfigData, FeeConfig, PriceUpdate},
};
use alloc::string::ToString;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, Symbol, Vec};
use std::collections::VecDeque;

pub fn generate_update_record_mask(e: &Env, updates: &VecDeque<i128>) -> Bytes {
    let mut mask = [0u8; 32];
    for (asset, price) in updates.iter().enumerate() {
        if price > &0 {
            let (byte, bitmask) = mapping::resolve_period_update_mask_position(asset as u32);
            let i = byte as usize;
            let bytemask = mask[i] | bitmask;
            mask[i] = bytemask
        }
    }
    Bytes::from_array(e, &mask)
}

pub fn generate_test_env() -> (ConfigData, Env) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let config = ConfigData {
        admin: admin.clone(),
        history_retention_period: (100 * RESOLUTION).into(),
        assets: generate_assets(&env, 10, 0),
        base_asset: Asset::Stellar(Address::generate(&env)),
        decimals: 14,
        resolution: RESOLUTION,
        cache_size: 0,
        fee_config: FeeConfig::None,
    };
    (config, env)
}

pub fn generate_updates(
    env: &Env,
    assets: &Vec<Asset>,
    price: i128,
) -> (PriceUpdate, VecDeque<i128>) {
    let mut updates = VecDeque::new();
    let mut filtered_price = Vec::new(&env);
    for _ in assets.iter() {
        updates.push_back(price);
        filtered_price.push_back(price);
    }
    let mask = generate_update_record_mask(env, &updates);
    (
        PriceUpdate {
            prices: filtered_price,
            mask,
        },
        updates,
    )
}

fn get_random_bool() -> bool {
    //TODO: rewrite to use deterministic algo
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let random_bool = (nanos % 200) == 0;
    random_bool
}

pub fn generate_random_updates(
    env: &Env,
    assets: &Vec<Asset>,
    price: i128,
) -> (PriceUpdate, VecDeque<i128>) {
    let mut updates = VecDeque::new();
    let mut filtered_price = Vec::new(&env);
    for _ in assets.iter() {
        let price = if get_random_bool() { 0 } else { price };
        updates.push_back(price);
        if price > 0 {
            filtered_price.push_back(price);
        }
    }
    let mask = generate_update_record_mask(env, &updates);
    (
        PriceUpdate {
            prices: filtered_price,
            mask,
        },
        updates,
    )
}

pub fn generate_assets(e: &Env, count: usize, start_index: u32) -> Vec<Asset> {
    let mut assets = Vec::new(&e);
    for i in 0..count {
        if i % 2 == 0 {
            assets.push_back(Asset::Stellar(Address::generate(&e)));
        } else {
            assets.push_back(Asset::Other(Symbol::new(
                e,
                &("ASSET_".to_string() + &(start_index + i as u32).to_string()),
            )));
        }
    }
    assets
}
