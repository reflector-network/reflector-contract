#![cfg(test)]
extern crate alloc;
extern crate std;

use super::*;
use alloc::string::ToString;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo, MockAuth, MockAuthInvoke},
    Address, Env, String, Symbol, TryIntoVal,
};
use std::panic::{self, AssertUnwindSafe};

use {extensions::i128_extensions::I128Extensions, types::asset::Asset};
use soroban_sdk::{contract, contractimpl};
use types::price_data::PriceData;

const RESOLUTION: u32 = 300_000;
const DECIMALS: u32 = 14;

// Mock FX Oracle Contract for testing
#[contract]
pub struct MockFxOracle;

#[contractimpl]
impl MockFxOracle {
    // Set error mode: "zero_timestamp", "none_price", "negative_price", "zero_price", or None for normal operation
    pub fn set_error_mode(e: Env, mode: Option<Symbol>) {
        e.storage().temporary().set(&Symbol::new(&e, "error_mode"), &mode);
    }

    // Set a custom timestamp for testing timestamp drift
    pub fn set_custom_timestamp(e: Env, timestamp: u64) {
        e.storage().temporary().set(&Symbol::new(&e, "custom_timestamp"), &timestamp);
    }

    pub fn last_timestamp(e: Env) -> u64 {
        // Check for custom timestamp first
        let custom_ts: Option<u64> = e.storage().temporary().get(&Symbol::new(&e, "custom_timestamp"));
        if let Some(ts) = custom_ts {
            return ts;
        }
        
        let mode: Option<Option<Symbol>> = e.storage().temporary().get(&Symbol::new(&e, "error_mode"));
        if let Some(Some(m)) = mode {
            if m == Symbol::new(&e, "zero_timestamp") {
                return 0;
            }
        }
        // Return a valid timestamp (in seconds)
        e.ledger().timestamp()
    }

    pub fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        // Get timestamp from last_timestamp()
        let timestamp = Self::last_timestamp(e.clone());
        if timestamp == 0 {
            return None;
        }
        // Use the same logic as price() but with the timestamp from last_timestamp()
        Self::price(e, asset, timestamp)
    }

    pub fn price(e: Env, asset: Asset, _timestamp: u64) -> Option<PriceData> {
        let mode: Option<Option<Symbol>> = e.storage().temporary().get(&Symbol::new(&e, "error_mode"));
        
        if let Some(Some(m)) = mode {
            if m == Symbol::new(&e, "none_price") {
                return None;
            } else if m == Symbol::new(&e, "negative_price") {
                let timestamp = Self::last_timestamp(e.clone());
                return Some(PriceData {
                    price: -100_000_000_000_000i128, // Negative price
                    timestamp,
                });
            } else if m == Symbol::new(&e, "zero_price") {
                let timestamp = Self::last_timestamp(e.clone());
                return Some(PriceData {
                    price: 0, // Zero price
                    timestamp,
                });
            }
        }
        
        // Normal operation: Return mock prices for different FX symbols
        // Prices are in USD with 14 decimals
        let price = match asset {
            Asset::Other(symbol) => {
                // Compare symbols directly
                if symbol == Symbol::new(&e, "MXN") {
                    57_000_000_000_000i128  // 0.057 USD
                } else if symbol == Symbol::new(&e, "EUR") {
                    110_000_000_000_000i128  // 1.10 USD
                } else if symbol == Symbol::new(&e, "GBP") {
                    127_000_000_000_000i128  // 1.27 USD
                } else if symbol == Symbol::new(&e, "JPY") {
                    6_500_000_000_000i128    // 0.0065 USD
                } else if symbol == Symbol::new(&e, "CAD") {
                    73_000_000_000_000i128   // 0.73 USD
                } else if symbol == Symbol::new(&e, "AUD") {
                    65_000_000_000_000i128   // 0.65 USD
                } else if symbol == Symbol::new(&e, "CHF") {
                    112_000_000_000_000i128  // 1.12 USD
                } else {
                    100_000_000_000_000i128  // Default: 1.0 USD
                }
            }
            Asset::Stellar(_) => 100_000_000_000_000i128, // Default for Stellar assets
        };
        
        let timestamp = Self::last_timestamp(e);
        Some(PriceData {
            price,
            timestamp,
        })
    }
}

fn convert_to_seconds(timestamp: u64) -> u64 {
    timestamp / 1000
}

fn init_contract_with_admin<'a>() -> (Env, PriceOracleContractClient<'a>, ConfigData, Address) {
    let env = Env::default();

    //set timestamp to 900 seconds
    let ledger_info = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp: 900,
        ..ledger_info
    });

    let admin = Address::generate(&env);

    // Register mock FX oracle contract
    let mock_oracle_id = env.register(MockFxOracle, ());

    let contract_id = &Address::from_string(&String::from_str(
        &env,
        "CDXHQTB7FGRMWTLJJLNI3XPKVC6SZDB5SFGZUYDPEGQQNC4G6CKE4QRC",
    ));

    env.register_at(contract_id, PriceOracleContract, ());
    let client: PriceOracleContractClient<'a> = PriceOracleContractClient::new(&env, contract_id);

    env.cost_estimate().budget().reset_unlimited();

    // Reset error mode to None (normal operation)
    let mock_oracle_client = MockFxOracleClient::new(&env, &mock_oracle_id);
    mock_oracle_client.set_error_mode(&None);

    let init_data = ConfigData {
        admin: admin.clone(),
        period: (100 * RESOLUTION).into(),
        base_asset: Asset::Stellar(Address::generate(&env)),
        decimals: 14,
        resolution: RESOLUTION,
        fx_oracle_address: mock_oracle_id.clone(),
        max_yield_deviation_percent: 10, // 10% for most tests
    };

    env.mock_all_auths();

    //set admin and fx oracle address
    client.config(&init_data);

    (env, client, init_data, mock_oracle_id)
}

