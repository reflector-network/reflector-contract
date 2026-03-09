#![cfg(test)]

use oracle::init_contract_with_admin;
use oracle::testutils::{generate_updates, set_ledger_timestamp};
use oracle::types::PriceData;
use test_case::test_case;

extern crate std;
use std::collections::VecDeque;

use crate::{PulseOracleContract, PulseOracleContractClient};

#[test_case(5, 600, VecDeque::from([]), None; "no updates")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 104)]), Some(()); "price increase below threshold")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 105)]), None; "price increase equal to threshold")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 106)]), None; "price increase above threshold")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 96)]), Some(()); "price drop below threshold")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 5000)]), None; "massive price increase")]
#[test_case(5, 600, VecDeque::from([(2100, 100)]), None; "no history for dev check")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 100)]), Some(()); "no price change")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (12000, 102)]), Some(()); "big gap with recent update")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (12000, 0)]), None; "big gap without recent update")]
#[test_case(0, 600, VecDeque::from([(2100, 100)]), Some(()); "price within max age")]
#[test_case(0, 900, VecDeque::from([(2100, 100), (3000, 0)]), Some(()); "price ts equal to max age")]
#[test_case(0, 100, VecDeque::from([(2100, 100), (2400, 0), (2700, 0), (3000, 0)]), None; "price ts older than max age")]
#[test_case(5, 600, VecDeque::from([(2100, 0), (2400, 0)]), None; "only skipped prices")]
#[test_case(5, 900, VecDeque::from([(2100, 100), (2400, 0), (2700, 0), (3000, 102)]), Some(()); "multiple skipped rounds between prices")]
fn yieldbox_get_price_test(
    max_dev: u32,
    max_age: u64,
    updates: VecDeque<(u32, i128)>,
    expected: Option<()>,
) {
    let (env, client, init_data) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    for (_, (timestamp, price)) in updates.iter().enumerate() {
        set_ledger_timestamp(&env, (*timestamp).into());
        if price > &0 {
            let updates = generate_updates(&env, &init_data.assets, *price);
            client.set_price(&updates.0, &(timestamp * 1000).into());
        }
    }

    let oldest_timestamp = env.ledger().timestamp() - max_age;
    let mut price: Option<PriceData> = None;
    if max_dev > 0 {
        let prices = client.prices(&init_data.assets.get_unchecked(0), &4);
        if let Some(prices) = prices {
            if prices.len() >= 2 {
                let first_price = prices.get_unchecked(0);
                let second_price = prices.get_unchecked(1);
                let diff = (first_price.price - second_price.price).abs();
                let max_dev = (second_price.price * max_dev as i128) / 100;
                if diff < max_dev {
                    price = Some(first_price);
                }
            }
        }
    } else {
        let round_timestamp = client.last_timestamp();
        if round_timestamp >= oldest_timestamp {
            price = client.price(&init_data.assets.get_unchecked(0), &round_timestamp);
        }
    }

    if let Some(ref mut price_data) = price {
        if price_data.timestamp >= oldest_timestamp {
            assert_eq!(Some(()), expected);
            return;
        }
    }

    assert_eq!(price.is_none(), expected.is_none());
}

#[test_case(5, 600, VecDeque::from([]), None; "no updates")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 104)]), Some(()); "price increase below threshold")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 105)]), Some(()); "price increase equal to threshold")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 106)]), None; "price increase above threshold")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 96)]), Some(()); "price drop below threshold")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 5000)]), None; "massive price increase")]
#[test_case(5, 600, VecDeque::from([(2100, 100)]), None; "no history for dev check")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (2400, 100)]), Some(()); "no price change")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (12000, 102)]), None; "big gap with recent update")]
#[test_case(5, 600, VecDeque::from([(2100, 100), (12000, 0)]), None; "big gap without recent update")]
#[test_case(0, 600, VecDeque::from([(2100, 100)]), Some(()); "price within max age")]
#[test_case(0, 900, VecDeque::from([(2100, 100), (3000, 0)]), Some(()); "price ts equal to max age")]
#[test_case(0, 100, VecDeque::from([(2100, 100), (2400, 0), (2700, 0), (3000, 0)]), None; "price ts older than max age")]
#[test_case(5, 600, VecDeque::from([(2100, 0), (2400, 0)]), None; "only skipped prices")]
#[test_case(5, 900, VecDeque::from([(2100, 100), (2400, 0), (2700, 0), (3000, 102)]), Some(()); "multiple skipped rounds between prices")]
fn fixed_pool_get_price_test(
    max_dev: u32,
    max_age: u64,
    updates: VecDeque<(u32, i128)>,
    expected: Option<()>,
) {
    let (env, client, init_data) =
        init_contract_with_admin!(PulseOracleContract, PulseOracleContractClient, true);

    for (_, (timestamp, price)) in updates.iter().enumerate() {
        set_ledger_timestamp(&env, (*timestamp).into());
        if price > &0 {
            let updates = generate_updates(&env, &init_data.assets, *price);
            client.set_price(&updates.0, &(timestamp * 1000).into());
        }
    }

    let oldest_timestamp = env.ledger().timestamp() - max_age;
    let round_timestamp = client.last_timestamp();
    let oracle_resolution = client.resolution() as u64;
    let mut price: Option<PriceData> = None;
    let mut next_timestamp = round_timestamp.clone();
    while price.is_none() && next_timestamp >= oldest_timestamp {
        price = client.price(&init_data.assets.get_unchecked(0), &next_timestamp);
        next_timestamp = next_timestamp - oracle_resolution;
    }

    // a price was found
    if let Some(ref price) = price {
        // if we need to verify the max dev, look for an older price
        if max_dev > 0 {
            // have a valid price for the asset from the oracle. Attempt to fetch an older price
            // to validate max_dev. Looks at most `max_age / resolution` prices back from the most recent
            // price.
            let max_steps = max_age / oracle_resolution;
            let mut old_price: Option<PriceData> = None;
            for _ in 0..max_steps {
                old_price = client.price(&init_data.assets.get_unchecked(0), &next_timestamp);
                if old_price.is_some() {
                    break;
                }
                next_timestamp = next_timestamp - oracle_resolution;
            }
            if let Some(old_price) = old_price {
                // check the price is within the max_dev, return None if it is not
                let diff = (price.price - old_price.price).abs();
                let max_dev = (old_price.price * max_dev as i128) / 100;
                if diff > max_dev {
                    assert_eq!(None, expected);
                    return;
                }
            } else {
                // no old price found, so we cannot verify the max_dev, return None
                assert_eq!(None, expected);
                return;
            }
        }

        // normalize the decimals and verify the timestamp returned
        if price.timestamp >= oldest_timestamp {
            assert_eq!(Some(()), expected);
            return;
        }
    }

    assert_eq!(price.is_none(), expected.is_none());
}
