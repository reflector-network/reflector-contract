#![cfg(test)]
extern crate alloc;
extern crate std;


use shared::prices;
use shared::types::{asset::Asset, fee_config::FeeConfig};
use soroban_sdk::testutils::{Address as _, Events, Ledger, LedgerInfo, MockAuth, MockAuthInvoke};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{symbol_short, Address, Env, IntoVal, String, Symbol, TryIntoVal, Vec};
use std::panic::{self, AssertUnwindSafe};
use alloc::string::ToString;

use crate::types::config_data::ConfigData;
use crate::{PriceOracleContract, PriceOracleContractClient};

const RESOLUTION: u32 = 300_000;
const DECIMALS: u32 = 14;

fn convert_to_seconds(timestamp: u64) -> u64 {
    timestamp / 1000
}

fn get_random_bool() -> bool {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let random_bool = (nanos % 200) == 0;
    random_bool
}

fn get_updates_with_random(env: &Env, assets: &Vec<Asset>, price: i128) -> Vec<i128> {
    let mut updates = Vec::new(&env);
    for _ in assets.iter() {
        let price = if get_random_bool() {
            0
        } else {
            price
        };
        updates.push_back(price);
    }
    updates
}

fn init_contract_with_admin<'a>() -> (Env, PriceOracleContractClient<'a>, ConfigData) {
    let env = Env::default();

    //set timestamp to 900 seconds
    let ledger_info = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp: 900,
        ..ledger_info
    });

    let admin = Address::generate(&env);

    let contract_id = &Address::from_string(&String::from_str(
        &env,
        "CDXHQTB7FGRMWTLJJLNI3XPKVC6SZDB5SFGZUYDPEGQQNC4G6CKE4QRC",
    ));

    env.register_at(contract_id, PriceOracleContract, ());
    let client = PriceOracleContractClient::new(&env, contract_id);

    env.cost_estimate().budget().reset_unlimited();

    let init_data = ConfigData {
        admin: admin.clone(),
        history_retention_period: (100 * RESOLUTION).into(),
        assets: generate_assets(&env, 10, 0),
        base_asset: Asset::Stellar(Address::generate(&env)),
        decimals: 14,
        resolution: RESOLUTION,
        cache_size: 0,
        retention_config: FeeConfig::None
    };

    env.mock_all_auths();

    //set admin
    client.config(&init_data);

    (env, client, init_data)
}

fn normalize_price(price: i128) -> i128 {
    price * 10i128.pow(DECIMALS)
}

fn generate_assets(e: &Env, count: usize, start_index: u32) -> Vec<Asset> {
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

fn get_updates(env: &Env, assets: &Vec<Asset>, price: i128) -> Vec<i128> {
    let mut updates = Vec::new(&env);
    for _ in assets.iter() {
        updates.push_back(price);
    }
    updates
}

#[test]
fn version_test() {
    let (_env, client, _init_data) = init_contract_with_admin();
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
fn init_test() {
    let (_env, client, init_data) = init_contract_with_admin();

    let address = client.admin();
    assert_eq!(address.unwrap(), init_data.admin.clone());

    let base = client.base();
    assert_eq!(base, init_data.base_asset);

    let resolution = client.resolution();
    assert_eq!(resolution, convert_to_seconds(RESOLUTION.into()) as u32);

    let period = client.history_retention_period().unwrap();
    assert_eq!(period, convert_to_seconds(init_data.history_retention_period));

    let decimals = client.decimals();
    assert_eq!(decimals, DECIMALS);

    let assets = client.assets();
    assert_eq!(assets, init_data.assets);
}

#[test]
fn set_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    assert_eq!(
        env.events().all().last().unwrap().1,
        (
            symbol_short!("REFLECTOR"),
            symbol_short!("update"),
            &600_000u64
        )
            .into_val(&env)
    );
}

#[test]
#[should_panic]
fn set_price_zero_timestamp_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 0;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);
}

#[test]
#[should_panic]
fn set_price_invalid_timestamp_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 600_001;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);
}

#[test]
#[should_panic]
fn set_price_future_timestamp_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 1_200_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);
}

#[test]
fn last_timestamp_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let mut result = client.last_timestamp();

    assert_eq!(result, 0);

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    result = client.last_timestamp();

    assert_eq!(result, convert_to_seconds(600_000));
}

#[test]
fn add_assets_test() {
    let (env, client, init_data) = init_contract_with_admin();

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
    let (env, client, _) = init_contract_with_admin();

    let mut assets = Vec::new(&env);
    let duplicate_asset = Asset::Other(Symbol::new(&env, &("ASSET_DUPLICATE")));
    assets.push_back(duplicate_asset.clone());
    assets.push_back(duplicate_asset);

    env.mock_all_auths();

    client.add_assets(&assets);
}

