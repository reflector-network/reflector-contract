#![cfg(test)]
extern crate std;

use crate::tests::setup_tests::{
    convert_to_seconds, generate_random_updates, generate_updates, init_contract, normalize_price,
};
use oracle::prices;
use oracle::types::FeeConfig;
use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::Vec;

#[test]
fn version_test() {
    let (_env, client, _) = init_contract();
    let result = client.version();
    let version = env!("CARGO_PKG_VERSION")
        .split(".")
        .next()
        .unwrap()
        .parse::<u32>()
        .unwrap();
    assert_eq!(result, version);
}

#[test]
fn last_timestamp_test() {
    let (env, client, init_data) = init_contract();

    let assets = init_data.assets;

    let mut result = client.last_timestamp();

    assert_eq!(result, 0);

    let timestamp = 600_000;
    let updates = generate_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    result = client.last_timestamp();

    assert_eq!(result, convert_to_seconds(600_000));
}

#[test]
fn price_test() {
    let (env, client, init_data) = init_contract();

    let assets = &init_data.assets;

    let timestamp = 600_000;
    let updates = generate_updates(&env, assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let fee_asset = env
        .register_stellar_asset_contract_v2(init_data.admin.clone())
        .address();
    let fee_config = FeeConfig::Some((fee_asset.clone(), 1_000_000));
    client.set_fee_config(&fee_config);

    //get price for the first asset
    let price = client
        .lastprice(&init_data.assets.first_unchecked())
        .unwrap();
    assert_eq!(price.price, normalize_price(100));
    assert_eq!(price.timestamp, convert_to_seconds(timestamp));
}

#[test]
fn prices_test() {
    let (env, client, init_data) = init_contract();

    let assets = init_data.assets;

    client.set_cache_size(&256);

    let mut history_prices = Vec::new(&env);

    //set more than 255 prices to check that history mask is overwritten correctly
    for i in 0..257 {
        let timestamp = 600_000 + i * 300_000;

        if timestamp != 900_000 && timestamp != 1200_000 {
            let updates = generate_random_updates(&env, &assets, normalize_price(100));
            history_prices.push_front((timestamp, updates.clone()));
            //set prices for assets
            client.set_price(&updates, &timestamp);
        } else {
            //simulate time passage without setting prices to create gaps in updates
            let updates = generate_random_updates(&env, &assets, 0);
            history_prices.push_front((timestamp, updates.clone()));
        }
        let ledger_info = env.ledger().get();
        env.ledger().set(LedgerInfo {
            timestamp: timestamp / 1000 + 300,
            ..ledger_info
        });
    }

    let mut had_gaps = false;
    let mut had_prices = false;
    //verify prices
    for (history_index, (timestamp, updates)) in history_prices.iter().enumerate() {
        if history_index > 255 {
            break;
        }
        let total = assets.len() + 10; //+10 to check that out of range assets are ignored
        let all_prices = prices::extract_update_record_prices(&env, &updates, total);
        for (asset_index, asset) in assets.iter().enumerate() {
            let price_data = client.price(&asset, &(timestamp / 1000));
            let expected_price = all_prices.get(asset_index as u32).unwrap_or_default();
            if expected_price > 0 {
                let price = price_data.unwrap();
                assert_eq!(price.price, expected_price);
                assert_eq!(price.timestamp, convert_to_seconds(timestamp));
                had_prices = true;
            } else {
                assert!(price_data.is_none());
                had_gaps = true;
            }
        }
    }
    assert!(had_prices);
    assert!(had_gaps);
}
