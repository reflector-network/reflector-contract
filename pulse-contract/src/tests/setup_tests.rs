#![cfg(test)]
extern crate alloc;
extern crate std;

use crate::{PulseOracleContract, PulseOracleContractClient};
use alloc::string::ToString;
use oracle::types::{Asset, ConfigData, FeeConfig, PriceUpdate};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{Address, Bytes, Env, String, Symbol, Vec};

pub(super) const RESOLUTION: u32 = 300_000;
pub(super) const DECIMALS: u32 = 14;

pub(super) fn init_contract<'a>() -> (Env, PulseOracleContractClient<'a>, ConfigData) {
    let env = Env::default();

    //set timestamp to 900 seconds
    let ledger_info = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp: 900,
        ..ledger_info
    });

    let contract_id = &Address::from_string(&String::from_str(
        &env,
        "CDXHQTB7FGRMWTLJJLNI3XPKVC6SZDB5SFGZUYDPEGQQNC4G6CKE4QRC",
    ));

    env.register_at(contract_id, PulseOracleContract, ());
    let client = PulseOracleContractClient::new(&env, contract_id);

    env.cost_estimate().budget().reset_unlimited();

    env.mock_all_auths();
    let init_data = prepare_contract_config(&env);
    client.config(&init_data);

    (env, client, init_data)
}

fn prepare_contract_config(env: &Env) -> ConfigData {
    let admin = Address::generate(&env);
    ConfigData {
        admin: admin.clone(),
        history_retention_period: (100 * RESOLUTION).into(),
        assets: generate_assets(&env, 10, 0),
        base_asset: Asset::Stellar(Address::generate(&env)),
        decimals: 14,
        resolution: RESOLUTION,
        cache_size: 0,
        fee_config: FeeConfig::None,
    }
}

pub(super) fn convert_to_seconds(timestamp: u64) -> u64 {
    timestamp / 1000
}

pub(super) fn normalize_price(price: i128) -> i128 {
    price * 10i128.pow(DECIMALS)
}

pub(super) fn generate_update_record_mask(e: &Env, updates: &Vec<i128>) -> Bytes {
    let mut mask = [0u8; 32];
    for (asset_index, price) in updates.iter().enumerate() {
        if price > 0 {
            let (byte, bitmask) =
                oracle::mapping::resolve_period_update_mask_position(asset_index as u32);
            let i = byte as usize;
            let bytemask = mask[i] | bitmask;
            mask[i] = bytemask
        }
    }
    Bytes::from_array(e, &mask)
}

pub(super) fn generate_updates(env: &Env, assets: &Vec<Asset>, price: i128) -> PriceUpdate {
    let mut updates = Vec::new(&env);
    for _ in assets.iter() {
        updates.push_back(price);
    }
    let mask = generate_update_record_mask(env, &updates);
    PriceUpdate {
        prices: updates,
        mask,
    }
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

pub(super) fn generate_random_updates(env: &Env, assets: &Vec<Asset>, price: i128) -> PriceUpdate {
    let mut updates = Vec::new(&env);
    for _ in assets.iter() {
        let price = if get_random_bool() { 0 } else { price };
        updates.push_back(price);
    }
    let mask = generate_update_record_mask(env, &updates);
    PriceUpdate {
        prices: updates,
        mask,
    }
}

pub(super) fn generate_assets(e: &Env, count: usize, start_index: u32) -> Vec<Asset> {
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
