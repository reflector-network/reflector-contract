#![allow(non_upper_case_globals)]
use soroban_sdk::storage::{Instance, Temporary};
use soroban_sdk::{Address, Env, Symbol, Vec, panic_with_error};

use crate::{PriceOracleContractClient, extensions};
use crate::types;

use extensions::u128_helper::U128Helper;
use types::{asset::Asset, error::Error};
const ADMIN_KEY: &str = "admin";
const LAST_TIMESTAMP: &str = "last_timestamp";
const RETENTION_PERIOD: &str = "period";
const ASSETS: &str = "assets";
const BASE_ASSET: &str = "base_asset";
const DECIMALS: &str = "decimals";
const RESOLUTION: &str = "resolution";
const FXS: &str = "fxs";
const FX_ORACLE_ADDRESS: &str = "fx_oracle_address";
const MAX_YIELD_DEVIATION: &str = "max_yield_deviation";

pub trait EnvExtensions {
    fn get_admin(&self) -> Option<Address>;

    fn set_admin(&self, admin: &Address);

    fn get_base_asset(&self) -> Asset;

    fn set_base_asset(&self, base_asset: &Asset);

    fn get_decimals(&self) -> u32;

    fn set_decimals(&self, decimals: u32);

    fn get_resolution(&self) -> u32;

    fn set_resolution(&self, resolution: u32);

    fn get_retention_period(&self) -> u64;

    fn set_retention_period(&self, period: u64);

    fn get_price(&self, asset: u8, timestamp: u64) -> Option<i128>;

    fn set_price(&self, asset: u8, fx: Symbol, price: i128, timestamp: u64, ledgers: u32);

    fn get_last_timestamp(&self) -> u64;

    fn set_last_timestamp(&self, timestamp: u64);

    fn get_assets(&self) -> Vec<Asset>;

    fn set_assets(&self, assets: Vec<Asset>);

    fn get_fxs(&self) -> Vec<Symbol>;

    fn set_fxs(&self, fxs: Vec<Symbol>);

    fn set_asset_index(&self, asset: &Asset, index: u32);

    fn get_asset_index(&self, asset: &Asset) -> Option<u8>;

    fn get_fx_index(&self, fx: &Symbol) -> Option<u8>;

    fn set_fx_index(&self, fx: &Symbol, index: u32);

    fn panic_if_not_admin(&self);

    fn is_initialized(&self) -> bool;

    fn get_fx_oracle_address(&self) -> Option<Address>;

    fn set_fx_oracle_address(&self, address: &Address);

    fn get_max_yield_deviation(&self) -> u32;

    fn set_max_yield_deviation(&self, percent: u32);

    fn get_last_yield_rate(&self, asset: u8, timestamp: u64) -> Option<i128>;

    fn set_last_yield_rate(&self, asset: u8, timestamp: u64, yield_rate: i128, ledgers: u32);
}

impl EnvExtensions for Env {
    fn is_initialized(&self) -> bool {
        get_instance_storage(&self).has(&ADMIN_KEY)
    }

    fn get_admin(&self) -> Option<Address> {
        get_instance_storage(&self).get(&ADMIN_KEY)
    }

    fn set_admin(&self, admin: &Address) {
        get_instance_storage(&self).set(&ADMIN_KEY, admin);
    }

    fn set_base_asset(&self, base_asset: &Asset) {
        get_instance_storage(&self).set(&BASE_ASSET, base_asset)
    }

    fn get_base_asset(&self) -> Asset {
        get_instance_storage(self).get(&BASE_ASSET).unwrap()
    }

    fn get_decimals(&self) -> u32 {
        get_instance_storage(self).get(&DECIMALS).unwrap()
    }

    fn set_decimals(&self, decimals: u32) {
        get_instance_storage(&self).set(&DECIMALS, &decimals)
    }

    fn get_resolution(&self) -> u32 {
        get_instance_storage(self).get(&RESOLUTION).unwrap()
    }

    fn set_resolution(&self, resolution: u32) {
        get_instance_storage(&self).set(&RESOLUTION, &resolution)
    }

    fn get_retention_period(&self) -> u64 {
        get_instance_storage(&self)
            .get(&RETENTION_PERIOD)
            .unwrap_or_default()
    }

    fn set_retention_period(&self, rdm_period: u64) {
        get_instance_storage(&self).set(&RETENTION_PERIOD, &rdm_period);
    }

    fn get_price(&self, asset: u8, timestamp: u64) -> Option<i128> {
        //build the key for the price
        let data_key = U128Helper::encode_record_key(timestamp, asset);
        //get the price
        get_temporary_storage(self).get(&data_key)
    }

