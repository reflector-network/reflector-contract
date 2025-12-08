#![cfg(test)]
extern crate std;

use crate::{price_oracle, prices, tests::util_tests::generate_update_record_mask, types};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Env, Symbol,
};
use test_case::test_case;

#[should_panic]
#[test_case(0; "zero timestamp")]
#[test_case(900_001 ; "invalid aligned timestamp")]
#[test_case(600_000 ; "valid timestamp same as last")]
#[test_case(300_000 ; "valid timestamp less than last")]
fn invalid_timestamp_update_test(ts: u64) {
    let e = Env::default();
    //register contract to have storage available
    let contract = e.register_stellar_asset_contract_v2(Address::generate(&e));
    e.mock_all_auths();
    e.as_contract(&contract.address(), || {
        price_oracle::PriceOracleContractBase::config(
            &e,
            types::ConfigData {
                admin: Address::generate(&e),
                history_retention_period: 86_400_000,
                assets: vec![&e, types::Asset::Other(Symbol::new(&e, "ASSET_A"))],
                base_asset: types::Asset::Other(Symbol::new(&e, "BASE_ASSET")),
                decimals: 8,
                resolution: 300_000,
                cache_size: 10,
                fee_config: types::FeeConfig::None,
            },
            100,
        );
        prices::set_last_timestamp(&e, 600_000);
        e.ledger().set(LedgerInfo {
            timestamp: 9001,
            ..e.ledger().get()
        });
    });

    e.as_contract(&contract.address(), || {
        price_oracle::PriceOracleContractBase::set_price(
            &e,
            types::PriceUpdate {
                prices: vec![&e, 12345678i128],
                mask: generate_update_record_mask(&e, &vec![&e, 12345678i128]),
            },
            ts,
        );
    });
}

#[test]
fn price_update_test() {
    let e = Env::default();
    //register contract to have storage available
    let contract = e.register_stellar_asset_contract_v2(Address::generate(&e));
    e.mock_all_auths();
    e.as_contract(&contract.address(), || {
        price_oracle::PriceOracleContractBase::config(
            &e,
            types::ConfigData {
                admin: Address::generate(&e),
                history_retention_period: 86_400_000,
                assets: vec![&e, types::Asset::Other(Symbol::new(&e, "ASSET_A"))],
                base_asset: types::Asset::Other(Symbol::new(&e, "BASE_ASSET")),
                decimals: 8,
                resolution: 300_000,
                cache_size: 10,
                fee_config: types::FeeConfig::None,
            },
            100,
        );
        prices::set_last_timestamp(&e, 600_000);
        e.ledger().set(LedgerInfo {
            timestamp: 9001,
            ..e.ledger().get()
        });
    });

    e.as_contract(&contract.address(), || {
        price_oracle::PriceOracleContractBase::set_price(
            &e,
            types::PriceUpdate {
                prices: vec![&e, 12345678i128],
                mask: generate_update_record_mask(&e, &vec![&e, 12345678i128]),
            },
            900_000,
        );
    });
}
