#![cfg(test)]
extern crate std;
extern crate alloc;

use alloc::rc::Rc;
use soroban_sdk::{testutils::Address as _, xdr, Address, BytesN, Env, Symbol, TryIntoVal};

use shared::{constants::Constants, extensions::u64_extensions::U64Extensions};

use super::*;

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

pub fn register_stellar_asset_contract(e: &Env, admin: Address) -> Address {
    let issuer_id = xdr::AccountId(xdr::PublicKey::PublicKeyTypeEd25519(xdr::Uint256(
        Constants::ADMIN.clone(),
    )));

    let asset = xdr::Asset::CreditAlphanum4(xdr::AlphaNum4 {
        asset_code: xdr::AssetCode4([1, 1, 1, 1]),
        issuer: issuer_id.clone(),
    });
    let create = xdr::HostFunction::CreateContract(xdr::CreateContractArgs {
        contract_id: xdr::ContractId::Asset(asset.clone()),
        source: xdr::ScContractExecutable::Token,
    });

    let token_id: BytesN<32> = e
        .host()
        .invoke_function(create)
        .unwrap()
        .try_into_val(e)
        .unwrap();

    let issuer_address = Address::from_account_id(e, &BytesN::from_array(e, &Constants::ADMIN));

    let _: () = e.invoke_contract(
        &token_id,
        &Symbol::short("set_admin"),
        (&issuer_address, &admin).try_into_val(e).unwrap(),
    );

    Address::from_contract_id(e, &token_id)
}

fn mint(e: &Env, admin: &Address, token: &Address, user: &Address, amount: i128) {
    let token = token::Client::new(&e, &token.contract_id().unwrap());
    token.mint(&admin, &user, &amount);
}

fn init_contract_with_admin() -> (Env, PriceOracleContractClient, ConfigData, Address) {
    let env = Env::default();

    register_account(&env, &Constants::ADMIN);

    let contract_id = BytesN::from_array(&env, &[0; 32]);
    env.register_contract(&contract_id, PriceOracleContract);
    let client = PriceOracleContractClient::new(&env, &contract_id);

    let resolution: u32 = 300_000;

    let config_data = ConfigData {
        admin: Address::random(&env),
        period: (100 * resolution).into(),
        assets: generate_assets(&env, 10),
        base_fee: 100,
    };

    let token = register_stellar_asset_contract(&env, config_data.admin.clone());

    let default_admin = Address::from_account_id(&env, &BytesN::from_array(&env, &Constants::ADMIN));

    //set admin
    client.config(&default_admin, &config_data);


    (env, client, config_data, token)
}

fn normalize_price(price: i128) -> i128 {
    price * 10i128.pow(Constants::DECIMALS)
}

fn generate_assets(e: &Env, count: usize) -> Vec<Address> {
    let mut assets = Vec::new(&e);
    for _ in 0..count {
        assets.push_back(Address::random(&e));
    }
    assets
}

fn get_updates(env: &Env, assets: &Vec<Address>, price: i128) -> Vec<i128> {
    let mut updates = Vec::new(&env);
    for _ in assets.iter() {
        updates.push_back(price);
    }
    updates
}

fn get_contract_address(e: &Env, bytes: [u8; 32]) -> Address {
    Address::from_contract_id(e, &BytesN::from_array(e, &bytes))
}

fn deposit(
    e: &Env,
    client: &PriceOracleContractClient,
    config_data: &ConfigData,
    user: &Address,
    contract: &BytesN<32>,
    token: &Address,
    amount: &i128,
) {
    mint(e, &config_data.admin, token, user, 10000);
    client.deposit(user, contract, token, amount);
}

fn deposit_random_contract(
    e: &Env,
    client: &PriceOracleContractClient,
    config_data: &ConfigData,
    token: &Address,
    amount: &i128,
) -> BytesN<32> {
    let user = Address::random(&e);
    let contract = Address::random(&e).contract_id().unwrap();
    deposit(e, client, config_data, &user, &contract, token, amount);
    contract
}

#[test]
fn init_test() {
    let (env, client, config_data, _) = init_contract_with_admin();

    let address = client.admin();
    assert_eq!(address, config_data.admin.clone());

    let base = client.base();
    assert_eq!(base, get_contract_address(&env, Constants::BASE));

    let resolution = client.resolution();
    assert_eq!(resolution, Constants::RESOLUTION / 1000);

    let period = client.period().unwrap();
    assert_eq!(period, config_data.period);

    let decimals = client.decimals();
    assert_eq!(decimals, Constants::DECIMALS);

    let assets = client.assets().unwrap();
    assert_eq!(assets, config_data.assets);

    let base_fee = client.base_fee().unwrap();
    assert_eq!(base_fee, config_data.base_fee);

    let fee_asset = client.fee_asset();
    assert_eq!(fee_asset, get_contract_address(&env, Constants::FEE_ASSET));
}

