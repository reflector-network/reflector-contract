#![cfg(test)]
extern crate std;
extern crate alloc;

use super::*;
use alloc::string::ToString;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol};

use shared::{constants::Constants, extensions::{u64_extensions::U64Extensions, env_extensions::EnvExtensions}, types::asset::Asset};

fn init_contract_with_admin<'a>() -> (Env, PriceOracleContractClient<'a>, ConfigData) {
    let env = Env::default();

    let admin = Address::random(&env);

    let contract_id = &Address::from_contract_id(&BytesN::from_array(&env, &[0; 32]));
    env.register_contract(contract_id, PriceOracleContract);
    let client: PriceOracleContractClient<'a> = PriceOracleContractClient::new(&env, contract_id);

    let resolution: u32 = 300_000;

    let init_data = ConfigData {
        admin: admin.clone(),
        period: (100 * resolution).into(),
        assets: generate_assets(&env, 10, 0),
        version: 1,
        base_fee: 0,
    };

    env.mock_all_auths();

    //set admin
    client.config(&admin, &init_data);

    (env, client, init_data)
}

fn normalize_price(price: i128) -> i128 {
    price * 10i128.pow(Constants::DECIMALS)
}

fn generate_assets(e: &Env, count: usize, start_index: u32) -> Vec<Asset> {
    let mut assets = Vec::new(&e);
    for i in 0..count {
        if i % 2 == 0 {
            assets.push_back(Asset::S(Address::random(&e)));
        } else {
            assets.push_back(Asset::G(Symbol::new(e, &("ASSET_".to_string() + &(start_index + i as u32).to_string()))));
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
fn init_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let address = client.admin();
    assert_eq!(address, init_data.admin.clone());

    let base = client.base();
    assert_eq!(base, env.get_base_asset());

    let resolution = client.resolution();
    assert_eq!(resolution, Constants::RESOLUTION / 1000);

    let period = client.period().unwrap();
    assert_eq!(period, init_data.period);

    let decimals = client.decimals();
    assert_eq!(decimals, Constants::DECIMALS);

    let assets = client.assets();
    assert_eq!(assets, init_data.assets);

    let config_version = client.config_version();
    assert_eq!(config_version, init_data.version);
}

#[test]
fn last_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;
    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &&assets, normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    //check last prices
    let result = client.lastprice(&assets.get_unchecked(1));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(200),
            timestamp: 900_000 as u64
        })
    );
}

#[test]
fn last_timestamp_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;
    let assets = init_data.assets;

    let mut result = client.last_timestamp();

    assert_eq!(result, 0);

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);
    
    result = client.last_timestamp();

    assert_eq!(result, 600_000);
}

#[test]
fn add_assets_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;

    let assets = generate_assets(&env, 10, init_data.assets.len() - 1);

    let current_version = client.config_version();

    env.mock_all_auths();

    client.add_assets(&admin, &assets, &(current_version + 1));

    let result = client.assets();

    let mut expected_assets = init_data.assets.clone();
    for asset in assets.iter() {
        expected_assets.push_back(asset.clone());
    }

    assert_eq!(result, expected_assets);
}

#[test]
fn set_period_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;

    let period = 100_000;

    let current_version = client.config_version();

    env.mock_all_auths();

    client.set_period(&admin, &period, &(current_version + 1));

    let result = client.period().unwrap();

    assert_eq!(result, period);
}

#[test]
fn config_version_test() {
    let (__env, client, __init_data) = init_contract_with_admin();

    let result = client.config_version();

    assert_eq!(result, 1);
}

#[test]
fn get_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;
    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(200));

    client.set_price(&admin, &updates, &timestamp);

    //check last prices
    let mut result = client.lastprice(&assets.get_unchecked(1));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(200),
            timestamp: 900_000 as u64
        })
    );

    //check price at 899_000
    result = client.price(&assets.get_unchecked(1), &899_000);
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(100),
            timestamp: 600_000 as u64
        })
    );
}

#[test]
fn get_x_last_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;
    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    client.set_price(&admin, &updates, &timestamp);

    //check last x price
    let result = client.x_last_price(
        &assets.get_unchecked(1),
        &assets.get_unchecked(2),
    );
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(1),
            timestamp: 600_000 as u64
        })
    );
}

#[test]
fn get_x_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;
    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    //check last prices
    let mut result = client.x_last_price(
        &assets.get_unchecked(1),
        &assets.get_unchecked(2),
    );
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(1),
            timestamp: 900_000 as u64
        })
    );

    //check price at 899_000
    result = client.x_price(
        &assets.get_unchecked(1),
        &assets.get_unchecked(2),
        &899_000,
    );
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(1),
            timestamp: 600_000 as u64
        })
    );
}

#[test]
fn twap_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;
    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let result = client.twap(&assets.get_unchecked(1), &2);

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(150));
}

#[test]
fn x_twap_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;
    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let result = client.x_twap(
        &assets.get_unchecked(1),
        &assets.get_unchecked(2),
        &2,
    );

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(1));
}

#[test]
fn get_non_registered_asset_price_test() {
    let (env, client, config_data) = init_contract_with_admin();

    //try to get price for unknown Stellar asset
    let mut result = client.lastprice(&Asset::S(Address::random(&env)));
    assert_eq!(result, None);

    //try to get price for unknown Generic asset
    result = client.lastprice(&Asset::G(Symbol::new(&env, "NonRegisteredAsset")));
    assert_eq!(result, None);

    //try to get price for unknown base asset
    result = client.x_last_price(&Asset::S(Address::random(&env)), &config_data.assets.get_unchecked(1));
    assert_eq!(result, None);

    //try to get price for unknown quote asset
    result = client.x_last_price(&config_data.assets.get_unchecked(1), &Asset::S(Address::random(&env)));
    assert_eq!(result, None);

    //try to get price for both unknown assets
    result = client.x_last_price(&Asset::S(Address::random(&env)), &Asset::G(Symbol::new(&env, "NonRegisteredAsset")));
    assert_eq!(result, None);
}

#[test]
fn get_asset_price_for_invalid_timestamp_test() {
    let (env, client, config_data) = init_contract_with_admin();

    
    let mut result = client.price(&config_data.assets.get_unchecked(1), &u64::MAX);
    assert_eq!(result, None);

    //try to get price for unknown asset
    result = client.lastprice(&Asset::S(Address::random(&env)));
    assert_eq!(result, None);
}

#[test]
#[should_panic]
fn unauthorized_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let updates = get_updates(&env, &assets, 100);

    let account = Address::random(&env);
    let timestamp = (112331 as u64).get_normalized_timestamp(Constants::RESOLUTION as u64);

    //mock auth to check only contract's admin validation
    env.mock_all_auths();

    //set prices for assets
    client.set_price(&account, &updates, &timestamp);
}