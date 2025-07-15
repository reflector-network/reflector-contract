#![allow(non_upper_case_globals)]
use soroban_sdk::storage::{Instance, Temporary};
use soroban_sdk::{panic_with_error, Address, Env, Vec};

use crate::extensions::u128_helper::U128Helper;
use crate::types;

use types::{asset::Asset, error::Error};
const ADMIN_KEY: &str = "admin";
const LAST_TIMESTAMP: &str = "last_timestamp";
const RETENTION_PERIOD: &str = "period";
const ASSETS: &str = "assets";
const BASE_ASSET: &str = "base_asset";
const DECIMALS: &str = "decimals";
const RESOLUTION: &str = "resolution";
const EXPIRATION: &str = "expiration";
const RETENTION: &str = "retention";
const CACHE: &str = "cache";
const CACHE_SIZE: &str = "cache_size";

const V2_UPDATE_TS: &str = "v2_update_ts";

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

    fn set_price(&self, asset: u8, price: i128, timestamp: u64, ledgers: u32);

    fn get_prices(&self, timestamp: u64) -> Option<Vec<i128>>;

    fn set_prices(&self, prices: &Vec<i128>, timestamp: u64, ledgers: u32);

    fn get_cache(&self) -> Option<Vec<(u64, Vec<i128>)>>;

    fn set_cache(&self, prices: Vec<(u64,Vec<i128>)>);

    fn get_cache_size(&self) -> u32;

    fn set_cache_size(&self, cache_size: u32);

    fn get_last_timestamp(&self) -> u64;

    fn set_last_timestamp(&self, timestamp: u64);

    fn get_assets(&self) -> Vec<Asset>;

    fn set_assets(&self, assets: Vec<Asset>);

    fn set_asset_index(&self, asset: &Asset, index: u32);

    fn get_asset_index(&self, asset: &Asset) -> Option<u32>;

    fn set_expiration(&self, assets: &Vec<u64>);

    fn get_expiration(&self) -> Vec<u64>;

    fn set_retention_config(&self, fee_data: (Address, i128));

    fn get_retention_config(&self) -> Option<(Address, i128)>;

    fn panic_if_not_admin(&self);

    fn is_initialized(&self) -> bool;

    fn get_v2_update_ts(&self) -> u64;

    fn set_v2_update_ts(&self, timestamp: u64);
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
        let data_key = U128Helper::encode_price_record_key(timestamp, asset);
        //get the price
        get_temporary_storage(self).get(&data_key)
    }

    fn set_price(&self, asset: u8, price: i128, timestamp: u64, ledgers_to_live: u32) {
        //build the key for the price
        let data_key = U128Helper::encode_price_record_key(timestamp, asset);

        //set the price
        let temps_storage = get_temporary_storage(&self);
        temps_storage.set(&data_key, &price);
        if ledgers_to_live > 16 {
            //16 is the minimum number
            temps_storage.extend_ttl(&data_key, ledgers_to_live, ledgers_to_live)
        }
    }

    fn get_prices(&self, timestamp: u64) -> Option<Vec<i128>> {
        //check if the timestamp is in the cache
        let cache = self.get_cache();
        if cache.is_some() {
            //check the cache first
            for (ts, prices) in cache.unwrap() {
                if ts == timestamp {
                    return Some(prices);
                }
            }
        }
        //get the price from the temporary storage
        get_temporary_storage(self).get(&timestamp)
    }

    fn set_prices(&self, prices: &Vec<i128>, timestamp: u64, ledgers: u32) {
        //set the price
        let temps_storage = get_temporary_storage(&self);
        temps_storage.set(&timestamp, prices);
        if ledgers > 16 {
            //16 is the minimum number
            temps_storage.extend_ttl(&timestamp, ledgers, ledgers)
        }
    }

    fn get_cache(&self) -> Option<Vec<(u64, Vec<i128>)>> {
        get_instance_storage(self).get(&CACHE)
    }

    fn set_cache(&self, prices: Vec<(u64,Vec<i128>)>) {
        get_instance_storage(&self).set(&CACHE, &prices);
    }

    fn get_cache_size(&self) -> u32 {
        get_instance_storage(self).get(&CACHE_SIZE).unwrap_or(0)
    }

    fn set_cache_size(&self, cache_size: u32) {
        get_instance_storage(&self).set(&CACHE_SIZE, &cache_size);
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

    fn get_asset_index(&self, asset: &Asset) -> Option<u32> {
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
        Some(index.unwrap())
    }

    fn set_expiration(&self, expiration: &Vec<u64>) {
        get_instance_storage(self).set(&EXPIRATION, expiration)
    }

    fn get_expiration(&self) -> Vec<u64> {
        get_instance_storage(self)
            .get(&EXPIRATION)
            .unwrap_or_else(|| Vec::new(self))
    }

    fn set_retention_config(&self, retention_config: (Address, i128)) {
        get_instance_storage(self).set(&RETENTION, &retention_config);
    }

    fn get_retention_config(&self) -> Option<(Address, i128)> {
        get_instance_storage(self).get(&RETENTION)
    }

    fn panic_if_not_admin(&self) {
        let admin = self.get_admin();
        if admin.is_none() {
            panic_with_error!(self, Error::Unauthorized);
        }
        admin.unwrap().require_auth()
    }

    fn get_v2_update_ts(&self) -> u64 {
        get_instance_storage(self).get(&V2_UPDATE_TS).unwrap_or(0)
    }

    fn set_v2_update_ts(&self, timestamp: u64) {
        get_instance_storage(self).set(&V2_UPDATE_TS, &timestamp);
    }
}

fn get_instance_storage(e: &Env) -> Instance {
    e.storage().instance()
}

fn get_temporary_storage(e: &Env) -> Temporary {
    e.storage().temporary()
}
