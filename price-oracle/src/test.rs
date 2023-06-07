#![cfg(test)]
extern crate std;
extern crate alloc;

use super::*;
use alloc::rc::Rc;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, xdr, Symbol};

use shared::{constants::Constants, extensions::u64_extensions::U64Extensions, types::asset::Asset};

pub fn register_account(e: &Env, account: &[u8; 32]) {
    let account_id = xdr::AccountId(xdr::PublicKey::PublicKeyTypeEd25519(xdr::Uint256(
        account.clone(),
    )));
    e.host()
            .with_mut_storage(|storage| {
                let k = Rc::new(xdr::LedgerKey::Account(xdr::LedgerKeyAccount {
                    account_id: account_id.clone(),
                }));

                let budget = e.host().budget_cloned();

                if !storage.has(
                    &k,
                    &budget,
                )? {
                    let v = Rc::new(xdr::LedgerEntry {
                        data: xdr::LedgerEntryData::Account(xdr::AccountEntry {
                            account_id: account_id.clone(),
                            balance: 0,
                            flags: 0,
                            home_domain: Default::default(),
                            inflation_dest: None,
                            num_sub_entries: 0,
                            seq_num: xdr::SequenceNumber(0),
                            thresholds: xdr::Thresholds([1; 4]),
                            signers: xdr::VecM::default(),
                            ext: xdr::AccountEntryExt::V0,
                        }),
                        last_modified_ledger_seq: 0,
                        ext: xdr::LedgerEntryExt::V0,
                    });
                    storage.put(
                        &k,
                        &v,
                        &budget,
                    )?
                }
                Ok(())
            })
            .unwrap();
}

fn init_contract_with_admin() -> (Env, PriceOracleContractClient, ConfigData) {
    let env = Env::default();

    register_account(&env, &Constants::ADMIN);

    let contract_id = BytesN::from_array(&env, &[0; 32]);
    env.register_contract(&contract_id, PriceOracleContract);
    let client = PriceOracleContractClient::new(&env, &contract_id);

    let resolution: u32 = 300_000;

    let init_data = ConfigData {
        admin: Address::random(&env),
        period: (100 * resolution).into(),
        assets: generate_assets(&env, 10),
        base_fee: 0,
    };

    let default_admin = Address::from_account_id(&env, &BytesN::from_array(&env, &Constants::ADMIN));

    //set admin
    client.config(&default_admin, &init_data);

    (env, client, init_data)
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
            assets.push_back(Asset::Generic(Symbol::new(e, &stringify!("ASSET_{}", i))));
        }
    }
    assets
}

fn get_updates(env: &Env, assets: Vec<Asset>, price: i128) -> Vec<i128> {
    let mut updates = Vec::new(&env);
    for _ in assets.iter() {
        updates.push_back(price);
    }
    updates
}

fn get_contract_address(e: &Env, bytes: [u8; 32]) -> Address {
    Address::from_contract_id(e, &BytesN::from_array(e, &bytes))
}

#[test]
fn init_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let address = client.admin();
    assert_eq!(address, init_data.admin.clone());

    let base = client.base();
    assert_eq!(base, get_contract_address(&env, Constants::BASE));

    let resolution = client.resolution();
    assert_eq!(resolution, Constants::RESOLUTION / 1000);

    let period = client.period().unwrap();
    assert_eq!(period, init_data.period);

    let decimals = client.decimals();
    assert_eq!(decimals, Constants::DECIMALS);

    let assets = client.assets();
    assert_eq!(assets, init_data.assets);
}

#[test]
fn last_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;
    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets.clone(), normalize_price(100));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets.clone(), normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    //check last prices
    let result = client.lastprice(&assets.get_unchecked(1).unwrap());
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
fn get_price_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;
    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets.clone(), normalize_price(100));

    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets.clone(), normalize_price(200));

    client.set_price(&admin, &updates, &timestamp);

    //check last prices
    let mut result = client.lastprice(&assets.get_unchecked(1).unwrap());
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: normalize_price(200),
            timestamp: 900_000 as u64
        })
    );

    //check price at 899_000
    result = client.price(&assets.get_unchecked(1).unwrap(), &899_000);
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
    let updates = get_updates(&env, assets.clone(), normalize_price(100));

    client.set_price(&admin, &updates, &timestamp);

    //check last x price
    let result = client.x_last_price(
        &assets.get_unchecked(1).unwrap(),
        &assets.get_unchecked(2).unwrap(),
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
    let updates = get_updates(&env, assets.clone(), normalize_price(100));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets.clone(), normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    //check last prices
    let mut result = client.x_last_price(
        &assets.get_unchecked(1).unwrap(),
        &assets.get_unchecked(2).unwrap(),
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
        &assets.get_unchecked(1).unwrap(),
        &assets.get_unchecked(2).unwrap(),
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
    let updates = get_updates(&env, assets.clone(), normalize_price(100));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets.clone(), normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let result = client.twap(&assets.get_unchecked(1).unwrap(), &2);

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(150));
}

#[test]
fn x_twap_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let admin = &init_data.admin;
    let assets = init_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets.clone(), normalize_price(100));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets.clone(), normalize_price(200));

    //set prices for assets
    client.set_price(&admin, &updates, &timestamp);

    let result = client.x_twap(
        &assets.get_unchecked(1).unwrap(),
        &assets.get_unchecked(2).unwrap(),
        &2,
    );

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(1));
}

#[test]
fn get_non_registered_asset_price_test() {
    let (env, client, config_data) = init_contract_with_admin();

    //try to get price for unknown Stellar asset
    let mut result = client.lastprice(&Asset::Stellar(Address::random(&env)));
    assert_eq!(result, None);

    //try to get price for unknown Generic asset
    result = client.lastprice(&Asset::Generic(Symbol::new(&env, stringify!("NonRegisteredAsset"))));
    assert_eq!(result, None);

    //try to get price for unknown base asset
    result = client.x_last_price(&Asset::Stellar(Address::random(&env)), &config_data.assets.get_unchecked(1).unwrap());
    assert_eq!(result, None);

    //try to get price for unknown quote asset
    result = client.x_last_price(&config_data.assets.get_unchecked(1).unwrap(), &Asset::Stellar(Address::random(&env)));
    assert_eq!(result, None);

    //try to get price for both unknown assets
    result = client.x_last_price(&Asset::Stellar(Address::random(&env)), &Asset::Generic(Symbol::new(&env, stringify!("NonRegisteredAsset"))));
    assert_eq!(result, None);
}

#[test]
fn get_asset_price_for_invalid_timestamp_test() {
    let (env, client, config_data) = init_contract_with_admin();

    
    let mut result = client.price(&config_data.assets.get_unchecked(1).unwrap(), &u64::MAX);
    assert_eq!(result, None);

    //try to get price for unknown asset
    result = client.lastprice(&Asset::Stellar(Address::random(&env)));
    assert_eq!(result, None);
}

#[test]
#[should_panic]
fn unauthorized_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let assets = init_data.assets;

    let updates = get_updates(&env, assets, 100);

    let account = Address::random(&env);
    let timestamp = (112331 as u64).get_normalized_timestamp(Constants::RESOLUTION as u64);
    //set prices for assets
    client.set_price(&account, &updates, &timestamp);
}