fn normalize_price(price: i128) -> i128 {
    price * 10i128.pow(DECIMALS)
}

fn generate_assets(e: &Env, count: usize, start_index: u32) -> Vec<Asset> {
    let mut assets = Vec::new(&e);
    for i in 0..count {
        if i % 2 == 0 {
            assets.push_back(Asset::Stellar(Address::generate(&e)));
        } else {
            assets.push_back(Asset::Other(Symbol::new(
                e,
                &("ASSET_".to_string() + &(start_index + i as u32).to_string()),
            )));
        }
    }
    assets
}

fn get_updates(env: &Env, assets: &Vec<Asset>, price: i128) -> Vec<i128> {
    let mut updates = Vec::new(&env);
    for _ in assets.iter() {
        updates.push_back(price);
    }
    updates
}

// Helper to calculate expected final price given yield rate and FX price
// final_price = (yield_rate * fx_price) / 10^decimals
fn calculate_expected_price(yield_rate: i128, fx_price: i128, decimals: u32) -> i128 {
    (yield_rate * fx_price) / 10i128.pow(decimals)
}

// Helper to get FX price for a symbol (matching mock oracle)
fn get_fx_price_for_symbol(e: &Env, symbol: Symbol) -> i128 {
    if symbol == Symbol::new(e, "USD") {
        10i128.pow(14) // 1.0 USD
    } else if symbol == Symbol::new(e, "MXN") {
        57_000_000_000_000i128  // 0.057 USD
    } else if symbol == Symbol::new(e, "EUR") {
        110_000_000_000_000i128  // 1.10 USD
    } else if symbol == Symbol::new(e, "GBP") {
        127_000_000_000_000i128  // 1.27 USD
    } else if symbol == Symbol::new(e, "JPY") {
        6_500_000_000_000i128    // 0.0065 USD
    } else if symbol == Symbol::new(e, "CAD") {
        73_000_000_000_000i128   // 0.73 USD
    } else if symbol == Symbol::new(e, "AUD") {
        65_000_000_000_000i128   // 0.65 USD
    } else if symbol == Symbol::new(e, "CHF") {
        112_000_000_000_000i128  // 1.12 USD
    } else {
        // Default for unknown FX symbols (including generated ones like FX9, FX10, etc.)
        100_000_000_000_000i128  // Default: 1.0 USD
    }
}

#[test]
fn version_test() {
    let (_env, client, _init_data, _) = init_contract_with_admin();
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
fn init_test() {
    let (_env, client, init_data, _) = init_contract_with_admin();

    let address = client.admin();
    assert_eq!(address.unwrap(), init_data.admin.clone());

    let base = client.base();
    assert_eq!(base, init_data.base_asset);

    let resolution = client.resolution();
    assert_eq!(resolution, RESOLUTION / 1000);

    let period = client.period().unwrap();
    assert_eq!(period, init_data.period / 1000);

    let decimals = client.decimals();
    assert_eq!(decimals, DECIMALS);

    let assets = client.assets();
    // Assets should be empty initially (added separately via add_assets)
    assert_eq!(assets.len(), 0);
}

#[test]
fn set_price_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();
    //set prices for assets
    client.set_price(&updates, &timestamp);
}

#[test]
#[should_panic]
fn set_price_zero_timestamp_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 0;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();
    //set prices for assets
    client.set_price(&updates, &timestamp);
}

#[test]
#[should_panic]
fn set_price_invalid_timestamp_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 600_001;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();
    //set prices for assets
    client.set_price(&updates, &timestamp);
}

#[test]
#[should_panic]
fn set_price_future_timestamp_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 1_200_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();
    //set prices for assets
    client.set_price(&updates, &timestamp);
}

#[test]
fn last_price_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();
    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(105));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    //check last prices
    // Asset 1 uses MXN (second FX), yield_rate = 105 * 10^14 (1.05)
    // Expected price = (1.05 * 0.057) = 0.05985 = 5985000000000000
    let fx_price = get_fx_price_for_symbol(&env, fxs.get_unchecked(1));
    let expected_price = calculate_expected_price(normalize_price(105), fx_price, DECIMALS);
    let result = client.lastprice(&assets.get_unchecked(1));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: expected_price,
            timestamp: convert_to_seconds(900_000)
        })
    );
}

#[test]
fn last_timestamp_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let mut result = client.last_timestamp();

    assert_eq!(result, 0);

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    result = client.last_timestamp();

    assert_eq!(result, convert_to_seconds(600_000));
}

#[test]
fn add_assets_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 10, 0);
    let fxs = generate_fxs(&env, 10);

    env.mock_all_auths();

    client.add_assets(&assets, &fxs);

    let result = client.assets();

    assert_eq!(result.len(), 10);
    for (i, asset) in assets.iter().enumerate() {
        assert_eq!(result.get_unchecked(i as u32), asset.clone());
    }
}

#[test]
#[should_panic]
fn add_assets_duplicate_test() {
    let (env, client, _, _) = init_contract_with_admin();

    let mut assets = Vec::new(&env);
    let duplicate_asset = Asset::Other(Symbol::new(&env, &("ASSET_DUPLICATE")));
    assets.push_back(duplicate_asset.clone());
    assets.push_back(duplicate_asset);
    
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "USD"));
    fxs.push_back(Symbol::new(&env, "MXN"));

    env.mock_all_auths();

    client.add_assets(&assets, &fxs);
}

#[test]
#[should_panic]
fn assets_update_overflow_test() {
    let (env, client, _, _) = init_contract_with_admin();

    env.mock_all_auths();

    env.cost_estimate().budget().reset_unlimited();

    let mut assets = Vec::new(&env);
    let mut fxs = Vec::new(&env);
    for i in 1..=256 {
        assets.push_back(Asset::Other(Symbol::new(
            &env,
            &("Asset".to_string() + &i.to_string()),
        )));
        fxs.push_back(Symbol::new(&env, "USD"));
    }

    client.add_assets(&assets, &fxs);
}

