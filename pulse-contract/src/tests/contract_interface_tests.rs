#![cfg(test)]

use crate::tests::setup_tests::{
    convert_to_seconds, generate_random_updates, generate_updates, init_contract, normalize_price,
};
use oracle::prices::{self, PRICE_RECORDS_LIMIT};
use oracle::types::FeeConfig;
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

#[test_case(2, Some(normalize_price(1)) ; "twap 2 rounds")]
#[test_case(PRICE_RECORDS_LIMIT + 1, None ; "twap exceeds limit")]
fn x_twap_test(records: u32, price: Option<i128>) {
    let (env, client, init_data) = init_contract();

    let assets = init_data.assets;

    //set prices for assets
    let timestamp = 600_000;
    let updates = generate_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = generate_updates(&env, &assets, normalize_price(200));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let result = client.x_twap(&assets.get_unchecked(1), &assets.get_unchecked(2), &records);

    assert_eq!(result, price);
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

    //set more than 255 prices to check that history mask is overwritten correctly
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
                if price_data.is_none() {
                    log!(
                        &env,
                        "Verifying asset {} at timestamp {}: expected price {:?}",
                        asset_index as u64,
                        timestamp,
                        expected_price
                    );
                    panic!(
                        "Expected price for asset {} at timestamp {}",
                        asset_index, timestamp
                    );
                }
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
