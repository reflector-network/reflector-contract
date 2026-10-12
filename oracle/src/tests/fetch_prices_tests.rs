#![cfg(test)]
extern crate alloc;
extern crate std;
use alloc::string::ToString;

use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, Symbol, Vec};

use test_case::test_case;

use crate::testutils::set_ledger_timestamp;
use crate::*;

fn generate_update_record_mask(e: &Env, updates: &Vec<i128>) -> Bytes {
    let mut mask = [0u8; 32];
    for (asset_index, price) in updates.iter().enumerate() {
        if price > 0 {
            let (byte, bitmask) = mapping::resolve_period_update_mask_position(asset_index as u32);
            let i = byte as usize;
            let bytemask = mask[i] | bitmask;
            mask[i] = bytemask
        }
    }
    Bytes::from_array(e, &mask)
}

fn generate_updates(env: &Env, assets: &Vec<types::Asset>, price: i128) -> types::PriceUpdate {
    let mut updates = Vec::new(&env);
    for _ in assets.iter() {
        updates.push_back(price);
    }
    let mask = generate_update_record_mask(env, &updates);
    types::PriceUpdate {
        prices: updates,
        mask,
    }
}

#[test_case(600_000, 8, 600_000, 2; "skipped 5 rounds")]
#[test_case(600_000, 30, 600_000, 2; "skipped 30 rounds")]
fn store_prices_test(
    first_timestamp: u64,
    rounds_gap: u64,
    expected_first_price_ts: u64,
    expected_prices_count: u32,
) {
    let e = Env::default();

    set_ledger_timestamp(&e, 600_000);

    let mut assets = Vec::new(&e);
    for i in 0..255 {
        assets.push_back(types::Asset::Other(Symbol::new(
            &e,
            &("ASSET_".to_string() + &i.to_string()),
        )));
    }
    //register asset contract just to have storage
    let contract_id = e.register_stellar_asset_contract_v2(Address::generate(&e));
    e.as_contract(&contract_id.address(), || {
        let timeframe: u64 = 300_000;
        //the base asset is required to validate the assets being added
        settings::init(
            &e,
            &types::Asset::Other(Symbol::new(&e, "BASE_ASSET")),
            14,
            timeframe as u32,
            0,
            2,
            &types::FeeConfig::None,
        );
        protocol::set_protocol_version(&e, 2);

        assets::add_assets(&e, assets.clone(), 180);
        fn set_price(e: &Env, timestamp: u64, assets: &Vec<types::Asset>) {
            let updates = generate_updates(e, &assets, 100);
            let asset_prices = prices::extract_update_record_prices(e, &updates, assets.len());
            let legacy_update = updates.prices.clone();
            //store history timestamps for all assets
            prices::update_history_mask(e, &asset_prices, timestamp);
            prices::store_prices(e, updates, timestamp, legacy_update);
        }

        let mut timestamp = first_timestamp;
        set_price(&e, timestamp, &assets);
        timestamp += timeframe * rounds_gap;
        set_price(&e, timestamp, &assets);

        set_ledger_timestamp(&e, timestamp / 1000);

        let prices = prices::load_prices(&e, 0, 3);
        assert_ne!(prices, None);
        let prices = prices.unwrap();
        assert_eq!(prices.len(), expected_prices_count);
        assert_eq!(prices.get_unchecked(0).timestamp, timestamp / 1000); //latest price
        assert_eq!(
            prices
                .get(1)
                .unwrap_or_else(|| types::PriceData {
                    price: 0,
                    timestamp: 0
                })
                .timestamp,
            expected_first_price_ts / 1000
        );
    });

    e.cost_estimate().budget().print();
}

#[test]
fn price_beyond_history_window_test() {
    let e = Env::default();
    let timeframe: u64 = 300_000;
    let first = 1_760_000_100_000 / timeframe * timeframe;
    let last = first + timeframe * 300;
    set_ledger_timestamp(&e, last / 1000);

    let mut assets = Vec::new(&e);
    assets.push_back(types::Asset::Other(Symbol::new(&e, "ASSET_0")));
    assets.push_back(types::Asset::Other(Symbol::new(&e, "ASSET_1")));
    //register asset contract just to have storage
    let contract_id = e.register_stellar_asset_contract_v2(Address::generate(&e));
    e.as_contract(&contract_id.address(), || {
        settings::init(
            &e,
            &types::Asset::Other(Symbol::new(&e, "BASE_ASSET")),
            14,
            timeframe as u32,
            0,
            2,
            &types::FeeConfig::None,
        );
        protocol::set_protocol_version(&e, 2);
        assets::add_assets(&e, assets.clone(), 180);
        //record a price for a single asset
        fn set_price(e: &Env, timestamp: u64, asset_index: usize) {
            let mut all = Vec::from_array(e, [0i128, 0]);
            all.set(asset_index as u32, 100);
            let update = types::PriceUpdate {
                prices: Vec::from_array(e, [100i128]),
                mask: generate_update_record_mask(e, &all),
            };
            prices::update_history_mask(e, &all, timestamp);
            prices::store_prices(e, update, timestamp, all);
        }

        //ASSET_0 is last quoted 300 periods before the latest round
        set_price(&e, first, 0);
        set_price(&e, last, 1);

        //an unbounded walk back through the history fails with a budget error instead of hanging
        e.cost_estimate().budget().reset_default();
        assert_eq!(prices::load_prices(&e, 0, 3), None);
        assert_eq!(prices::load_prices(&e, 1, 3).unwrap().len(), 1);
    });
}
