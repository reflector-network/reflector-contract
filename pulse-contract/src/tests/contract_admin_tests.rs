#![cfg(test)]
extern crate alloc;
extern crate std;

use alloc::string::ToString;
use oracle::init_contract_with_admin;
use oracle::testutils::{
    convert_to_seconds, generate_assets, generate_update_record_mask, generate_updates,
    normalize_price, DECIMALS, RESOLUTION,
};
use oracle::types::{Asset, FeeConfig, PriceUpdate};
use soroban_sdk::testutils::{Address as _, Events, MockAuth, MockAuthInvoke};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Event, Symbol, TryIntoVal, Vec};

use crate::{PulseOracleContract, PulseOracleContractClient};

#[test]
fn init_test() {
    let (_env, client, init_data) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    let address = client.admin();
    assert_eq!(address.unwrap(), init_data.admin.clone());

    let base = client.base();
    assert_eq!(base, init_data.base_asset);

    let resolution = client.resolution();
    assert_eq!(resolution, convert_to_seconds(RESOLUTION.into()) as u32);

    let period = client.history_retention_period().unwrap();
    assert_eq!(
        period,
        convert_to_seconds(init_data.history_retention_period)
    );

    let decimals = client.decimals();
    assert_eq!(decimals, DECIMALS);

    let assets = client.assets();
    assert_eq!(assets, init_data.assets);
}

#[test]
fn set_price_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = generate_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates.0, &timestamp);

    //build expected event
    let expected_event = oracle::events::UpdateEvent {
        timestamp: 600_000,
        update_data: {
            let mut upd = Vec::new(&env);
            for asset in assets.iter() {
                let asset_val = match asset {
                    Asset::Stellar(address) => address.to_val(),
                    Asset::Other(symbol) => symbol.to_val(),
                };
                upd.push_back((asset_val, normalize_price(100)));
            }
            upd
        },
    };
    assert_eq!(
        env.events().all().events().last().unwrap(),
        &expected_event.to_xdr(&env, &client.address)
    );
}

#[test]
#[should_panic]
fn set_price_zero_timestamp_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    let assets = init_data.assets;

    let timestamp = 0;
    let updates = generate_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates.0, &timestamp);
}

#[test]
#[should_panic]
fn set_price_invalid_timestamp_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    let assets = init_data.assets;

    let timestamp = 600_001;
    let updates = generate_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates.0, &timestamp);
}

#[test]
#[should_panic]
fn set_price_future_timestamp_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    let assets = init_data.assets;

    let timestamp = 1_200_000;
    let updates = generate_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates.0, &timestamp);
}

#[test]
fn add_assets_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    let assets = generate_assets(&env, 10, init_data.assets.len() - 1);

    env.mock_all_auths();

    client.add_assets(&assets);

    let result = client.assets();

    let mut expected_assets = init_data.assets.clone();
    for asset in assets.iter() {
        expected_assets.push_back(asset.clone());
    }

    assert_eq!(result, expected_assets);
}

#[test]
#[should_panic]
fn add_assets_duplicate_test() {
    let (env, client, _) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    let mut assets = Vec::new(&env);
    let duplicate_asset = Asset::Other(Symbol::new(&env, &("ASSET_DUPLICATE")));
    assets.push_back(duplicate_asset.clone());
    assets.push_back(duplicate_asset);

    env.mock_all_auths();

    client.add_assets(&assets);
}

#[test]
#[should_panic]
fn asset_update_overflow_test() {
    let (env, client, _) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    env.mock_all_auths();

    let mut assets = Vec::new(&env);
    for i in 1..=1000 {
        assets.push_back(Asset::Other(Symbol::new(
            &env,
            &("Asset".to_string() + &i.to_string()),
        )));
    }

    client.add_assets(&assets);
}

#[test]
#[should_panic]
fn price_update_overflow_test() {
    let (env, client, _) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    env.mock_all_auths();

    let mut raw_prices = std::collections::VecDeque::new();
    for i in 1..=256 {
        raw_prices.push_back(normalize_price(i as i128 + 1));
    }
    let mask = generate_update_record_mask(&env, &raw_prices);
    let update = PriceUpdate {
        prices: Vec::from_iter(&env, raw_prices.into_iter()),
        mask,
    };
    client.set_price(&update, &600_000);
}

#[test]
fn set_history_retention_period_test() {
    let (env, client, _) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    let period = 100_000;

    env.mock_all_auths();

    client.set_history_retention_period(&period);

    let result = client.history_retention_period().unwrap();

    assert_eq!(result, convert_to_seconds(period));
}

#[test]
fn set_fee_config_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    //emulate old contract state
    env.as_contract(&client.address, || {
        env.storage().instance().remove(&"retention");
        env.storage().instance().remove(&"expiration");
    });

    //create fee asset token
    let fee_asset = env.register_stellar_asset_contract_v2(init_data.admin.clone());

    let fee_config = FeeConfig::Some((fee_asset.address(), 7));

    client.set_fee_config(&fee_config); //3 days

    let result = client.fee_config();
    assert_ne!(result, FeeConfig::None);
    assert_eq!(result, fee_config);

    let asset: Asset = init_data.assets.get_unchecked(0);

    let expires = client.expires(&asset);
    assert!(expires.is_some());

    let sponsor = Address::generate(&env);
    let fee_token = StellarAssetClient::new(&env, &fee_asset.address());
    fee_token.mint(&sponsor, &10);

    let symbol_expires = client.expires(&asset).unwrap();
    assert_eq!(symbol_expires, 15552900); // 900s current ledger timestamp + 180 days of initial expiration period
    client.extend_asset_ttl(&sponsor, &asset, &10);
    //123428571 ms you get for 10 XRF tokens
    assert_eq!(
        client.expires(&asset).unwrap(),
        symbol_expires + 123428571 / 1000
    );

    let fee_token_balance = TokenClient::new(&env, &fee_asset.address()).balance(&sponsor);
    assert_eq!(fee_token_balance, 0);
}

#[test]
fn authorization_successful_test() {
    let (env, client, config_data) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    let period: u64 = 100;
    //set prices for assets
    client
        .mock_auths(&[MockAuth {
            address: &config_data.admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "set_history_retention_period",
                args: Vec::from_array(&env, [period.clone().try_into_val(&env).unwrap()]),
                sub_invokes: &[],
            },
        }])
        .set_history_retention_period(&period);
}

#[test]
#[should_panic]
fn authorization_failed_test() {
    let (env, client, _) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);
    let account = Address::generate(&env);

    let period: u64 = 100;
    //set prices for assets
    client
        .mock_auths(&[MockAuth {
            address: &account,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "set_period",
                args: Vec::from_array(&env, [period.clone().try_into_val(&env).unwrap()]),
                sub_invokes: &[],
            },
        }])
        .set_history_retention_period(&period);
}
