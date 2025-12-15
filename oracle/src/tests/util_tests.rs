#![cfg(test)]
extern crate std;

use soroban_sdk::{log, testutils::Address as _, Address, Bytes, Env, Vec};
use test_case::test_case;

use crate::{mapping, settings};

pub fn generate_update_record_mask(e: &Env, updates: &Vec<i128>) -> Bytes {
    let mut mask = [0u8; 32];
    for (asset, price) in updates.iter().enumerate() {
        if price > 0 {
            let (byte, bitmask) = mapping::resolve_period_update_mask_position(asset as u32);
            let i = byte as usize;
            let bytemask = mask[i] | bitmask;
            mask[i] = bytemask
        }
    }
    Bytes::from_array(e, &mask)
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
        mask = mapping::update_history_mask(&e, mask, &updates);
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

    let mut updates = Vec::from_array(&e, [0i128; 254]);
    for i in 0..iterations {
        for asset_index in 0..updates.len() {
            let price = match i & asset_index == 0 {
                true => 1,
                _ => 0,
            };
            updates.set(asset_index, price);
        }
        let mask = generate_update_record_mask(&e, &updates);
        //log!(&e, "entire mask", mask);
        for (asset_index, price) in updates.iter().enumerate() {
            assert_eq!(
                mapping::check_period_updated(&mask, asset_index as u32),
                price > 0
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
