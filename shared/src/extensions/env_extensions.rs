#![allow(non_upper_case_globals)]
use soroban_sdk::{panic_with_error, Address, BytesN, Env, Symbol, Vec};

use crate::constants;
use crate::extensions;
use crate::types;

use constants::Constants;
use extensions::i128_extensions::I128Extensions;
use extensions::u128_helper::U128Helper;
use types::{
    asset::Asset, asset_type::AssetType,
    error::Error, price_data::PriceData,
};
const ADMIN_KEY: &str = "admin";
const CONFIG_VERSION: &str = "config_version";
const LAST_TIMESTAMP: &str = "last_timestamp";
const RETENTION_PERIOD:&str = "period";
const ASSETS: &str = "assets";

pub trait EnvExtensions {
    fn is_authorized(&self, invoker: &Address) -> bool;

    fn get_config_version(&self) -> u32;

    fn set_config_version(&self, version: u32);

    fn get_admin(&self) -> Address;

    fn set_admin(&self, admin: &Address);

    fn get_price(&self, asset: u8, timestamp: u64) -> Option<i128>;

    fn set_price(&self, asset: u8, price: i128, timestamp: u64);

    fn get_last_timestamp(&self) -> u64;

    fn set_last_timestamp(&self, timestamp: u64);

    fn get_retention_period(&self) -> Option<u64>;

    fn set_retention_period(&self, period: u64);

    fn get_assets(&self) -> Vec<Asset>;

    fn set_assets(&self, assets: Vec<Asset>);

    fn set_asset_index(&self, asset: Asset, index: u32);

    fn get_asset_index(&self, asset: Asset) -> Option<u8>;

    fn get_prices(&self, asset: u8, records: u32) -> Option<Vec<PriceData>>;

    fn get_x_price(&self, base_asset: u8, quote_asset: u8, timestamp: u64) -> Option<i128>;

    fn get_x_prices(
        &self,
        base_asset: u8,
        quote_asset: u8,
        records: u32,
    ) -> Option<Vec<PriceData>>;

    fn invoker(&self) -> Option<Address>;

    fn try_delete_old_price(&self, asset: u8, timestamp: u64, period: u64) -> bool;

    fn panic_if_not_admin(&self, invoker: &Address);

    fn panic_if_version_invalid(&self, version: u32);

    fn get_base_asset(&self) -> Asset;

    fn is_initialized(&self) -> bool;
}

impl EnvExtensions for Env {
    fn is_authorized(&self, invoker: &Address) -> bool {
        invoker.require_auth();

        //invoke get_admin to check if the admin is set
        let admin = self.get_admin();
        invoker == &admin
    }

    fn is_initialized(&self) -> bool {
        self.storage().persistent().has(&ADMIN_KEY)
    }

    fn get_admin(&self) -> Address {
        //TODO: add getting default admin from constants, when convertion from string to address is implemented in soroban-sdk
        self.storage().persistent().get(&ADMIN_KEY).unwrap()
    }

    fn set_admin(&self, admin: &Address) {
        self.storage().persistent().set(&ADMIN_KEY, admin);
    }

    fn get_config_version(&self) -> u32 {
        if !self.storage().persistent().has(&CONFIG_VERSION) {
            return 0;
        }
        self.storage().persistent().get(&CONFIG_VERSION).unwrap()
    }

    fn set_config_version(&self, version: u32) {
        self.storage().persistent().set(&CONFIG_VERSION, &version);
    }

    fn get_price(&self, asset: u8, timestamp: u64) -> Option<i128> {
        //build the key for the price
        let data_key = U128Helper::encode_to_u128(timestamp, asset);

        //check if the price is available
        if !self.storage().persistent().has(&data_key) {
            return None;
        }

        //get the price
        Some(self.storage().persistent().get(&data_key).unwrap())
    }

    fn set_price(&self, asset: u8, price: i128, timestamp: u64) {
        //build the key for the price
        let data_key = U128Helper::encode_to_u128(timestamp, asset);

        //set the price
        self.storage().persistent().set(&data_key, &price);
    }

    fn get_last_timestamp(&self) -> u64 {
        //check if the marker is available
        if !self.storage().persistent().has(&LAST_TIMESTAMP) {
            return 0;
        }

        //get the marker
        self.storage().persistent().get(&LAST_TIMESTAMP).unwrap()
    }

    fn set_last_timestamp(&self, timestamp: u64) {
        self.storage().persistent().set(&LAST_TIMESTAMP, &timestamp);
    }

    fn get_retention_period(&self) -> Option<u64> {
        if !self.storage().persistent().has(&RETENTION_PERIOD) {
            return None;
        }
        Some(self.storage().persistent().get(&RETENTION_PERIOD).unwrap())
    }

    fn set_retention_period(&self, rdm_period: u64) {
        self.storage()
            .persistent()
            .set(&RETENTION_PERIOD, &rdm_period);
    }

