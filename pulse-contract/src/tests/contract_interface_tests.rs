#![cfg(test)]

use crate::tests::setup_tests::{
    convert_to_seconds, generate_random_updates, generate_updates, init_contract, normalize_price,
};
use oracle::prices::{self};
use oracle::types::{FeeConfig, PriceData};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::{log, Address, Env, Vec};
use test_case::test_case;

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
fn lastprice_test() {
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

#[test_case(255, "gap 255")]
#[test_case(256, "gap 256")]
#[test_case(257, "gap 257")]
#[test_case(1000, "gap 1000")]
fn prices_update_test(gap: u64, _description: &str) {
    let (env, client, init_data) = init_contract();

    let assets = init_data.assets;

    client.set_cache_size(&3);

    let mut history_prices = Vec::new(&env);

    //set more than 256 prices to check that history mask is overwritten correctly
    for i in 0..(gap + 256) {
        let timestamp = 600_000 + i * 300_000;

        if i < 1 || i > gap {
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
    //prepare an array with zero prices
    let mut zero_prices = Vec::new(&env);
    for _ in 0..assets.len() {
        zero_prices.push_back(0i128);
    }

    //verify
    let mut had_gaps = false;
    let mut had_prices = false;
    let mut iterations = 0;

    for (history_index, (timestamp, updates)) in history_prices.iter().enumerate() {
        let all_prices;
        if history_index > 255 {
            all_prices = zero_prices.clone();
        } else {
            let total = assets.len() + 10; //+10 to check that out of range assets are ignored
                                           //get records from generated updates
            all_prices = prices::extract_update_record_prices(&env, &updates, total);
        }

        //match price with mask for each asset in update
        for (asset_index, asset) in assets.iter().enumerate() {
            //get oracle-quoted price
            let oracle_price = client.price(&asset, &(timestamp / 1000));
            //get expected price (from generated data)
            let expected_price = all_prices.get(asset_index as u32).unwrap_or_default();
            if expected_price > 0 {
                let price = oracle_price.unwrap_or_else(|| PriceData {
                    price: 0,
                    timestamp: 0,
                });
                assert_eq!(
                    price.price, expected_price,
                    "asset {} at timestamp {}",
                    asset_index, timestamp
                );
                assert_eq!(price.timestamp, convert_to_seconds(timestamp));
                had_prices = true;
            } else {
                assert!(
                    oracle_price.is_none(),
                    "asset {} at timestamp {}",
                    asset_index,
                    timestamp
                );
                had_gaps = true;
            }
        }
        iterations += 1;
    }
    assert!(had_prices);
    assert!(had_gaps);
    log!(&env, "{} iterations", iterations);
}

#[test]
fn extend_asset_ttl_test() {
    let (env, client, init_data) = init_contract();

    env.mock_all_auths();

    let fee_asset = env
        .register_stellar_asset_contract_v2(init_data.admin.clone())
        .address();
    let fee_config = FeeConfig::Some((fee_asset.clone(), 1_000_000));
    client.set_fee_config(&fee_config);

    //generate sponsor and mint fee tokens
    let sponsor = Address::generate(&env);
    let token_client = StellarAssetClient::new(&env, &fee_asset);
    token_client.mint(&sponsor, &10_000_000);

    //get initial expiration
    let asset = &init_data.assets.first_unchecked();
    let initial_expiration = client.expires(&asset).unwrap();

    //extend TTL by 10 day (864000 seconds)
    client.extend_asset_ttl(&sponsor, &asset, &10_000_000);

    //verify new expiration
    let new_expiration = client.expires(&asset).unwrap();
    assert_eq!(new_expiration, initial_expiration + 864000);
}
