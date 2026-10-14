#![cfg(test)]
extern crate std;

use crate::{BeamOracleContract, BeamOracleContractClient};
use oracle::init_contract_with_admin;
use oracle::testutils::{
    generate_single_asset_update, generate_updates, normalize_price, register_token,
    set_ledger_timestamp,
};
use oracle::types::{FeeConfig, PriceData};
use soroban_sdk::{testutils::Address as _, Address, Vec};

#[test]
fn version_test() {
    let (_env, client, _) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
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
fn lastprice_partial_update_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), 1_000_000)));

    let account = Address::generate(&env);
    fee_token.mint(&account, &1_000_000);
    let asset = init_data.assets.get_unchecked(0);
    client.track(
        &account,
        &account,
        &Vec::from_array(&env, [asset.clone()]),
        &1_000_000,
    );

    //all assets quoted in the first round
    let updates = generate_updates(&env, &init_data.assets, normalize_price(100));
    client.set_price(&updates.0, &600_000);
    //only another asset quoted in the second round
    set_ledger_timestamp(&env, 1_200);
    let partial =
        generate_single_asset_update(&env, init_data.assets.len(), 1, normalize_price(200));
    client.set_price(&partial, &900_000);

    //the entitled reader gets the most recent record of the omitted asset
    assert_eq!(
        client.lastprice(&account, &asset),
        Some(PriceData {
            price: normalize_price(100),
            timestamp: 600
        })
    );
}