#[test]
#[should_panic]
fn assets_update_overflow_test() {
    let (env, client, _) = init_contract_with_admin();

    env.mock_all_auths();

    env.cost_estimate().budget().reset_unlimited();

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
fn prices_update_overflow_test() {
    let (env, client, _) = init_contract_with_admin();

    env.mock_all_auths();

    env.cost_estimate().budget().reset_unlimited();

    let mut updates = Vec::new(&env);
    for i in 1..=256 {
        updates.push_back(normalize_price(i as i128 + 1));
    }
    client.set_price(&updates, &600_000);
}

#[test]
fn set_period_test() {
    let (env, client, _) = init_contract_with_admin();

    let period = 100_000;

    env.mock_all_auths();

    client.set_history_retention_period(&period);

    let result = client.history_retention_period().unwrap();

    assert_eq!(result, convert_to_seconds(period));
}

#[test]
fn authorized_test() {
    let (env, client, config_data) = init_contract_with_admin();

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
fn unauthorized_test() {
    let (env, client, _) = init_contract_with_admin();

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

#[test]
fn div_tests() {
    let test_cases = [
        (154467226919499, 133928752749774, 115335373284703),
        (
            i128::MAX / 100,
            231731687303715884105728,
            734216306110962248249052545,
        ),
        (231731687303715884105728, i128::MAX / 100, 13),
        // -1 expected result for errors
        (1, 0, -1),
        (0, 1, -1),
        (0, 0, -1),
        (-1, 0, -1),
        (0, -1, -1),
        (-1, -1, -1),
    ];

    for (a, b, expected) in test_cases.iter() {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            prices::fixed_div_floor(a.clone(), *b, 14)
        }));
        if expected == &-1 {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), *expected);
        }
    }
}

#[test]
fn set_retention_config_test() {
    let (env, client, init_data) = init_contract_with_admin();

    //emulate old contract state
    env.as_contract(&client.address, || {
        env.storage().instance().remove(&"retention");
        env.storage().instance().remove(&"expiration");
    });

    //create fee asset token
    let fee_asset = env.register_stellar_asset_contract_v2(init_data.admin.clone());

    let retention_config = FeeConfig::Some((fee_asset.address(), 7));

    client.set_retention_config(&retention_config);

    let result = client.retention_config();
    assert_ne!(result, FeeConfig::None);
    assert_eq!(result, retention_config);

    let asset: Asset = init_data.assets.get_unchecked(0);

    let expires = client.expires(&asset);
    assert!(expires.is_some());

    let sponsor = Address::generate(&env);
    let fee_token = StellarAssetClient::new(&env, &fee_asset.address());
    fee_token.mint(&sponsor, &10);

    let symbol_expires = client.expires(&asset).unwrap();
    client.extend_asset_ttl(&sponsor, &asset, &10);
    assert_eq!(client.expires(&asset).unwrap(), symbol_expires + 123428571); //123428571 ms you get for 9 XRF tokens

    let fee_token_balance = TokenClient::new(&env, &fee_asset.address()).balance(&sponsor);
    assert_eq!(fee_token_balance, 0); //1 XRF token is left after paying the fee
}



#[test]
fn set_price_prices() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    client.set_cache_size(&256);

    let mut history_prices = Vec::new(&env);

    //set more than 255 prices to check history is overritten correctly
    for i in 0..257 {
        let timestamp = 600_000 + i * 300_000;

        if timestamp != 900_000 && timestamp != 1200_000 {
            let updates = get_updates_with_random(&env, &assets, normalize_price(100));
            history_prices.push_front((timestamp, updates.clone()));
            //set prices for assets
            client.set_price(&updates, &timestamp);
        } else {
            //simulate time passage without setting prices to create gaps in updates
            let updates = get_updates_with_random(&env, &assets, 0);
            history_prices.push_front((timestamp, updates.clone()));
        }
        let ledger_info = env.ledger().get();
        env.ledger().set(LedgerInfo {
            timestamp: convert_to_seconds(timestamp) + 300,
            ..ledger_info
        });
    }

    //verify prices
    for (history_index, (timestamp, updates)) in history_prices.iter().enumerate() {
        if history_index > 255 {
            break;
        }
        for (asset_index, asset) in assets.iter().enumerate() {
            let price_data = client.price(&asset, &convert_to_seconds(timestamp));
            let expected_price = updates.get(asset_index as u32).unwrap_or_default();
            if expected_price > 0 {
                let price = price_data.unwrap();
                assert_eq!(price.price, expected_price);
                assert_eq!(price.timestamp, convert_to_seconds(timestamp));
            } else {
                assert!(price_data.is_none());
            }
        }
    }
}