#[test]
fn deposit_and_charge_test() {
    let (env, client, config_data, token) = init_contract_with_admin();

    let contract = deposit_random_contract(&env, &client, &config_data, &token, &100);

    let mut balance = client.balance(&contract);
    assert_ne!(balance, None);
    assert_eq!(balance, Some(100));

    let updates = get_updates(&env, &config_data.assets, normalize_price(100));

    let timestamp = 600_000;
    client.set_price(&config_data.admin, &updates, &timestamp);

    let price = env.as_contract(&contract, || {
        client.lastprice(&config_data.assets.get_unchecked(1).unwrap())
    });
    assert_ne!(price, None);

    balance = client.balance(&contract);
    assert_ne!(balance, None);
    assert_eq!(balance, Some(0));
}

#[test]
fn last_price_test() {
    let (env, client, config_data, token) = init_contract_with_admin();

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
    let contract = deposit_random_contract(&env, &client, &config_data, &token, &100);

    let result = env.as_contract(&contract, || {
        client.lastprice(&assets.get_unchecked(1).unwrap())
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
fn get_price_test() {
    let (env, client, config_data, token) = init_contract_with_admin();

    let admin = &config_data.admin;
    let assets = &config_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets, normalize_price(100));

    client.set_price(&admin, &updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, assets, normalize_price(200));

    client.set_price(&admin, &updates, &timestamp);

    let contract = deposit_random_contract(&env, &client, &config_data, &token, &200);

    //check last prices
    let mut result = env.as_contract(&contract, || {
        client.lastprice(&assets.get_unchecked(1).unwrap())
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
        client.price(&assets.get_unchecked(1).unwrap(), &899_000)
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
    let (env, client, config_data, token) = init_contract_with_admin();

    let admin = &config_data.admin;
    let assets = &config_data.assets;

    let timestamp = 600_000;
    let updates = get_updates(&env, assets, normalize_price(100));

    client.set_price(&admin, &updates, &timestamp);

    let contract = deposit_random_contract(&env, &client, &config_data, &token, &200);

    //check last x price
    let result = env.as_contract(&contract, || {
        client.x_last_price(
            &assets.get_unchecked(1).unwrap(),
            &assets.get_unchecked(2).unwrap(),
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
    let (env, client, config_data, token) = init_contract_with_admin();

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

    let contract = deposit_random_contract(&env, &client, &config_data, &token, &400);

    //check last prices
    let mut result = env.as_contract(&contract, || {
        client.x_last_price(
            &assets.get_unchecked(1).unwrap(),
            &assets.get_unchecked(2).unwrap(),
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
            &assets.get_unchecked(1).unwrap(),
            &assets.get_unchecked(2).unwrap(),
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
    let (env, client, config_data, token) = init_contract_with_admin();

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

    let contract = deposit_random_contract(&env, &client, &config_data, &token, &200);

    let result = env.as_contract(&contract, || {
        client.twap(&assets.get_unchecked(1).unwrap(), &2)
    });

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(150));
}

#[test]
fn x_twap_test() {
    let (env, client, config_data, token) = init_contract_with_admin();

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

    let contract = deposit_random_contract(&env, &client, &config_data, &token, &400);

    let result = env.as_contract(&contract, || {
        client.x_twap(
            &assets.get_unchecked(1).unwrap(),
            &assets.get_unchecked(2).unwrap(),
            &2,
        )
    });

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), normalize_price(1));
}

#[test]
fn get_non_registered_asset_price_test() {
    let (env, client, config_data, token) = init_contract_with_admin();

    let contract = deposit_random_contract(&env, &client, &config_data, &token, &1000);

    //try to get price for unknown asset
    let mut result = env.as_contract(&contract, || {client.lastprice(&Address::random(&env)) });
    assert_eq!(result, None);

    //try to get price for unknown base asset
    result = env.as_contract(&contract, || {client.x_last_price(&Address::random(&env), &config_data.assets.get_unchecked(1).unwrap()) });
    assert_eq!(result, None);

    //try to get price for unknown quote asset
    result = env.as_contract(&contract, || {client.x_last_price(&config_data.assets.get_unchecked(1).unwrap(), &Address::random(&env)) });
    assert_eq!(result, None);

    //try to get price for both unknown assets
    result = env.as_contract(&contract, || {client.x_last_price(&Address::random(&env), &Address::random(&env)) });
    assert_eq!(result, None);
}

#[test]
fn get_asset_price_for_invalid_timestamp_test() {
    let (env, client, config_data, token) = init_contract_with_admin();

    let contract = deposit_random_contract(&env, &client, &config_data, &token, &400);

    
    let mut result = env.as_contract(&contract, || {client.price(&config_data.assets.get_unchecked(1).unwrap(), &u64::MAX) });
    assert_eq!(result, None);

    //try to get price for unknown asset
    result = env.as_contract(&contract, || {client.lastprice(&Address::random(&env)) });
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
