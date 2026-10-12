#![cfg(test)]
extern crate std;

use crate::{BeamOracleContract, BeamOracleContractClient};
use oracle::init_contract_with_admin;

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