#[test]
#[should_panic]
fn prices_update_overflow_test() {
    let (env, client, _, _) = init_contract_with_admin();

    env.mock_all_auths();

    env.cost_estimate().budget().reset_unlimited();

    let mut updates = Vec::new(&env);
    for i in 1..=256 {
        updates.push_back(normalize_price(i as i128 + 1));
    }
    client.set_price(&updates, &600_000);
}

#[test]
fn set_period_test() {
    let (env, client, _, _) = init_contract_with_admin();

    let period = 100_000;

    env.mock_all_auths();

    client.set_period(&period);

    let result = client.period().unwrap();

    assert_eq!(result, convert_to_seconds(period));
}

#[test]
fn get_price_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 3, 0);
    let fxs = generate_fxs(&env, 3);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(105));

    client.set_price(&updates, &timestamp);

    //check last prices
    // Asset 1 uses MXN (second FX), yield_rate = 105 * 10^14 (1.05)
    // Expected price = (1.05 * 0.057) = 0.05985 = 5985000000000000
    let fx_price = get_fx_price_for_symbol(&env, fxs.get_unchecked(1));
    let expected_price_105 = calculate_expected_price(normalize_price(105), fx_price, DECIMALS);
    let expected_price_100 = calculate_expected_price(normalize_price(100), fx_price, DECIMALS);
    
    let mut result = client.lastprice(&assets.get_unchecked(1));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: expected_price_105,
            timestamp: convert_to_seconds(900_000)
        })
    );

    //check price at 899_000
    result = client.price(&assets.get_unchecked(1), &convert_to_seconds(899_000));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: expected_price_100,
            timestamp: convert_to_seconds(600_000)
        })
    );
}

#[test]
fn get_lastprice_delayed_update_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 300_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    client.set_price(&updates, &timestamp);

    //check last prices
    let result = client.lastprice(&assets.get_unchecked(1));
    assert_eq!(result, None);
}

#[test]
fn get_x_last_price_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 3, 0);
    let fxs = generate_fxs(&env, 3);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    client.set_price(&updates, &timestamp);

    //check last x price
    // Asset 1 uses MXN (0.057), Asset 2 uses EUR (1.10)
    // Both have yield_rate = 100 * 10^14 (1.0)
    // Asset 1 price = 1.0 * 0.057 = 0.057
    // Asset 2 price = 1.0 * 1.10 = 1.10
    // Cross price = 0.057 / 1.10 = 0.051818... = 51818181818181 (with 14 decimals)
    let fx_price_1 = get_fx_price_for_symbol(&env, fxs.get_unchecked(1));
    let fx_price_2 = get_fx_price_for_symbol(&env, fxs.get_unchecked(2));
    let price_1 = calculate_expected_price(normalize_price(100), fx_price_1, DECIMALS);
    let price_2 = calculate_expected_price(normalize_price(100), fx_price_2, DECIMALS);
    let expected_x_price = (price_1 * 10i128.pow(DECIMALS)) / price_2;
    
    let result = client.x_last_price(&assets.get_unchecked(1), &assets.get_unchecked(2));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: expected_x_price,
            timestamp: convert_to_seconds(600_000)
        })
    );
}

#[test]
fn get_x_price_with_zero_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 600_000;
    let mut updates = get_updates(&env, &assets, normalize_price(100));
    updates.set(1, 0);

    env.mock_all_auths();

    //set prices for assets - zero prices are skipped, so second asset won't have a price
    client.set_price(&updates, &timestamp);

    let result = client.x_price(
        &assets.get(0).unwrap(),
        &assets.get(1).unwrap(),
        &convert_to_seconds(timestamp),
    );
    assert_eq!(result, None);
}

#[test]
fn get_x_price_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 3, 0);
    let fxs = generate_fxs(&env, 3);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(105));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    //check last prices
    // Asset 1 uses MXN (0.057), Asset 2 uses EUR (1.10)
    // Both have yield_rate = 105 * 10^14 (1.05) at timestamp 900_000
    // Asset 1 price = 1.05 * 0.057 = 0.05985
    // Asset 2 price = 1.05 * 1.10 = 1.155
    // Cross price = 0.05985 / 1.155 = 0.051818... = 51818181818181 (with 14 decimals)
    let fx_price_1 = get_fx_price_for_symbol(&env, fxs.get_unchecked(1));
    let fx_price_2 = get_fx_price_for_symbol(&env, fxs.get_unchecked(2));
    let price_1_105 = calculate_expected_price(normalize_price(105), fx_price_1, DECIMALS);
    let price_2_105 = calculate_expected_price(normalize_price(105), fx_price_2, DECIMALS);
    let expected_x_price_105 = (price_1_105 * 10i128.pow(DECIMALS)) / price_2_105;
    
    // At 600_000, both have yield_rate = 100 * 10^14 (1.0)
    let price_1_100 = calculate_expected_price(normalize_price(100), fx_price_1, DECIMALS);
    let price_2_100 = calculate_expected_price(normalize_price(100), fx_price_2, DECIMALS);
    let expected_x_price_100 = (price_1_100 * 10i128.pow(DECIMALS)) / price_2_100;
    
    let mut result = client.x_last_price(&assets.get_unchecked(1), &assets.get_unchecked(2));
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: expected_x_price_105,
            timestamp: convert_to_seconds(900_000)
        })
    );

    //check price at 899_000
    result = client.x_price(
        &assets.get_unchecked(1),
        &assets.get_unchecked(2),
        &convert_to_seconds(899_000),
    );
    assert_ne!(result, None);
    assert_eq!(
        result,
        Some(PriceData {
            price: expected_x_price_100,
            timestamp: convert_to_seconds(600_000)
        })
    );
}