    fn set_price(&self, asset: u8, fx: Symbol, yield_rate: i128, timestamp: u64, ledgers_to_live: u32) {
        //validate yield_rate >= 1.0 (with matching decimals)
        let decimals = self.get_decimals();
        let min_yield_rate = 10i128.pow(decimals);
        if yield_rate < min_yield_rate {
            panic_with_error!(self, Error::InvalidYieldRate);
        }

        // Retrieve the last yield rate for this asset from the previous timestamp (if it exists)
        let last_timestamp = self.get_last_timestamp();
        let previous_yield_rate = if last_timestamp > 0 {
            self.get_last_yield_rate(asset, last_timestamp)
        } else {
            None
        };
        
        if let Some(prev_rate) = previous_yield_rate {
            // Monotonic check: new yield rate must be >= previous yield rate
            if yield_rate < prev_rate {
                panic_with_error!(self, Error::YieldRateDecreased);
            }
            
            // Deviation check: calculate absolute percentage change
            // Formula: (new - old) / old * 100
            let change = yield_rate - prev_rate;
            
            // Use checked operations to prevent overflow
            let change_times_100 = match change.checked_mul(100) {
                Some(val) => val,
                None => panic_with_error!(self, Error::IntegerOverflow),
            };
            
            let percentage_change = match change_times_100.checked_div(prev_rate) {
                Some(val) => val,
                None => panic_with_error!(self, Error::IntegerOverflow),
            };
            
            let max_deviation = self.get_max_yield_deviation() as i128;
            if percentage_change > max_deviation {
                panic_with_error!(self, Error::YieldRateDeviationExceeded);
            }
        }
        
        // Store the new yield rate for future comparisons
        self.set_last_yield_rate(asset, timestamp, yield_rate, ledgers_to_live);

        //build the key for the price
        let data_key = U128Helper::encode_record_key(timestamp, asset);
        let fx_price = get_reflector_fx_price(self, fx);
        let price = get_price_with_yield(self, yield_rate, fx_price, decimals);

        //set the price
        let temps_storage = get_temporary_storage(&self);
        temps_storage.set(&data_key, &price);
        if ledgers_to_live > 16 {
            //16 is the minimum number
            temps_storage.extend_ttl(&data_key, ledgers_to_live, ledgers_to_live)
        }
    }

    fn get_last_timestamp(&self) -> u64 {
        //get the marker
        get_instance_storage(&self)
            .get(&LAST_TIMESTAMP)
            .unwrap_or_default()
    }

    fn set_last_timestamp(&self, timestamp: u64) {
        get_instance_storage(&self).set(&LAST_TIMESTAMP, &timestamp);
    }

    fn get_assets(&self) -> Vec<Asset> {
        get_instance_storage(&self)
            .get(&ASSETS)
            .unwrap_or_else(|| Vec::new(&self))
    }
    
    fn set_assets(&self, assets: Vec<Asset>) {
        get_instance_storage(&self).set(&ASSETS, &assets);
    }

    fn get_fxs(&self) -> Vec<Symbol> {
        get_instance_storage(&self)
            .get(&FXS)
            .unwrap_or_else(|| Vec::new(&self))
    }

    fn set_fxs(&self, fxs: Vec<Symbol>) {
        get_instance_storage(&self).set(&FXS, &fxs);
    }

    fn set_asset_index(&self, asset: &Asset, index: u32) {
        match asset {
            Asset::Stellar(address) => {
                get_instance_storage(&self).set(&address, &index);
            }
            Asset::Other(symbol) => {
                get_instance_storage(&self).set(&symbol, &index);
            }
        }
    }

    fn get_asset_index(&self, asset: &Asset) -> Option<u8> {
        let index: Option<u32>;
        match asset {
            Asset::Stellar(address) => {
                index = get_instance_storage(self).get(&address);
            }
            Asset::Other(symbol) => {
                index = get_instance_storage(self).get(&symbol);
            }
        }
        if index.is_none() {
            return None;
        }
        return Some(index.unwrap() as u8);
    }

    fn get_fx_index(&self, fx: &Symbol) -> Option<u8> {
        let index: Option<u32> = get_instance_storage(self).get(&fx);
        if index.is_none() {
            return None;
        }
        return Some(index.unwrap() as u8);
    }

    fn set_fx_index(&self, fx: &Symbol, index: u32) {
        get_instance_storage(self).set(&fx, &index);
    }

