#![cfg(test)]
extern crate std;

use soroban_sdk::{log, testutils::Address as _, Address, Bytes, Env, Vec};
use test_case::test_case;

use crate::{
    assets, mapping, prices, settings,
    testutils::{generate_assets, generate_update_record_mask},
};

#[test_case(1, 0, 14)]
#[test_case(0, 1, 14)]
#[test_case(0, 0, 14)]
#[test_case(-1i128, 0, 14)]
#[test_case(0, -1i128, 14)]
#[test_case(-1, -1, 14)]
#[test_case(1000000000000000000000, 5, 18)]
#[test_case(5000000000000000000000000000000, 10000000000, 14)]
#[test_case(i128::MAX, 1, 14)]
fn fixed_div_floor_failed_tests(a: i128, b: i128, decimals: u32) {
    let result = prices::fixed_div_floor(a.clone(), b, decimals);
    assert!(result.is_none());
}

#[test_case(154467226919499, 133928752749774, 14, 115335373284703)]
#[test_case(i128::MAX / 100, 231731687303715884105728, 14, 734216306110962248249052545)]
#[test_case(231731687303715884105728, i128::MAX / 100, 14, 13)]
#[test_case(i128::MAX, i128::MAX, 14, 100000000000000)]
fn fixed_div_floor_success_tests(a: i128, b: i128, decimals: u32, expected: i128) {
    let result = prices::fixed_div_floor(a.clone(), b, decimals);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn position_encoding_bitmask_test() {
    let e = Env::default();
    let mut mask = Bytes::new(&e);
    let total_assets = 5;
    let mut total_periods = 130;
    for period in 0..total_periods {
        let mut updates = Vec::new(&e);
        for asset_index in 0..total_assets {
            let price = match asset_index > 0 && (period % asset_index == 0) {
                true => 1,
                _ => 0,
            };
            updates.push_back(price);
        }
        mask = mapping::update_history_mask(mask, &updates, 1);
    }
    log!(&e, "entire mask", mask);

    //check previous prices
    let period_diff = if total_periods > 255 {
        total_periods - 255
    } else {
        0
    };
    total_periods = std::cmp::min(total_periods, 255);
    for period in 0..total_periods {
        let check_period = total_periods - period - 1;
        for asset_index in 0..total_assets {
            let expected = asset_index > 0 && ((period + period_diff) % asset_index == 0);
            let found = mapping::check_history_updated(&mask, asset_index, check_period);
            assert_eq!(found, expected);
        }
    }
}

#[test]
fn update_record_bitmask_test() {
    let e = Env::default();
    let iterations = 70;

    let mut updates = std::collections::VecDeque::from([0i128; 254]);
    for i in 0..iterations {
        for asset_index in 0..updates.len() {
            let price = match i & asset_index == 0 {
                true => 1,
                _ => 0,
            };
            updates[asset_index] = price;
        }
        let mask = generate_update_record_mask(&e, &updates);
        //log!(&e, "entire mask", mask);
        for (asset_index, price) in updates.iter().enumerate() {
            assert_eq!(
                mapping::check_period_updated(&mask, asset_index as u32),
                price > &0
            );
        }
    }
}

#[test_case(0, 0; "zero timestamp")]
#[test_case(600_000, 600_000; "aligned timestamp")]
#[test_case(623_456, 600_000; "non-aligned timestamp")]
fn normalize_timestamp_test(input: u64, expected: u64) {
    let e = Env::default();
    //register contract to have storage available
    let contract = e.register_stellar_asset_contract_v2(Address::generate(&e));
    e.as_contract(&contract.address(), || {
        settings::set_resolution(&e, 300_000);
        let normalized = crate::timestamps::normalize(&e, input);
        assert_eq!(normalized, expected);
    });
}

#[test]
fn ensure_expirations_test() {
    let e = Env::default();
    //register contract to have storage available
    let contract = e.register_stellar_asset_contract_v2(Address::generate(&e));
    e.as_contract(&contract.address(), || {
        //add three assets with zero initial expiration
        let test_assets = generate_assets(&e, 3, 0);
        assets::add_assets(&e, test_assets.clone(), 0);
        //bump expiration for the first asset
        assets::ensure_expirations(&e, &Vec::from_array(&e, [(0u32, 5_000u64)]));
        assert_eq!(
            assets::expires(&e, test_assets.get_unchecked(0)),
            Some(5_000)
        );
        //a batch applies every update independently, and a smaller value never
        //decreases the expiration already recorded for the asset
        assets::ensure_expirations(&e, &Vec::from_array(&e, [(0u32, 1_000u64), (1, 9_000)]));
        assert_eq!(
            assets::expires(&e, test_assets.get_unchecked(0)),
            Some(5_000)
        );
        assert_eq!(
            assets::expires(&e, test_assets.get_unchecked(1)),
            Some(9_000)
        );
        //assets missing from the batch are untouched
        assert_eq!(assets::expires(&e, test_assets.get_unchecked(2)), Some(0));
        //an empty batch is a no-op
        assets::ensure_expirations(&e, &Vec::new(&e));
        assert_eq!(
            assets::expires(&e, test_assets.get_unchecked(1)),
            Some(9_000)
        );
    });
}