#[test]
fn twap_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(105));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    // Asset 1 uses MXN (0.057)
    // At 600_000: yield_rate = 1.0, price = 1.0 * 0.057 = 0.057
    // At 900_000: yield_rate = 1.05, price = 1.05 * 0.057 = 0.05985
    // TWAP = (0.057 + 0.05985) / 2 = 0.058425 = 5842500000000000
    let fx_price = get_fx_price_for_symbol(&env, fxs.get_unchecked(1));
    let price_100 = calculate_expected_price(normalize_price(100), fx_price, DECIMALS);
    let price_105 = calculate_expected_price(normalize_price(105), fx_price, DECIMALS);
    let expected_twap = (price_100 + price_105) / 2;
    
    let result = client.twap(&assets.get_unchecked(1), &2);

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), expected_twap);
}

#[test]
fn x_twap_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 3, 0);
    let fxs = generate_fxs(&env, 3);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    //set prices for assets
    let timestamp = 600_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(105));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    // Asset 1 uses MXN (0.057), Asset 2 uses EUR (1.10)
    // Both have yield_rate = 1.0 at 600_000 and 2.0 at 900_000
    // Cross prices are the same at both timestamps (0.057/1.10 = 0.051818...)
    // TWAP = 0.051818... = 51818181818181
    let fx_price_1 = get_fx_price_for_symbol(&env, fxs.get_unchecked(1));
    let fx_price_2 = get_fx_price_for_symbol(&env, fxs.get_unchecked(2));
    let price_1_100 = calculate_expected_price(normalize_price(100), fx_price_1, DECIMALS);
    let price_2_100 = calculate_expected_price(normalize_price(100), fx_price_2, DECIMALS);
    let x_price_100 = (price_1_100 * 10i128.pow(DECIMALS)) / price_2_100;
    // At 200 yield rate, both prices double, so cross price stays the same
    let expected_x_twap = x_price_100;
    
    let result = client.x_twap(&assets.get_unchecked(1), &assets.get_unchecked(2), &2);

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), expected_x_twap);
}

#[test]
#[should_panic]
fn x_twap_with_gap_test() {
    let (env, client, _init_data, _) = init_contract_with_admin();

    let assets = generate_assets(&env, 3, 0);
    let fxs = generate_fxs(&env, 3);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    //set prices for assets with gap
    let timestamp = 300_000;
    let updates = get_updates(&env, &assets, normalize_price(100));

    env.mock_all_auths();

    //set prices for assets
    client.set_price(&updates, &timestamp);

    let timestamp = 900_000;
    let updates = get_updates(&env, &assets, normalize_price(105));

    //set prices for assets
    client.set_price(&updates, &timestamp);

    // Asset 1 uses MXN (0.057), Asset 2 uses EUR (1.10)
    // Both have yield_rate = 1.0 at 300_000 and 2.0 at 900_000
    // Cross prices are the same at both timestamps
    // TWAP = 0.051818... = 51818181818181
    let fx_price_1 = get_fx_price_for_symbol(&env, fxs.get_unchecked(1));
    let fx_price_2 = get_fx_price_for_symbol(&env, fxs.get_unchecked(2));
    let price_1_100 = calculate_expected_price(normalize_price(100), fx_price_1, DECIMALS);
    let price_2_100 = calculate_expected_price(normalize_price(100), fx_price_2, DECIMALS);
    let x_price_100 = (price_1_100 * 10i128.pow(DECIMALS)) / price_2_100;
    let expected_x_twap = x_price_100;
    
    let result = client.x_twap(&assets.get_unchecked(1), &assets.get_unchecked(2), &3);

    assert_ne!(result, None);
    assert_eq!(result.unwrap(), expected_x_twap);
}

#[test]
fn get_non_registered_asset_price_test() {
    let (env, client, _config_data, _) = init_contract_with_admin();

    //try to get price for unknown Stellar asset
    let mut result = client.lastprice(&Asset::Stellar(Address::generate(&env)));
    assert_eq!(result, None);

    //try to get price for unknown Other asset
    result = client.lastprice(&Asset::Other(Symbol::new(&env, "NonRegisteredAsset")));
    assert_eq!(result, None);

    // Add some assets first for testing
    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    //try to get price for unknown base asset
    result = client.x_last_price(
        &Asset::Stellar(Address::generate(&env)),
        &assets.get_unchecked(1),
    );
    assert_eq!(result, None);

    //try to get price for unknown quote asset
    result = client.x_last_price(
        &assets.get_unchecked(1),
        &Asset::Stellar(Address::generate(&env)),
    );
    assert_eq!(result, None);

    //try to get price for both unknown assets
    result = client.x_last_price(
        &Asset::Stellar(Address::generate(&env)),
        &Asset::Other(Symbol::new(&env, "NonRegisteredAsset")),
    );
    assert_eq!(result, None);
}

#[test]
fn get_asset_price_for_invalid_timestamp_test() {
    let (env, client, _config_data, _) = init_contract_with_admin();

    // Add some assets first
    let assets = generate_assets(&env, 2, 0);
    let fxs = generate_fxs(&env, 2);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);

    let mut result = client.price(
        &assets.get_unchecked(1),
        &convert_to_seconds(u64::MAX),
    );
    assert_eq!(result, None);

    //try to get price for unknown asset
    result = client.lastprice(&Asset::Stellar(Address::generate(&env)));
    assert_eq!(result, None);
}

#[test]
fn authorized_test() {
    let (env, client, config_data, _) = init_contract_with_admin();

    let period: u64 = 100;
    //set prices for assets
    client
        .mock_auths(&[MockAuth {
            address: &config_data.admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "set_period",
                args: Vec::from_array(&env, [period.clone().try_into_val(&env).unwrap()]),
                sub_invokes: &[],
            },
        }])
        .set_period(&period);
}

