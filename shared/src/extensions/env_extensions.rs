use soroban_sdk::{panic_with_error, Address, BytesN, Env, Vec, Symbol};

use crate::constants;
use crate::extensions;
use crate::types;

use constants::Constants;
use extensions::i128_extensions::I128Extensions;
use types::{
    asset::Asset, asset_price_key::AssetPriceKey, asset_type::AssetType, data_key::DataKey,
    error::Error, price_data::PriceData,
};

pub trait EnvExtensions {
    fn is_authorized(&self, invoker: &Address) -> bool;

    fn get_admin(&self) -> Address;

    fn set_admin(&self, admin: &Address);

    fn get_price(&self, asset: Asset, timestamp: u64) -> Option<i128>;

    fn set_price(&self, asset: Asset, price: i128, timestamp: u64);

    fn get_last_timestamp(&self) -> Option<u64>;

    fn set_last_timestamp(&self, timestamp: u64);

    fn get_retention_period(&self) -> Option<u64>;

    fn set_retention_period(&self, period: u64);

    fn get_assets(&self) -> Vec<Asset>;

    fn set_assets(&self, assets: Vec<Asset>);

    fn get_prices(&self, asset: Asset, records: u32) -> Option<Vec<PriceData>>;

    fn get_x_price(&self, base_asset: Asset, quote_asset: Asset, timestamp: u64) -> Option<i128>;

    fn get_x_prices(
        &self,
        base_asset: Asset,
        quote_asset: Asset,
        records: u32,
    ) -> Option<Vec<PriceData>>;

    fn invoker(&self) -> Option<Address>;

    fn try_delete_data(&self, key: DataKey) -> bool;

    fn try_delete_old_price(&self, asset: Asset, timestamp: u64, period: u64) -> bool;

    fn panic_if_not_admin(&self, invoker: &Address);

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
        self.storage().has(&DataKey::Admin)
    }

    fn get_admin(&self) -> Address {
        self.storage().get_unchecked(&DataKey::Admin).unwrap()
    }

    fn set_admin(&self, admin: &Address) {
        self.storage().set(&DataKey::Admin, admin);
    }

    fn get_price(&self, asset: Asset, timestamp: u64) -> Option<i128> {
        //build the key for the price
        let data_key = DataKey::Price(AssetPriceKey { asset, timestamp });

        //check if the price is available
        if !self.storage().has(&data_key) {
            return None;
        }

        //get the price
        Some(self.storage().get_unchecked(&data_key).unwrap())
    }

    fn set_price(&self, asset: Asset, price: i128, timestamp: u64) {
        //build the key for the price
        let data_key = DataKey::Price(AssetPriceKey {
            asset: asset.clone(),
            timestamp,
        });

        //set the price
        self.storage().set(&data_key, &price);
    }

    fn get_last_timestamp(&self) -> Option<u64> {
        //check if the marker is available
        if !self.storage().has(&DataKey::LastTimestamp) {
            return None;
        }

        //get the marker
        Some(
            self.storage()
                .get_unchecked(&DataKey::LastTimestamp)
                .unwrap(),
        )
    }

    fn set_last_timestamp(&self, timestamp: u64) {
        self.storage().set(&DataKey::LastTimestamp, &timestamp);
    }

    fn get_retention_period(&self) -> Option<u64> {
        if !self.storage().has(&DataKey::RetentionPeriod) {
            return None;
        }
        Some(
            self.storage()
                .get_unchecked(&DataKey::RetentionPeriod)
                .unwrap(),
        )
    }

    fn set_retention_period(&self, rdm_period: u64) {
        self.storage().set(&DataKey::RetentionPeriod, &rdm_period);
    }

    fn get_assets(&self) -> Vec<Asset> {
        if !self.storage().has(&DataKey::Assets) {
            //return empty vector
            return Vec::new(&self);
        }
        self.storage().get_unchecked(&DataKey::Assets).unwrap()
    }

    fn set_assets(&self, assets: Vec<Asset>) {
        self.storage().set(&DataKey::Assets, &assets);
    }

    fn get_prices(&self, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        prices(
            &self,
            |timestamp| self.get_price(asset.clone(), timestamp),
            records,
        )
    }

    fn get_x_price(&self, base_asset: Asset, quote_asset: Asset, timestamp: u64) -> Option<i128> {
        get_x_price(&self, &base_asset, &quote_asset, timestamp)
    }

    fn get_x_prices(
        &self,
        base_asset: Asset,
        quote_asset: Asset,
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
        let unwraped = last_invoker.unwrap();
        if !unwraped.is_ok() {
            return None;
        }
        Some(unwraped.ok().unwrap().0)
    }

    fn try_delete_data(&self, key: DataKey) -> bool {
        if !self.storage().has(&key) {
            return false;
        }
        self.storage().remove(&key);
        true
    }

    fn try_delete_old_price(&self, asset: Asset, timestamp: u64, period: u64) -> bool {
        if timestamp < period {
            return false;
        }
        let data_key = DataKey::Price(AssetPriceKey {
            asset,
            timestamp: timestamp - period,
        });
        if !self.storage().has(&data_key) {
            return false;
        }
        self.storage().remove(&data_key);
        true
    }

    fn panic_if_not_admin(&self, invoker: &Address) {
        if !self.is_authorized(invoker) {
            panic_with_error!(self, Error::Unauthorized);
        }
    }

    fn get_base_asset(&self) -> Asset {
        match Constants::BASE_ASSET_TYPE {
            AssetType::STELLAR => {
                let asset_bytes = BytesN::from_array(self, &Constants::BASE);
                let address = Address::from_contract_id(&asset_bytes);
                return Asset::Stellar(address);
            }
            AssetType::GENERIC => {
                //drop the trailing zeros
                let first_zero_index = Constants::BASE.iter().position(|&b| b == 0).unwrap_or(Constants::BASE.len());
                return Asset::Generic(Symbol::new(
                    self,
                    core::str::from_utf8(&Constants::BASE[..first_zero_index]).unwrap()
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
    let mut timestamp = e.get_last_timestamp().unwrap_or(0);
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
        timestamp -= resolution;
    }

    if prices.len() == 0 {
        return None;
    }

    Some(prices)
}

fn get_x_price(e: &Env, base_asset: &Asset, quote_asset: &Asset, timestamp: u64) -> Option<i128> {
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
