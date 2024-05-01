#![cfg(test)]
extern crate alloc;
extern crate std;

use super::*;
use alloc::string::ToString;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo, MockAuth, MockAuthInvoke},
    Address, Env, String, Symbol, TryIntoVal,
};
use std::panic::{self, AssertUnwindSafe};

use {extensions::i128_extensions::I128Extensions, types::asset::Asset};

const RESOLUTION: u32 = 300_000;
const DECIMALS: u32 = 14;

fn convert_to_seconds(timestamp: u64) -> u64 {
    timestamp / 1000
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

    env.register_contract(contract_id, PriceOracleContract);
    let client: PriceOracleContractClient<'a> = PriceOracleContractClient::new(&env, contract_id);

    env.budget().reset_unlimited();

    let init_data = ConfigData {
        admin: admin.clone(),
        period: (100 * RESOLUTION).into(),
        assets: generate_assets(&env, 10, 0),
        base_asset: Asset::Stellar(Address::generate(&env)),
        decimals: 14,
        resolution: RESOLUTION,
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
    assert_eq!(resolution, RESOLUTION / 1000);

    let period = client.period().unwrap();
    assert_eq!(period, init_data.period / 1000);

    let decimals = client.decimals();
    assert_eq!(decimals, DECIMALS);

    let assets = client.assets();
    assert_eq!(assets, init_data.assets);
}

#[test]
fn bump_test() {
    let (_env, client, _init_data) = init_contract_with_admin();

    client.bump(&6_000_000);
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
fn last_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &&assets, normalize_price(200));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    //check last prices
    let result = client.lastprice(&assets.get_unchecked(1));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(200),
            timestamp: convert_to_seconds(900_000)
        })
    );
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

    env.budget().reset_unlimited();

    let mut assets = Vec::new(&env);
    for i in 1..=256 {
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

    env.budget().reset_unlimited();

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

    client.set_period(&period);

    let result = client.period().unwrap();

    assert_eq!(result, convert_to_seconds(period));
}

#[test]
fn get_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(200));

    client.set_price(&updates, &timestamp);

    //check last prices
    let mut result = client.lastprice(&assets.get_unchecked(1));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(200),
            timestamp: convert_to_seconds(900_000)
        })
    );

    //check price at 899_000
    result = client.price(&assets.get_unchecked(1), &convert_to_seconds(899_000));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(100),
            timestamp: convert_to_seconds(600_000)
        })
    );
}

#[test]
fn get_lastprice_delayed_update_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 300_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    client.set_price(&updates, &timestamp);

    //check last prices
    let result = client.lastprice(&assets.get_unchecked(1));
    assert_eq!(result, None);
}

#[test]
fn get_x_last_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    client.set_price(&updates, &timestamp);

    //check last x price
    let result = client.x_last_price(&assets.get_unchecked(1), &assets.get_unchecked(2));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(1),
            timestamp: convert_to_seconds(600_000)
        })
    );
}

#[test]
fn get_x_price_with_zero_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 600_000;
    let mut updates = get_updates(&env, &assets, normalize_price(100));
    updates.set(1, 0);

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let result = client.x_price(
        &assets.get(0).unwrap(),
        &assets.get(1).unwrap(),
        &convert_to_seconds(timestamp),
    );

    assert_eq!(result, None);
}

#[test]
fn get_x_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(200));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    //check last prices
    let mut result = client.x_last_price(&assets.get_unchecked(1), &assets.get_unchecked(2));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(1),
            timestamp: convert_to_seconds(900_000)
        })
    );

    //check price at 899_000
    result = client.x_price(
        &assets.get_unchecked(1),
        &assets.get_unchecked(2),
        &convert_to_seconds(899_000),
    );
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(1),
            timestamp: convert_to_seconds(600_000)
        })
    );
}

#[test]
fn twap_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(200));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let result = client.twap(&assets.get_unchecked(1), &2);

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(150));
}

#[test]
fn x_twap_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    //set prices for assets
    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(200));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let result = client.x_twap(&assets.get_unchecked(1), &assets.get_unchecked(2), &2);

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(1));
}

#[test]
#[should_panic]
fn x_twap_with_gap_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    //set prices for assets with gap
    let timestamp = 300_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(200));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let result = client.x_twap(&assets.get_unchecked(1), &assets.get_unchecked(2), &3);

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(1));
}

#[test]
fn get_non_registered_asset_price_test() {
    let (env, client, config_data) = init_contract_with_admin();

    //try to get price for unknown Stellar asset
    let mut result = client.lastprice(&Asset::Stellar(Address::generate(&env)));
    assert_eq!(result, None);

    //try to get price for unknown Other asset
    result = client.lastprice(&Asset::Other(Symbol::new(&env, "NonRegisteredAsset")));
    assert_eq!(result, None);

    //try to get price for unknown base asset
    result = client.x_last_price(
        &Asset::Stellar(Address::generate(&env)),
        &config_data.assets.get_unchecked(1),
    );
    assert_eq!(result, None);

    //try to get price for unknown quote asset
    result = client.x_last_price(
        &config_data.assets.get_unchecked(1),
        &Asset::Stellar(Address::generate(&env)),
    );
    assert_eq!(result, None);

    //try to get price for both unknown assets
    result = client.x_last_price(
        &Asset::Stellar(Address::generate(&env)),
        &Asset::Other(Symbol::new(&env, "NonRegisteredAsset")),
    );
    assert_eq!(result, None);
}

#[test]
fn get_asset_price_for_invalid_timestamp_test() {
    let (env, client, config_data) = init_contract_with_admin();

    let mut result = client.price(
        &config_data.assets.get_unchecked(1),
        &convert_to_seconds(u64::MAX),
    );
    assert_eq!(result, None);

    //try to get price for unknown asset
    result = client.lastprice(&Asset::Stellar(Address::generate(&env)));
    assert_eq!(result, None);
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
                fn_name: "set_period",
                args: Vec::from_array(&env, [period.clone().try_into_val(&env).unwrap()]),
                sub_invokes: &[],
            },
        }])
        .set_period(&period);
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
        .set_period(&period);
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
        let result = panic::catch_unwind(AssertUnwindSafe(|| a.fixed_div_floor(*b, 14)));
        if expected == &-1 {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), *expected);
        }
    }
}