    fn get_assets(&self) -> Vec<Asset> {
        if !self.storage().persistent().has(&ASSETS) {
            //return empty vector
            return Vec::new(&self);
        }
        self.storage().persistent().get(&ASSETS).unwrap()
    }

    fn set_assets(&self, assets: Vec<Asset>) {
        self.storage().persistent().set(&ASSETS, &assets);
    }

    fn set_asset_index(&self, asset: Asset, index: u32) {
        match  asset {
            Asset::S(address) => {
                self.storage().persistent().set(&address, &index);
            },
            Asset::G(symbol) => {
                self.storage().persistent().set(&symbol, &index);
            }
        }
    }

    fn get_asset_index(&self, asset: Asset) -> Option<u8> {
        match asset {
            Asset::S(address) => {
                if !self.storage().persistent().has(&address) {
                    return None;
                }
                let index: u32 = self.storage().persistent().get(&address).unwrap();
                return Some(index as u8);
            },
            Asset::G(symbol) => {
                if !self.storage().persistent().has(&symbol) {
                    return None;
                }
                let index: u32 = self.storage().persistent().get(&symbol).unwrap();
                return Some(index as u8);
            }
            
        }
    }

    fn get_prices(&self, asset: u8, records: u32) -> Option<Vec<PriceData>> {
        prices(
            &self,
            |timestamp| self.get_price(asset.clone(), timestamp),
            records,
        )
    }

    fn get_x_price(&self, base_asset: u8, quote_asset: u8, timestamp: u64) -> Option<i128> {
        get_x_price(&self, &base_asset, &quote_asset, timestamp)
    }

    fn get_x_prices(
        &self,
        base_asset: u8,
        quote_asset: u8,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        prices(
            self,
            |timestamp| get_x_price(&self, &base_asset, &quote_asset, timestamp),
            records,
        )
    }

    fn invoker(&self) -> Option<Address> {
        let last_invoker = self.call_stack().first();
        if last_invoker.is_none() {
            return None;
        }
        Some(last_invoker.unwrap().0)
    }

    fn try_delete_old_price(&self, asset: u8, timestamp: u64, period: u64) -> bool {
        if timestamp < period {
            return false;
        }
        let data_key = U128Helper::encode_to_u128(timestamp - period, asset);
        if !self.storage().persistent().has(&data_key) {
            return false;
        }
        self.storage().persistent().remove(&data_key);
        true
    }

    fn panic_if_not_admin(&self, invoker: &Address) {
        if !self.is_authorized(invoker) {
            panic_with_error!(self, Error::Unauthorized);
        }
    }

    fn panic_if_version_invalid(&self, version: u32) {
        if version != self.get_config_version() + 1 {
            panic_with_error!(self, Error::InvalidConfigVersion);
        }
    }

    fn get_base_asset(&self) -> Asset {
        match Constants::BASE_ASSET_TYPE {
            AssetType::S => {
                let asset_bytes = BytesN::from_array(self, &Constants::BASE);
                let address = Address::from_contract_id(&asset_bytes);
                return Asset::S(address);
            }
            AssetType::G => {
                //drop the trailing zeros
                let first_zero_index = Constants::BASE
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(Constants::BASE.len());
                return Asset::G(Symbol::new(
                    self,
                    core::str::from_utf8(&Constants::BASE[..first_zero_index]).unwrap(),
                ));
            }
        }
    }
}

fn prices<F: Fn(u64) -> Option<i128>>(
    e: &Env,
    get_price_fn: F,
    records: u32,
) -> Option<Vec<PriceData>> {
    //check if the asset is valid
    let mut timestamp = e.get_last_timestamp();
    if timestamp == 0 {
        return None;
    }

    let mut prices = Vec::new(&e);
    let resolution = Constants::RESOLUTION as u64;

    let mut records = records;
    if records > 50 {
        records = 50;
    }

    for _ in 0..records {
        let price = get_price_fn(timestamp);
        if price.is_none() {
            //TODO: should we put None here?
            continue;
        }
        prices.push_back(PriceData {
            price: price.unwrap(),
            timestamp,
        });
        if timestamp < resolution {
            break;
        }
        timestamp -= resolution;
    }

    if prices.len() == 0 {
        return None;
    }

    Some(prices)
}

fn get_x_price(e: &Env, base_asset: &u8, quote_asset: &u8, timestamp: u64) -> Option<i128> {
    //check if the asset are the same
    if base_asset == quote_asset {
        return Some(10i128.pow(Constants::DECIMALS));
    }

    //get the price for base_asset
    let base_asset_price = e.get_price(base_asset.clone(), timestamp);
    if base_asset_price.is_none() {
        return None;
    }

    //get the price for quote_asset
    let quote_asset_price = e.get_price(quote_asset.clone(), timestamp);
    if quote_asset_price.is_none() {
        return None;
    }

    //calculate the cross price
    Some(
        base_asset_price
            .unwrap()
            .fixed_div_floor(quote_asset_price.unwrap(), Constants::DECIMALS),
    )
}
