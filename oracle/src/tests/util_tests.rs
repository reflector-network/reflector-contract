#![cfg(test)]
extern crate std;

use soroban_sdk::{log, testutils::Address as _, Address, Bytes, Env, Vec};
use test_case::test_case;

use crate::{
    mapping, prices, settings,
    testutils::{generate_assets, generate_random_updates, generate_update_record_mask},
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

#[test_case(0, &[0xFF; 32], &[0xFF; 32]; "zero shift")]
#[test_case(8, &{let mut arr = [0u8; 32]; arr[31] = 0xFF; arr}, &{let mut arr = [0u8; 32]; arr[30] = 0xFF; arr}; "shift by 8 bits")]
#[test_case(4, &{let mut arr = [0u8; 32]; arr[30] = 0x12; arr[31] = 0x34; arr}, &{let mut arr = [0u8; 32]; arr[29] = 0x01; arr[30] = 0x23; arr[31] = 0x40; arr}; "shift by 4 bits")]
#[test_case(137, &{let mut arr = [0u8; 32]; arr[31] = 0xFF; arr}, &{let mut arr = [0u8; 32]; arr[13] = 0x01; arr[14] = 0xFE; arr}; "shift by 137 bits")]
#[test_case(256, &[0xFF; 32], &[0x00; 32]; "overflow 256 bits")]
#[test_case(255, &{let mut arr = [0u8; 32]; arr[31] = 0x01; arr}, &{let mut arr = [0u8; 32]; arr[0] = 0x80; arr}; "shift by 255 bits")]
fn shift_left_test(shift: u32, input: &[u8; 32], expected: &[u8; 32]) {
    let e = Env::default();
    let bytes = Bytes::from_array(&e, input);
    let result = mapping::shift_left(bytes, 0, shift);

    for i in 0..32 {
        assert_eq!(result.get(i).unwrap(), expected[i as usize]);
    }
}

#[test_case(&[0x00; 32], &{let mut arr = [0u8; 32]; arr[31] = 0x01; arr}; "add one to zero")]
#[test_case(&{let mut arr = [0u8; 32]; arr[31] = 0xFF; arr}, &{let mut arr = [0u8; 32]; arr[30] = 0x01; arr}; "add one with carry")]
#[test_case(&[0xFF; 32], &[0x00; 32]; "add one all ones")]
#[test_case(&{let mut arr = [0u8; 32]; arr[28] = 0x01; arr[29] = 0xFF; arr[30] = 0xFF; arr[31] = 0xFF; arr}, &{let mut arr = [0u8; 32]; arr[28] = 0x02; arr}; "add one multiple carry")]
fn mark_updated_test(input: &[u8; 32], expected: &[u8; 32]) {
    let e = Env::default();
    let bytes = Bytes::from_array(&e, input);
    let result = mapping::mark_updated(bytes, 0);

    for i in 0..32 {
        assert_eq!(result.get(i).unwrap(), expected[i as usize]);
    }
}

#[test_case(1; "no gaps")]
#[test_case(254; "gap of 254 rounds")]
#[test_case(300; "gap of 300 rounds")]
fn mask_the_same_with_history_mask_legacy_test(gap: u32) {
    let env = Env::default();
    let assets = generate_assets(&env, 150, 0);
    //init history mask
    let updates_delta = 1;
    let history_mask = Bytes::new(&env);
    let updates = generate_random_updates(&env, &assets, 100);
    let prices = Vec::from_iter(&env, updates.1.into_iter());
    let legacy_mask =
        mapping::update_history_mask_legacy(&env, history_mask.clone(), &prices, updates_delta);
    let new_mask = mapping::update_history_mask(history_mask, &prices, updates_delta);
    assert_eq!(legacy_mask, new_mask);

    //set prices after gap
    let history_mask = legacy_mask;
    let updates = generate_random_updates(&env, &assets, 100);
    let prices = Vec::from_iter(&env, updates.1.into_iter());
    let legacy_mask = mapping::update_history_mask_legacy(&env, history_mask.clone(), &prices, gap);
    let new_mask = mapping::update_history_mask(history_mask, &prices, gap);
    assert_eq!(legacy_mask, new_mask);
}
