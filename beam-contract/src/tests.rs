#![cfg(test)]
extern crate std;

use crate::cost::InvocationComplexity;
use crate::{BeamOracleContract, BeamOracleContractClient};
use oracle::assets;
use oracle::types::{Asset, ConfigData, FeeConfig};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::{Address, Env, String, Vec};
use test_case::test_case;

pub fn init_contract_with_admin<'a>() -> (Env, BeamOracleContractClient<'a>, ConfigData) {
    let env = Env::default();

    //set timestamp to 900 seconds
    let ledger_info = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp: 900,
        ..ledger_info
    });

    let contract_id = &Address::from_string(&String::from_str(
        &env,
        "CDXHQTB7FGRMWTLJJLNI3XPKVC6SZDB5SFGZUYDPEGQQNC4G6CKE4QRC",
    ));

    env.register_at(contract_id, BeamOracleContract, ());
    let client = BeamOracleContractClient::new(&env, contract_id);

    env.cost_estimate().budget().reset_unlimited();

    env.mock_all_auths();
    let init_data = prepare_contract_config(&env);
    client.config(&init_data);

    (env, client, init_data)
}

fn prepare_contract_config(env: &Env) -> ConfigData {
    let admin = Address::generate(env);
    let mut assets = Vec::new(env);
    for _ in 0..10 {
        assets.push_back(Asset::Stellar(Address::generate(env)));
    }
    let resolution = 300_000u32;
    ConfigData {
        admin: admin.clone(),
        history_retention_period: (100 * resolution).into(),
        assets,
        base_asset: Asset::Stellar(Address::generate(&env)),
        decimals: 14,
        resolution,
        cache_size: 0,
        fee_config: FeeConfig::None,
    }
}

#[test]
fn set_invocation_config_test() {
    let (env, client, _) = init_contract_with_admin();

    let costs = Vec::from_array(&env, [10, 20, 30, 40, 50]);
    client.set_invocation_costs_config(&costs);

    let result = client.invocation_costs();
    assert_eq!(result, costs);
}

#[test]
fn invocation_charge_test() {
    let (env, client, init_data) = init_contract_with_admin();

    let fee_asset = env
        .register_stellar_asset_contract_v2(init_data.admin.clone())
        .address();
    let fee_config = FeeConfig::Some((fee_asset.clone(), 1_000_000));
    client.set_fee_config(&fee_config);

    let caller = Address::generate(&env);
    //mint fee token to caller
    let fee_token = StellarAssetClient::new(&env, &fee_asset);
    fee_token.mint(&caller, &100_000_000);
    //get price for the first asset
    client.lastprice(&caller, &init_data.assets.first_unchecked());
    //get cross price
    client.x_twap(
        &caller,
        &init_data.base_asset,
        &init_data.assets.first_unchecked(),
        &5,
    );
    //check that fee token was deducted
    let fee_token_balance = fee_token.balance(&caller);
    assert_eq!(fee_token_balance, 90_000_000);
}

#[test_case(InvocationComplexity::Price, 1, 10_000_000 ; "price")]
#[test_case(InvocationComplexity::Twap, 1, 15_000_000 ; "twap")]
#[test_case(InvocationComplexity::CrossPrice, 1, 20_000_000 ; "cross price")]
#[test_case(InvocationComplexity::CrossTwap, 1, 30_000_000 ; "cross twap")]
#[test_case(InvocationComplexity::Price, 2, 12_000_000 ; "multi round price")]
#[test_case(InvocationComplexity::Twap, 5, 27_000_000 ; "multi round twap")]
#[test_case(InvocationComplexity::CrossPrice, 2, 24_000_000 ; "multi round cross price")]
#[test_case(InvocationComplexity::CrossTwap, 7, 66_000_000 ; "multi round cross twap")]
fn invocation_charge_estimate_test(
    invocation: InvocationComplexity,
    periods: u32,
    expected_fee: i128,
) {
    let (env, client, init_data) = init_contract_with_admin();

    let fee_asset = env
        .register_stellar_asset_contract_v2(init_data.admin.clone())
        .address();
    let fee_config = FeeConfig::Some((fee_asset.clone(), 1_000_000));
    client.set_fee_config(&fee_config);
    let costs = Vec::from_array(
        &env,
        [2_000_000, 10_000_000, 15_000_000, 20_000_000, 30_000_000],
    );
    client.set_invocation_costs_config(&costs);

    let fee = client.estimate_cost(&invocation, &periods);
    assert_eq!(fee, expected_fee);
}

#[test]
fn check_extending_asset_ttl() {
    //initialize contract
    let (env, client, init_data) = init_contract_with_admin();

    //set fee config
    let asset_contract = env.register_stellar_asset_contract_v2(init_data.admin.clone());
    let fee_asset = asset_contract.address();
    let fee_config = FeeConfig::Some((fee_asset.clone(), 1_000_000));
    client.set_fee_config(&fee_config);

    //add new asset to the oracle
    let new_asset = Asset::Stellar(Address::generate(&env));
    let mut new_assets = Vec::new(&env);
    new_assets.push_back(new_asset.clone());
    client.add_assets(&new_assets);

    //check that expiration is set for the new asset
    let exp = client.expires(&new_asset);
    assert_ne!(exp, None, "Expected expiration to be set for the new asset");

    //extend TTL for the new asset
    let sponsor = Address::generate(&env);
    let token_client = StellarAssetClient::new(&env, &fee_asset);
    token_client.mint(&sponsor, &10_000_000);

    //check the extending
    client.extend_asset_ttl(&sponsor, &new_asset, &1_000_000);
    assert_eq!(client.expires(&new_asset), Some(87_300));

    //check that expiration records length matches assets length
    env.as_contract(&client.address, || {
        let expiration: Vec<u64> = env.storage().instance().get(&"expiration").unwrap();
        assert_eq!(assets::load_all_assets(&env).len(), expiration.len());
    });
}
