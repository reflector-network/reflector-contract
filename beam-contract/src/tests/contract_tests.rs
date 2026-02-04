#![cfg(test)]
extern crate std;

use crate::{BeamOracleContract, BeamOracleContractClient};
use oracle::testutils::register_token;
use oracle::types::{Asset, FeeConfig};
use oracle::{assets, init_contract_with_admin};
use soroban_sdk::{testutils::Address as _, Address, Vec};
use test_case::test_case;

#[test]
fn set_invocation_config_test() {
    let (env, client, _) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);

    let initial_costs = client.invocation_costs();
    assert_eq!(initial_costs.len(), 2);
    assert_eq!(
        initial_costs,
        Vec::from_array(&env, [2_000_000, 10_000_000])
    );

    let costs = Vec::from_array(&env, [10, 20]);
    client.set_invocation_costs_config(&costs);

    let result = client.invocation_costs();
    assert_eq!(result, costs);
}

#[test]
fn invocation_charge_for_none_result_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);

    let fee_token_client = register_token(&env, &init_data.admin);
    let fee_config = FeeConfig::Some((fee_token_client.address.clone(), 1_000_000));
    client.set_fee_config(&fee_config);

    let caller = Address::generate(&env);
    //mint fee token to caller
    fee_token_client.mint(&caller, &100_000_000);
    //get price for the first asset
    client.lastprice(&caller, &init_data.assets.first_unchecked());
    //check that fee token was deducted
    let fee_token_balance = fee_token_client.balance(&caller);
    assert_eq!(fee_token_balance, 100_000_000);
}

#[test_case(1, 5_000_000 ; "price")]
#[test_case(2, 5_750_000 ; "multi round price")]
fn invocation_charge_estimate_test(periods: u32, expected_fee: i128) {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);

    let fee_token_client = register_token(&env, &init_data.admin);
    let fee_config = FeeConfig::Some((fee_token_client.address.clone(), 1_000_000));
    client.set_fee_config(&fee_config);
    let costs = Vec::from_array(&env, [1_500_000, 5_000_000]);
    client.set_invocation_costs_config(&costs);

    let fee = client.estimate_cost(&periods);
    assert_eq!(fee, expected_fee);
}

#[test]
fn check_extending_asset_ttl() {
    //initialize contract
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);

    //set fee config
    let fee_token_client = register_token(&env, &init_data.admin);
    let fee_config = FeeConfig::Some((fee_token_client.address.clone(), 1_000_000));
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
    fee_token_client.mint(&sponsor, &10_000_000);

    //check the extending
    client.extend_asset_ttl(&sponsor, &new_asset, &1_000_000);
    assert_eq!(client.expires(&new_asset), Some(87_300));

    //check that expiration records length matches assets length
    env.as_contract(&client.address, || {
        let expiration: Vec<u64> = env.storage().instance().get(&"expiration").unwrap();
        assert_eq!(assets::load_all_assets(&env).len(), expiration.len());
    });
}