#[test]
#[should_panic]
fn unauthorized_test() {
    let (env, client, _, _) = init_contract_with_admin();

    let account = Address::generate(&env);

    let period: u64 = 100;
    //set prices for assets
    client
        .mock_auths(&[MockAuth {
            address: &account,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "set_period",
                args: Vec::from_array(&env, [period.clone().try_into_val(&env).unwrap()]),
                sub_invokes: &[],
            },
        }])
        .set_period(&period);
}

#[test]
fn div_tests() {
    let test_cases = [
        (154467226919499, 133928752749774, 115335373284703),
        (
            i128::MAX / 100,
            231731687303715884105728,
            734216306110962248249052545,
        ),
        (231731687303715884105728, i128::MAX / 100, 13),
        // -1 expected result for errors
        (1, 0, -1),
        (0, 1, -1),
        (0, 0, -1),
        (-1, 0, -1),
        (0, -1, -1),
        (-1, -1, -1),
    ];

    for (a, b, expected) in test_cases.iter() {
        let result = panic::catch_unwind(AssertUnwindSafe(|| a.fixed_div_floor(*b, 14)));
        if expected == &-1 {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), *expected);
        }
    }
}

// Helper function to generate FX symbols
// Now that we have a mock oracle, we can use different FX symbols for testing
// Note: The contract doesn't allow duplicate FX symbols, so we need unique ones
fn generate_fxs(e: &Env, count: usize) -> Vec<Symbol> {
    let mut fxs = Vec::new(&e);
    // Use a large enough list of unique FX symbols
    let fx_names = ["USD", "MXN", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY", "INR", "BRL", "KRW", "SGD", "HKD", "NZD", "SEK", "NOK", "DKK", "PLN", "CZK", "HUF", "RUB", "TRY", "ZAR", "THB", "MYR", "PHP", "IDR", "VND", "TWD"];
    for i in 0..count {
        if i < fx_names.len() {
            fxs.push_back(Symbol::new(e, fx_names[i]));
        } else {
            // For more than available FX names, cycle through them (tests shouldn't need more)
            fxs.push_back(Symbol::new(e, fx_names[i % fx_names.len()]));
        }
    }
    fxs
}

// Helper function to initialize contract with assets and FXs
fn init_contract_with_assets_fxs<'a>(
    asset_count: usize,
) -> (Env, PriceOracleContractClient<'a>, Vec<Asset>, Vec<Symbol>) {
    let (env, client, _init_data, _) = init_contract_with_admin();
    let assets = generate_assets(&env, asset_count, 0);
    let fxs = generate_fxs(&env, asset_count);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    (env, client, assets, fxs)
}

// ========== Phase 7: Comprehensive Tests ==========

// Array Length Validation Tests
#[test]
#[should_panic]
fn test_add_assets_mismatched_lengths() {
    let (env, client, _init_data, _) = init_contract_with_admin();
    let assets = generate_assets(&env, 3, 0);
    let fxs = generate_fxs(&env, 2); // Different length
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
}

#[test]
fn test_add_assets_matching_lengths() {
    let (env, client, _init_data, _) = init_contract_with_admin();
    let assets = generate_assets(&env, 3, 0);
    let fxs = generate_fxs(&env, 3);
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    // Verify assets and fxs were added
    let stored_assets = client.assets();
    assert_eq!(stored_assets.len(), 3);
}

// Note: Testing FX mismatch in set_price is difficult because fxs are stored in the contract
// and we can't easily create a mismatch scenario. The validation in set_price checks
// that fxs.len() == updates.len() == assets.len(), which is tested via test_set_price_updates_mismatch

#[test]
#[should_panic]
fn test_set_price_updates_mismatch() {
    let (env, client, _assets, _fxs) = init_contract_with_assets_fxs(3);
    // Only 2 updates for 3 assets
    let updates = Vec::from_array(&env, [
        normalize_price(100).try_into_val(&env).unwrap(),
        normalize_price(100).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &600_000);
}

#[test]
fn test_set_price_all_lengths_match() {
    let (env, client, _assets, _fxs) = init_contract_with_assets_fxs(2);
    let updates = Vec::from_array(&env, [
        normalize_price(100).try_into_val(&env).unwrap(),
        normalize_price(100).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &600_000);
    // Should succeed without panic
}

// Yield Rate Validation Tests
#[test]
#[should_panic]
fn test_set_price_yield_rate_less_than_one() {
    let (env, client, _assets, _fxs) = init_contract_with_assets_fxs(1);
    // Yield rate less than 1.0 (10^14) - e.g., 0.5 = 5 * 10^13
    let yield_rate = 5_000_000_000_000i128; // 0.5 with 14 decimals
    let updates = Vec::from_array(&env, [yield_rate.try_into_val(&env).unwrap()]);
    
    env.mock_all_auths();
    client.set_price(&updates, &600_000);
}

#[test]
fn test_set_price_yield_rate_exactly_one() {
    let (env, client, _assets, _fxs) = init_contract_with_assets_fxs(1);
    // Yield rate exactly 1.0 (10^14)
    let yield_rate = 10i128.pow(14);
    let updates = Vec::from_array(&env, [yield_rate.try_into_val(&env).unwrap()]);
    
    env.mock_all_auths();
    client.set_price(&updates, &600_000);
    // Should succeed
}

#[test]
fn test_set_price_yield_rate_greater_than_one() {
    let (env, client, _assets, _fxs) = init_contract_with_assets_fxs(1);
    // Yield rate 1.1 (110% of base)
    let yield_rate = 110_000_000_000_000i128; // 1.1 with 14 decimals
    let updates = Vec::from_array(&env, [yield_rate.try_into_val(&env).unwrap()]);
    
    env.mock_all_auths();
    client.set_price(&updates, &600_000);
    // Should succeed
}

// USD Handling Tests
#[test]
fn test_usd_fx_with_14_decimals() {
    let (env, client, _init_data, _) = init_contract_with_admin();
    let assets = generate_assets(&env, 1, 0);
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "USD"));
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    // USD should return 10^14 with 14 decimals
    let yield_rate = 110_000_000_000_000i128; // 1.1
    let updates = Vec::from_array(&env, [yield_rate.try_into_val(&env).unwrap()]);
    
    env.mock_all_auths();
    client.set_price(&updates, &600_000);
    // Should succeed - USD handling works
}

