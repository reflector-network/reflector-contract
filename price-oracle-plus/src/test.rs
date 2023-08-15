#![cfg(test)]
extern crate alloc;
extern crate std;

use alloc::string::ToString;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol};

use shared::{constants::Constants, extensions::u64_extensions::U64Extensions};

use super::*;

fn init_contract_with_admin<'a>() -> (Env, PriceOracleContractClient<'a>, ConfigData, Address) {
    let env = Env::default();

    let contract_id = Address::from_contract_id(&BytesN::from_array(&env, &[0; 32]));
    env.register_contract(&contract_id, PriceOracleContract);
    let client = PriceOracleContractClient::new(&env, &contract_id);

    let resolution: u32 = 300_000;

    let admin = Address::random(&env);

    let config_data = ConfigData {
        admin: admin.clone(),
        period: (100 * resolution).into(),
        assets: generate_assets(&env, 10),
        version: 1,
        base_fee: 100,
    };

    let token = env.register_stellar_asset_contract(config_data.admin.clone());

    env.mock_all_auths();

    //set admin
    client.config(&admin, &config_data);

    (env, client, config_data, token)
}

fn normalize_price(price: i128) -> i128 {
    price * 10i128.pow(Constants::DECIMALS)
}

fn generate_assets(e: &Env, count: usize) -> Vec<Asset> {
    let mut assets = Vec::new(&e);
    for i in 0..count {
        if i % 2 == 0 {
            assets.push_back(Asset::Stellar(Address::random(&e)));
        } else {
            assets.push_back(Asset::Generic(Symbol::new(
                e,
                &("ASSET_".to_string() + &i.to_string()),
            )));
        }
    }
    assets
}

fn get_updates(env: &Env, assets: &Vec<Asset>, price: i128) -> Vec<PriceUpdateItem> {
    let mut updates = Vec::new(&env);
    for asset in assets.iter() {
        updates.push_back(PriceUpdateItem {
            asset: asset.clone(),
            price
        });
    }
    updates
}

fn get_contract_address(e: &Env, bytes: [u8; 32]) -> Address {
    Address::from_contract_id(&BytesN::from_array(e, &bytes))
}

fn deposit_random_contract(e: &Env, as_contract: &Address, amount: i128) -> Address {
    let contract = Address::random(&e);
    e.as_contract(as_contract, || {
        e.try_inc_balance(contract.clone(), amount);
    });
    contract
}

#[test]
fn init_test() {
    let (env, client, config_data, _) = init_contract_with_admin();

    let address = client.admin();
    assert_eq!(address, config_data.admin.clone());

    let base = client.base();
    assert_eq!(base, env.get_base_asset());

    let resolution = client.resolution();
    assert_eq!(resolution, Constants::RESOLUTION / 1000);

    let period = client.period().unwrap();
    assert_eq!(period, config_data.period);

    let decimals = client.decimals();
    assert_eq!(decimals, Constants::DECIMALS);

    let assets = client.assets();
    assert_eq!(assets, config_data.assets);

    let base_fee = client.base_fee().unwrap();
    assert_eq!(base_fee, config_data.base_fee);

    let fee_asset = client.fee_asset();
    assert_eq!(fee_asset, get_contract_address(&env, Constants::FEE_ASSET));

    let version = client.config_version();
    assert_eq!(version, config_data.version);
}

#[test]
fn deposit_and_charge_test() {
    let (env, client, config_data, _) = init_contract_with_admin();

    let contract = deposit_random_contract(&env, &client.address, 100);

    let mut balance = client.balance(&contract);
    assert_ne!(balance, None);
    assert_eq!(balance, Some(100));

    let updates = get_updates(&env, &config_data.assets, normalize_price(100));

    let timestamp = 600_000;

    client.set_price(&config_data.admin, &updates, &timestamp);

    let price = env.as_contract(&contract, || {
        client.lastprice(&config_data.assets.get_unchecked(1))
    });
    assert_ne!(price, None);

    balance = client.balance(&contract);
    assert_ne!(balance, None);
    assert_eq!(balance, Some(0));
}

