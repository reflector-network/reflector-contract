#![allow(non_upper_case_globals)]
use soroban_sdk::storage::{Instance, Temporary};
use soroban_sdk::{panic_with_error, Address, BytesN, Env, Symbol, Vec};

use crate::constants;
use crate::extensions;
use crate::types;

use constants::Constants;
use extensions::u128_helper::U128Helper;
use types::{asset::Asset, asset_type::AssetType, error::Error};
const ADMIN_KEY: &str = "admin";
const LAST_TIMESTAMP: &str = "last_timestamp";
const RETENTION_PERIOD: &str = "period";
const ASSETS: &str = "assets";

pub trait EnvExtensions {
    fn is_authorized(&self, invoker: &Address) -> bool;

    fn get_admin(&self) -> Option<Address>;

    fn set_admin(&self, admin: &Address);

    fn get_price(&self, asset: u8, timestamp: u64) -> Option<i128>;

    fn set_price(&self, asset: u8, price: i128, timestamp: u64, ledgers: u32);

    fn get_last_timestamp(&self) -> u64;

    fn set_last_timestamp(&self, timestamp: u64);

    fn get_retention_period(&self) -> Option<u64>;

    fn set_retention_period(&self, period: u64);

    fn get_assets(&self) -> Vec<Asset>;

    fn set_assets(&self, assets: Vec<Asset>);

    fn set_asset_index(&self, asset: Asset, index: u32);

    fn get_asset_index(&self, asset: Asset) -> Option<u8>;

    fn panic_if_not_admin(&self, invoker: &Address);

    fn get_base_asset(&self) -> Asset;

    fn is_initialized(&self) -> bool;

    fn bump(&self, ledgers_to_live: u32);
}

impl EnvExtensions for Env {
    fn is_authorized(&self, invoker: &Address) -> bool {
        invoker.require_auth();

        //invoke get_admin to check if the admin is set
        let admin = self.get_admin();
        !admin.is_none() && invoker == &admin.unwrap()
    }

    fn is_initialized(&self) -> bool {
        get_instance_storage(&self).has(&ADMIN_KEY)
    }

    fn get_admin(&self) -> Option<Address> {
        get_instance_storage(&self).get(&ADMIN_KEY)
    }

    fn set_admin(&self, admin: &Address) {
        get_instance_storage(&self).set(&ADMIN_KEY, admin);
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
        if ledgers_to_live > 16 { //16 is the minimum number 
            temps_storage.bump(&data_key, ledgers_to_live, ledgers_to_live)
        }
    }

    fn get_last_timestamp(&self) -> u64 {
        //get the marker
        get_instance_storage(&self).get(&LAST_TIMESTAMP).unwrap_or_default()
    }

    fn set_last_timestamp(&self, timestamp: u64) {
        get_instance_storage(&self).set(&LAST_TIMESTAMP, &timestamp);
    }

    fn get_retention_period(&self) -> Option<u64> {
        get_instance_storage(&self)
            .get(&RETENTION_PERIOD)
            .unwrap_or_default()
    }

    fn set_retention_period(&self, rdm_period: u64) {
        get_instance_storage(&self).set(&RETENTION_PERIOD, &rdm_period);
    }

    fn get_assets(&self) -> Vec<Asset> {
        get_instance_storage(&self)
            .get(&ASSETS)
            .unwrap_or_else(|| Vec::new(&self))
    }

    fn set_assets(&self, assets: Vec<Asset>) {
        get_instance_storage(&self).set(&ASSETS, &assets);
    }

    fn set_asset_index(&self, asset: Asset, index: u32) {
        match asset {
            Asset::Stellar(address) => {
                get_instance_storage(&self).set(&address, &index);
            }
            Asset::Generic(symbol) => {
                get_instance_storage(&self).set(&symbol, &index);
            }
        }
    }

    fn get_asset_index(&self, asset: Asset) -> Option<u8> {
        let index: Option<u32>;
        match asset {
            Asset::Stellar(address) => {
                index = get_instance_storage(self).get(&address);
            }
            Asset::Generic(symbol) => {
                index = get_instance_storage(self).get(&symbol);
            }
        }
        if index.is_none() {
            return None;
        }
        return Some(index.unwrap() as u8);
    }

    fn panic_if_not_admin(&self, invoker: &Address) {
        if !self.is_authorized(invoker) {
            panic_with_error!(self, Error::Unauthorized);
        }
    }

    fn get_base_asset(&self) -> Asset {
        match Constants::BASE_ASSET_TYPE {
            AssetType::Stellar => {
                let asset_bytes = BytesN::from_array(self, &Constants::BASE);
                let address = Address::from_contract_id(&asset_bytes);
                return Asset::Stellar(address);
            }
            AssetType::Generic => {
                //drop the trailing zeros
                let first_zero_index = Constants::BASE
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(Constants::BASE.len());
                return Asset::Generic(Symbol::new(
                    self,
                    core::str::from_utf8(&Constants::BASE[..first_zero_index]).unwrap(),
                ));
            }
        }
    }

    fn bump(&self, ledgers_to_live: u32) {
        get_instance_storage(&self).bump(ledgers_to_live, ledgers_to_live);
    }

}

fn get_instance_storage(e: &Env) -> Instance {
    e.storage().instance()
}

fn get_temporary_storage(e: &Env) -> Temporary {
    e.storage().temporary()
}