    fn panic_if_not_admin(&self) {
        let admin = self.get_admin();
        if admin.is_none() {
            panic_with_error!(self, Error::Unauthorized);
        }
        admin.unwrap().require_auth()
    }

    fn get_fx_oracle_address(&self) -> Option<Address> {
        get_instance_storage(self).get(&FX_ORACLE_ADDRESS)
    }

    fn set_fx_oracle_address(&self, address: &Address) {
        get_instance_storage(self).set(&FX_ORACLE_ADDRESS, address);
    }

    fn get_max_yield_deviation(&self) -> u32 {
        get_instance_storage(self).get(&MAX_YIELD_DEVIATION).unwrap_or(0)
    }

    fn set_max_yield_deviation(&self, percent: u32) {
        get_instance_storage(self).set(&MAX_YIELD_DEVIATION, &percent);
    }

    fn get_last_yield_rate(&self, asset: u8, timestamp: u64) -> Option<i128> {
        // Store yield rate per asset and timestamp, similar to price data
        // Use the same key encoding as price records but with high bit set to distinguish
        let data_key = U128Helper::encode_record_key(timestamp, asset) | (1u128 << 127);
        get_temporary_storage(self).get(&data_key)
    }

    fn set_last_yield_rate(&self, asset: u8, timestamp: u64, yield_rate: i128, ledgers: u32) {
        // Store yield rate per asset and timestamp, similar to price data
        // Use the same key encoding as price records but with high bit set to distinguish
        let data_key = U128Helper::encode_record_key(timestamp, asset) | (1u128 << 127);
        let temps_storage = get_temporary_storage(self);
        temps_storage.set(&data_key, &yield_rate);
        if ledgers > 16 {
            temps_storage.extend_ttl(&data_key, ledgers, ledgers);
        }
    }
}

fn get_instance_storage(e: &Env) -> Instance {
    e.storage().instance()
}

fn get_temporary_storage(e: &Env) -> Temporary {
    e.storage().temporary()
}

// The yield rate is sent as a 14 decimal place number, such as 110987898736637 (for 1.10987898736637%)
// To get the price with yield, we need to multiply the fx rate of the fiat by this yield percent,
// and then divide by 10^14 to get the price with yield.
fn get_price_with_yield(e: &Env, yield_rate: i128, fx_price: i128, decimals: u32) -> i128 {
    // Use checked multiplication to prevent overflow
    let intermediate = match fx_price.checked_mul(yield_rate) {
        Some(val) => val,
        None => panic_with_error!(e, Error::IntegerOverflow),
    };
    
    // Use checked division to handle edge cases
    let divisor = 10i128.pow(decimals);
    match intermediate.checked_div(divisor) {
        Some(val) => val,
        None => panic_with_error!(e, Error::IntegerOverflow),
    }
}

fn get_reflector_fx_price(e: &Env, fx: Symbol) -> i128 {
    if fx == Symbol::new(e, "USD") {
        return 10i128.pow(e.get_decimals()); // 1 USD with matching decimals
    }
    let reflector_client = get_reflector_oracle(e);
    let ticker = Asset::Other(fx);
    
    // Get the last price from the oracle (single call instead of last_timestamp + price)
    let price_data = reflector_client.lastprice(&ticker);
    if price_data.is_none() {
        panic_with_error!(&e, Error::StaleFxPrice);
    }
    
    let price_data = price_data.unwrap();
    
    // Check timestamp drift: oracle timestamp should be within 2 resolutions of contract's last timestamp
    let contract_last_timestamp = e.get_last_timestamp(); // in milliseconds
    if contract_last_timestamp > 0 {
        // Convert oracle timestamp from seconds to milliseconds
        let oracle_timestamp_ms = price_data.timestamp * 1000;
        let resolution_ms = e.get_resolution() as u64; // resolution is in milliseconds
        let max_drift = 2 * resolution_ms;
        
        // Calculate absolute difference
        let drift = oracle_timestamp_ms.abs_diff(contract_last_timestamp);
        
        if drift > max_drift {
            panic_with_error!(&e, Error::FxOracleTimestampDrift);
        }
    }
    
    // Validate the price
    let fx_price = price_data.price;
    if fx_price <= 0 {
        panic_with_error!(&e, Error::InvalidFxPrice);
    }
    fx_price
}

fn get_reflector_oracle(e: &Env) -> PriceOracleContractClient {
    // Get the FX oracle address from storage (set during config)
    let oracle_address = e.get_fx_oracle_address()
        .unwrap_or_else(|| panic_with_error!(e, Error::FxOracleUnavailable));
    PriceOracleContractClient::new(&e, &oracle_address)
}