#[test]
fn last_price_test() {
    let (env, client, config_data, _) = init_contract_with_admin();

    let admin = &config_data.admin;
    let assets = &config_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets, normalize_price(100));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets, normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    //check last prices
    let contract = deposit_random_contract(&env, &client.address, 100);

    let result = env.as_contract(&contract, || {
        client.lastprice(&assets.get_unchecked(1))
    });
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
    let (env, client, init_data, _) = init_contract_with_admin();

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
fn get_price_test() {
    let (env, client, config_data, _) = init_contract_with_admin();

    let admin = &config_data.admin;
    let assets = &config_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets, normalize_price(100));

    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets, normalize_price(200));

    client.set_price(&admin, &updates, &timestamp);

    let contract = deposit_random_contract(&env, &client.address, 200);

    //check last prices
    let mut result = env.as_contract(&contract, || {
        client.lastprice(&assets.get_unchecked(1))
    });
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(200),
            timestamp: 900_000 as u64
        })
    );

    //check price at 899_000
    result = env.as_contract(&contract, || {
        client.price(&assets.get_unchecked(1), &899_000)
    });
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
    let (env, client, config_data, _) = init_contract_with_admin();

    let admin = &config_data.admin;
    let assets = &config_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets, normalize_price(100));

    client.set_price(&admin, &updates, &timestamp);

    let contract = deposit_random_contract(&env, &client.address, 200);

    //check last x price
    let result = env.as_contract(&contract, || {
        client.x_last_price(
            &assets.get_unchecked(1),
            &assets.get_unchecked(2),
        )
    });
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
    let (env, client, config_data, _) = init_contract_with_admin();

    let admin = &config_data.admin;
    let assets = &config_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets, normalize_price(100));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets, normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let contract = deposit_random_contract(&env, &client.address, 400);

    //check last prices
    let mut result = env.as_contract(&contract, || {
        client.x_last_price(
            &assets.get_unchecked(1),
            &assets.get_unchecked(2),
        )
    });
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(1),
            timestamp: 900_000 as u64
        })
    );

    //check price at 899_000
    result = env.as_contract(&contract, || {
        client.x_price(
            &assets.get_unchecked(1),
            &assets.get_unchecked(2),
            &899_000,
        )
    });
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
    let (env, client, config_data, _) = init_contract_with_admin();

    let admin = &config_data.admin;
    let assets = &config_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets, normalize_price(100));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets, normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let contract = deposit_random_contract(&env, &client.address, 200);

    let result = env.as_contract(&contract, || {
        client.twap(&assets.get_unchecked(1), &2)
    });

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(150));
}

#[test]
fn x_twap_test() {
    let (env, client, config_data, _) = init_contract_with_admin();

    let admin = &config_data.admin;
    let assets = &config_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets, normalize_price(100));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets, normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let contract = deposit_random_contract(&env, &client.address, 400);

    let result = env.as_contract(&contract, || {
        client.x_twap(
            &assets.get_unchecked(1),
            &assets.get_unchecked(2),
            &2,
        )
    });

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(1));
}

#[test]
fn get_non_registered_asset_price_test() {
    let (env, client, config_data, _) = init_contract_with_admin();

    let contract = deposit_random_contract(&env, &client.address, 1000);

    //try to get price for unknown asset
    let mut result = env.as_contract(&contract, || {
        client.lastprice(&Asset::Generic(Symbol::new(
            &env,
            "NonRegisteredAsset",
        )))
    });
    assert_eq!(result, None);

    //try to get price for unknown base asset
    result = env.as_contract(&contract, || {
        client.x_last_price(
            &Asset::Stellar(Address::random(&env)),
            &config_data.assets.get_unchecked(1),
        )
    });
    assert_eq!(result, None);

    //try to get price for unknown quote asset
    result = env.as_contract(&contract, || {
        client.x_last_price(
            &config_data.assets.get_unchecked(1),
            &Asset::Stellar(Address::random(&env)),
        )
    });
    assert_eq!(result, None);

    //try to get price for both unknown assets
    result = env.as_contract(&contract, || {
        client.x_last_price(
            &Asset::Stellar(Address::random(&env)),
            &Asset::Generic(Symbol::new(&env, "NonRegisteredAsset")),
        )
    });
    assert_eq!(result, None);
}

#[test]
fn get_asset_price_for_invalid_timestamp_test() {
    let (env, client, config_data, _) = init_contract_with_admin();

    let contract = deposit_random_contract(&env, &client.address, 400);

    let mut result = env.as_contract(&contract, || {
        client.price(&config_data.assets.get_unchecked(1), &u64::MAX)
    });
    assert_eq!(result, None);

    //try to get price for unknown asset
    result = env.as_contract(&contract, || {
        client.lastprice(&Asset::Stellar(Address::random(&env)))
    });
    assert_eq!(result, None);
}

#[test]
#[should_panic]
fn unauthorized_test() {
    let (env, client, config_data, _) = init_contract_with_admin();

    let assets = &config_data.assets;

    let updates = get_updates(&env, assets, 100);

    let account = Address::random(&env);
    let timestamp = (112331 as u64).get_normalized_timestamp(Constants::RESOLUTION as u64);
    //set prices for assets
    client.set_price(&account, &updates, &timestamp);
}