// ========== FX Oracle Error Handling Tests ==========

// Helper to initialize contract with mock oracle in error mode
fn init_contract_with_error_mode<'a>(error_mode: &str) -> (Env, PriceOracleContractClient<'a>, Address) {
    let env = Env::default();

    //set timestamp to 900 seconds
    let ledger_info = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp: 900,
        ..ledger_info
    });

    let admin = Address::generate(&env);

    // Register mock FX oracle contract
    let mock_oracle_id = env.register(MockFxOracle, ());
    
    // Set the error mode for the oracle
    let mock_oracle_client = MockFxOracleClient::new(&env, &mock_oracle_id);
    mock_oracle_client.set_error_mode(&Some(Symbol::new(&env, error_mode)));

    let contract_id = &Address::from_string(&String::from_str(
        &env,
        "CDXHQTB7FGRMWTLJJLNI3XPKVC6SZDB5SFGZUYDPEGQQNC4G6CKE4QRC",
    ));

    env.register_at(contract_id, PriceOracleContract, ());
    let client: PriceOracleContractClient<'a> = PriceOracleContractClient::new(&env, contract_id);

    env.cost_estimate().budget().reset_unlimited();

    let init_data = ConfigData {
        admin: admin.clone(),
        period: (100 * RESOLUTION).into(),
        base_asset: Asset::Stellar(Address::generate(&env)),
        decimals: 14,
        resolution: RESOLUTION,
        fx_oracle_address: mock_oracle_id.clone(),
        max_yield_deviation_percent: 10, // 10% for most tests
    };

    env.mock_all_auths();

    //set admin and fx oracle address
    client.config(&init_data);

    (env, client, mock_oracle_id)
}

#[test]
#[should_panic]
fn test_fx_oracle_stale_price_zero_timestamp() {
    // Test that StaleFxPrice error is raised when oracle returns timestamp == 0
    let (env, client, _mock_oracle_id) = init_contract_with_error_mode("zero_timestamp");
    
    let assets = generate_assets(&env, 1, 0);
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "MXN")); // Use non-USD to trigger oracle call
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [
        normalize_price(100).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    // Should panic with StaleFxPrice error
}

#[test]
#[should_panic]
fn test_fx_oracle_stale_price_none() {
    // Test that StaleFxPrice error is raised when oracle returns None
    let (env, client, _mock_oracle_id) = init_contract_with_error_mode("none_price");
    
    let assets = generate_assets(&env, 1, 0);
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "MXN")); // Use non-USD to trigger oracle call
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [
        normalize_price(100).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    // Should panic with StaleFxPrice error
}

#[test]
#[should_panic]
fn test_fx_oracle_invalid_price_zero() {
    // Test that InvalidFxPrice error is raised when oracle returns price == 0
    let (env, client, _mock_oracle_id) = init_contract_with_error_mode("zero_price");
    
    let assets = generate_assets(&env, 1, 0);
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "MXN")); // Use non-USD to trigger oracle call
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [
        normalize_price(100).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    // Should panic with InvalidFxPrice error
}

#[test]
#[should_panic]
fn test_fx_oracle_invalid_price_negative() {
    // Test that InvalidFxPrice error is raised when oracle returns negative price
    let (env, client, _mock_oracle_id) = init_contract_with_error_mode("negative_price");
    
    let assets = generate_assets(&env, 1, 0);
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "MXN")); // Use non-USD to trigger oracle call
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [
        normalize_price(100).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    // Should panic with InvalidFxPrice error
}

#[test]
#[should_panic]
fn test_integer_overflow_in_price_calculation() {
    // Test that IntegerOverflow error is raised when price calculation overflows
    let (env, client, _init_data, _) = init_contract_with_admin();
    
    let assets = generate_assets(&env, 1, 0);
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "USD"));
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    // Use a yield rate and FX price that will cause overflow when multiplied
    // i128::MAX is 2^127 - 1, so we need yield_rate * fx_price > i128::MAX
    // For USD, fx_price = 10^14, so yield_rate needs to be > i128::MAX / 10^14
    // Let's use a very large yield rate that will cause overflow
    let huge_yield_rate = i128::MAX / 2; // This will overflow when multiplied by 10^14
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [
        huge_yield_rate.try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    // Should panic with IntegerOverflow error
}

// ========== FX Oracle Timestamp Drift Tests ==========

#[test]
fn test_fx_oracle_timestamp_drift_within_limit() {
    // Test that FX oracle timestamp within 2 resolutions succeeds
    let (env, client, _init_data, mock_oracle_id) = init_contract_with_admin();
    
    let assets = generate_assets(&env, 1, 0);
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "MXN"));
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    // Set an initial price to establish a contract timestamp
    let initial_timestamp = 600_000; // 600 seconds in milliseconds
    let updates = Vec::from_array(&env, [
        normalize_price(100).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &initial_timestamp);
    
    // Set oracle timestamp to be within 2 resolutions (within 600 seconds = 1 resolution)
    // Resolution is 300 seconds, so 2 resolutions = 600 seconds = 600_000 ms
    // Set oracle to be 300 seconds (1 resolution) ahead: 600 + 300 = 900 seconds
    let mock_oracle_client = MockFxOracleClient::new(&env, &mock_oracle_id);
    mock_oracle_client.set_custom_timestamp(&900); // 900 seconds
    
    // Try to set price again - should succeed as drift is within 2 resolutions
    let new_timestamp = 900_000; // 900 seconds in milliseconds
    let updates2 = Vec::from_array(&env, [
        normalize_price(110).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates2, &new_timestamp);
    // Should succeed
}

#[test]
#[should_panic]
fn test_fx_oracle_timestamp_drift_too_far_ahead() {
    // Test that FX oracle timestamp more than 2 resolutions ahead fails
    let (env, client, _init_data, mock_oracle_id) = init_contract_with_admin();
    
    let assets = generate_assets(&env, 1, 0);
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "MXN"));
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    // Set an initial price to establish a contract timestamp
    let initial_timestamp = 600_000; // 600 seconds in milliseconds
    let updates = Vec::from_array(&env, [
        normalize_price(100).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &initial_timestamp);
    
    // Set oracle timestamp to be more than 2 resolutions ahead
    // Resolution is 300 seconds, so 2 resolutions = 600 seconds
    // Set oracle to be 601 seconds ahead: 600 + 601 = 1201 seconds
    let mock_oracle_client = MockFxOracleClient::new(&env, &mock_oracle_id);
    mock_oracle_client.set_custom_timestamp(&1201); // 1201 seconds (more than 2 resolutions ahead)
    
    // Try to set price again - should fail with timestamp drift error
    let new_timestamp = 1201_000; // 1201 seconds in milliseconds
    let updates2 = Vec::from_array(&env, [
        normalize_price(110).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates2, &new_timestamp);
    // Should panic with FxOracleTimestampDrift error
}

#[test]
#[should_panic]
fn test_fx_oracle_timestamp_drift_too_far_behind() {
    // Test that FX oracle timestamp more than 2 resolutions behind fails
    let (env, client, _init_data, mock_oracle_id) = init_contract_with_admin();
    
    let assets = generate_assets(&env, 1, 0);
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "MXN"));
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    // Set an initial price to establish a contract timestamp at a later time
    // This allows us to test the "behind" scenario
    let initial_timestamp = 1200_000; // 1200 seconds in milliseconds
    let updates = Vec::from_array(&env, [
        normalize_price(100).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &initial_timestamp);
    
    // Set oracle timestamp to be more than 2 resolutions behind
    // Resolution is 300 seconds, so 2 resolutions = 600 seconds
    // Contract timestamp is 1200 seconds, so oracle at 599 seconds is 601 seconds behind (more than 2 resolutions)
    let mock_oracle_client = MockFxOracleClient::new(&env, &mock_oracle_id);
    mock_oracle_client.set_custom_timestamp(&599); // 599 seconds (601 seconds behind, more than 2 resolutions)
    
    // Try to set price again - should fail with timestamp drift error
    let new_timestamp = 1500_000; // 1500 seconds in milliseconds
    let updates2 = Vec::from_array(&env, [
        normalize_price(110).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates2, &new_timestamp);
    // Should panic with FxOracleTimestampDrift error
}

#[test]
fn test_fx_oracle_timestamp_drift_skipped_on_first_update() {
    // Test that timestamp drift check is skipped when contract has no last_timestamp (first update)
    let (env, client, _init_data, mock_oracle_id) = init_contract_with_admin();
    
    let assets = generate_assets(&env, 1, 0);
    let mut fxs = Vec::new(&env);
    fxs.push_back(Symbol::new(&env, "MXN"));
    
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    // Set oracle timestamp to any value (even far from current time)
    // Since contract has no last_timestamp yet, drift check should be skipped
    let mock_oracle_client = MockFxOracleClient::new(&env, &mock_oracle_id);
    mock_oracle_client.set_custom_timestamp(&10000); // Far in the future
    
    // First price update - should succeed even with large oracle timestamp difference
    let timestamp = 600_000; // 600 seconds in milliseconds
    let updates = Vec::from_array(&env, [
        normalize_price(100).try_into_val(&env).unwrap(),
    ]);
    
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    // Should succeed - drift check skipped on first update
}

// ========== Yield Rate Validation Tests ==========

#[test]
fn test_yield_rate_monotonic_increase() {
    // Verify that increasing yield rates within deviation work correctly
    let (env, client, assets, _fxs) = init_contract_with_assets_fxs(1);
    let asset = assets.get_unchecked(0);
    
    // First update with yield rate 1.05 (105% with 14 decimals = 105000000000000)
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [105000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    
    // Second update with yield rate 1.10 (5% increase, within 10% deviation)
    let timestamp2 = 900_000;
    let updates2 = Vec::from_array(&env, [110000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates2, &timestamp2);
    
    // Verify the prices were set correctly
    let price1 = client.price(&asset, &convert_to_seconds(timestamp));
    let price2 = client.price(&asset, &convert_to_seconds(timestamp2));
    assert!(price1.is_some());
    assert!(price2.is_some());
    assert!(price2.unwrap().price > price1.unwrap().price); // Second price should be higher
}

#[test]
fn test_yield_rate_same_value() {
    // Verify that setting the same yield rate twice is allowed
    let (env, client, assets, _fxs) = init_contract_with_assets_fxs(1);
    let asset = assets.get_unchecked(0);
    
    // First update with yield rate 1.05
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [105000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    
    // Second update with same yield rate 1.05
    let timestamp2 = 900_000;
    let updates2 = Vec::from_array(&env, [105000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates2, &timestamp2);
    
    // Verify both prices were set
    let price1 = client.price(&asset, &convert_to_seconds(timestamp));
    let price2 = client.price(&asset, &convert_to_seconds(timestamp2));
    assert!(price1.is_some());
    assert!(price2.is_some());
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")] // YieldRateDecreased = 17
fn test_yield_rate_decrease_rejected() {
    // Verify that decreasing yield rates panic with YieldRateDecreased error
    let (env, client, _assets, _fxs) = init_contract_with_assets_fxs(1);
    
    // First update with yield rate 1.10
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [110000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    
    // Second update with lower yield rate 1.05 - should panic
    let timestamp2 = 900_000;
    let updates2 = Vec::from_array(&env, [105000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates2, &timestamp2);
}

#[test]
#[should_panic(expected = "Error(Contract, #18)")] // YieldRateDeviationExceeded = 18
fn test_yield_rate_deviation_exceeded() {
    // Verify that large increases panic with YieldRateDeviationExceeded error
    let (env, client, _assets, _fxs) = init_contract_with_assets_fxs(1);
    
    // First update with yield rate 1.05
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [105000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    
    // Second update with yield rate 1.17 (11.4% increase, exceeds 10% max deviation)
    let timestamp2 = 900_000;
    let updates2 = Vec::from_array(&env, [117000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates2, &timestamp2);
}

#[test]
fn test_yield_rate_deviation_at_boundary() {
    // Verify that increases exactly at the max deviation are allowed
    let (env, client, assets, _fxs) = init_contract_with_assets_fxs(1);
    let asset = assets.get_unchecked(0);
    
    // First update with yield rate 1.00 (100000000000000)
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [100000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    
    // Second update with yield rate 1.10 (exactly 10% increase)
    let timestamp2 = 900_000;
    let updates2 = Vec::from_array(&env, [110000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates2, &timestamp2);
    
    // Verify both prices were set
    let price1 = client.price(&asset, &convert_to_seconds(timestamp));
    let price2 = client.price(&asset, &convert_to_seconds(timestamp2));
    assert!(price1.is_some());
    assert!(price2.is_some());
}

#[test]
fn test_yield_rate_first_update_any_value() {
    // Verify that the first update for an asset accepts any valid yield rate
    let (env, client, assets, _fxs) = init_contract_with_assets_fxs(1);
    let asset = assets.get_unchecked(0);
    
    // First update with a high yield rate (1.50)
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [150000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    
    // Verify the price was set
    let price = client.price(&asset, &convert_to_seconds(timestamp));
    assert!(price.is_some());
}

#[test]
fn test_yield_rate_per_asset_independence() {
    // Verify that each asset's yield rate is tracked independently
    let (env, client, assets, _fxs) = init_contract_with_assets_fxs(2);
    
    // First update for both assets with different yield rates
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [
        105000000000000i128, // Asset 0: 1.05
        110000000000000i128, // Asset 1: 1.10
    ]);
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    
    // Second update: Asset 0 increases by 5%, Asset 1 stays the same
    let timestamp2 = 900_000;
    let updates2 = Vec::from_array(&env, [
        110250000000000i128, // Asset 0: 1.1025 (5% increase from 1.05)
        110000000000000i128, // Asset 1: 1.10 (same)
    ]);
    env.mock_all_auths();
    client.set_price(&updates2, &timestamp2);
    
    // Verify both assets have their prices set correctly
    let asset0 = assets.get_unchecked(0);
    let asset1 = assets.get_unchecked(1);
    
    let price0 = client.price(&asset0, &convert_to_seconds(timestamp2));
    let price1 = client.price(&asset1, &convert_to_seconds(timestamp2));
    assert!(price0.is_some());
    assert!(price1.is_some());
}

#[test]
fn test_yield_rate_with_different_max_deviations() {
    // Test with different configured max deviation values
    let env = Env::default();
    
    // Set timestamp to 900 seconds
    let ledger_info = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp: 900,
        ..ledger_info
    });
    
    let admin = Address::generate(&env);
    
    // Register mock FX oracle contract
    let mock_oracle_id = env.register(MockFxOracle, ());
    let mock_oracle_client = MockFxOracleClient::new(&env, &mock_oracle_id);
    mock_oracle_client.set_error_mode(&None);
    
    let contract_id = &Address::from_string(&String::from_str(
        &env,
        "CDXHQTB7FGRMWTLJJLNI3XPKVC6SZDB5SFGZUYDPEGQQNC4G6CKE4QRC",
    ));
    
    env.register_at(contract_id, PriceOracleContract, ());
    let client: PriceOracleContractClient = PriceOracleContractClient::new(&env, contract_id);
    
    env.cost_estimate().budget().reset_unlimited();
    
    // Initialize with 5% max deviation instead of 10%
    let init_data = ConfigData {
        admin: admin.clone(),
        period: (100 * RESOLUTION).into(),
        base_asset: Asset::Stellar(Address::generate(&env)),
        decimals: 14,
        resolution: RESOLUTION,
        fx_oracle_address: mock_oracle_id.clone(),
        max_yield_deviation_percent: 5, // 5% max deviation
    };
    
    env.mock_all_auths();
    client.config(&init_data);
    
    // Add an asset
    let assets = generate_assets(&env, 1, 0);
    let fxs = generate_fxs(&env, 1);
    env.mock_all_auths();
    client.add_assets(&assets, &fxs);
    
    // First update with yield rate 1.00
    let timestamp = 600_000;
    let updates = Vec::from_array(&env, [100000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates, &timestamp);
    
    // Second update with 5% increase should succeed
    let timestamp2 = 900_000;
    let updates2 = Vec::from_array(&env, [105000000000000i128]);
    env.mock_all_auths();
    client.set_price(&updates2, &timestamp2);
    
    let asset = assets.get_unchecked(0);
    let price = client.price(&asset, &convert_to_seconds(timestamp2));
    assert!(price.is_some());
}
