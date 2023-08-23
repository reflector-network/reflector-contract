use crate::constants::Constants;
use crate::extensions::{env_extensions::EnvExtensions, u64_extensions::U64Extensions};
use crate::types::asset::Asset;
use crate::types::{config_data::ConfigData, error::Error, price_data::PriceData};
use soroban_sdk::{panic_with_error, Address, Env, Vec};

pub struct PriceOracle;

impl PriceOracle {
    //Admin section

    pub fn config(e: &Env, user: Address, config: ConfigData) {
        user.require_auth();
        if e.is_initialized() {
            e.panic_with_error(Error::AlreadyInitialized);
        }
        e.panic_if_version_invalid(config.version);
        e.set_admin(&config.admin);
        e.set_retention_period(config.period);

        Self::__add_assets(e, config.assets);
        e.set_config_version(config.version);
    }

    pub fn add_assets(e: &Env, user: Address, assets: Vec<Asset>, version: u32) {
        e.panic_if_not_admin(&user);
        e.panic_if_version_invalid(version);
        Self::__add_assets(e, assets);
        e.set_config_version(version);
    }

    fn __add_assets(e: &Env, assets: Vec<Asset>) {
        let mut presented_assets = e.get_assets();

        let mut assets_indexes: Vec<(Asset, u32)> = Vec::new(e);
        for asset in assets.iter() {
            //check if the asset is already added
            if is_asset_presented(&presented_assets, &asset) {
                panic_with_error!(&e, Error::AssetAlreadyPresented);
            }
            presented_assets.push_back(asset.clone());
            assets_indexes.push_back((asset, presented_assets.len() as u32 - 1));
        }

        e.set_assets(presented_assets);
        for (asset, index) in assets_indexes.iter() {
            e.set_asset_index(asset, index);
        }
    }

    pub fn set_period(e: &Env, user: Address, period: u64, version: u32) {
        e.panic_if_not_admin(&user);
        e.panic_if_version_invalid(version);
        e.set_retention_period(period);
        e.set_config_version(version);
    }

    pub fn set_price(e: &Env, user: Address, updates: Vec<i128>, timestamp: u64) {
        e.panic_if_not_admin(&user);

        let retention_period = e.get_retention_period().unwrap();

        //get the last timestamp
        let last_timestamp = e.get_last_timestamp();

        //iterate over the updates
        for (i, price) in updates.iter().enumerate() {
            let asset = i as u8;
            //store the new price
            e.set_price(asset, price, timestamp);

            //remove the old price
            e.try_delete_old_price(asset, timestamp, retention_period);
        }
        if timestamp > last_timestamp {
            e.set_last_timestamp(timestamp);
        }
    }

    //end of admin section

    pub fn admin(e: &Env) -> Address {
        e.get_admin()
    }

    pub fn config_version(e: &Env) -> u32 {
        e.get_config_version()
    }

    pub fn base(e: &Env) -> Asset {
        e.get_base_asset()
    }

    pub fn decimals(_e: &Env) -> u32 {
        Constants::DECIMALS
    }

    pub fn resolution(_e: &Env) -> u32 {
        //return resolution in seconds
        Constants::RESOLUTION / 1000
    }

    pub fn period(e: &Env) -> Option<u64> {
        e.get_retention_period()
    }

    pub fn assets(e: &Env) -> Vec<Asset> {
        e.get_assets()
    }

    pub fn last_timestamp(e: &Env) -> u64 {
        e.get_last_timestamp()
    }

    pub fn price(e: &Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        let normalized_timestamp = timestamp.get_normalized_timestamp(Constants::RESOLUTION.into());

        let asset = e.get_asset_index(asset);
        if asset.is_none() {
            return None;
        }
        //get the price
        let price = e.get_price(asset.unwrap(), normalized_timestamp);

        if price.is_none() {
            return None;
        }

        Some(PriceData {
            price: price.unwrap(),
            timestamp: normalized_timestamp,
        })
    }

    //Get the price for an asset.
    pub fn lastprice(e: &Env, asset: Asset) -> Option<PriceData> {
        //get the last timestamp
        let timestamp = e.get_last_timestamp();
        if timestamp == 0 {
            return None;
        }

        let asset = e.get_asset_index(asset);
        if asset.is_none() {
            return None;
        }

        //get the price
        let price = e.get_price(asset.unwrap(), timestamp);
        if price.is_none() {
            return None;
        }

        Some(PriceData {
            price: price.unwrap(),
            timestamp,
        })
    }

    pub fn x_price(
        e: &Env,
        base_asset: Asset,
        quote_asset: Asset,
        timestamp: u64,
    ) -> Option<PriceData> {
        let normalized_timestamp = timestamp.get_normalized_timestamp(Constants::RESOLUTION.into());

        let base_asset = e.get_asset_index(base_asset);
        if base_asset.is_none() {
            return None;
        }

        let quote_asset = e.get_asset_index(quote_asset);
        if base_asset.is_none() {
            return None;
        }

        let price = e.get_x_price(base_asset.unwrap(), quote_asset.unwrap(), normalized_timestamp);

        if price.is_none() {
            return None;
        }

        Some(PriceData {
            price: price.unwrap(),
            timestamp: normalized_timestamp,
        })
    }

    pub fn x_last_price(e: &Env, base_asset: Asset, quote_asset: Asset) -> Option<PriceData> {
        let timestamp = e.get_last_timestamp();
        if timestamp == 0 {
            return None;
        }

        let base_asset = e.get_asset_index(base_asset);
        if base_asset.is_none() {
            return None;
        }

        let quote_asset = e.get_asset_index(quote_asset);
        if quote_asset.is_none() {
            return None;
        }

        let price = e.get_x_price(base_asset.unwrap(), quote_asset.unwrap(), timestamp);

        if price.is_none() {
            return None;
        }

        Some(PriceData {
            price: price.unwrap(),
            timestamp,
        })
    }

    pub fn prices(e: &Env, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        let asset = e.get_asset_index(asset);
        if asset.is_none() {
            return None;
        }
        e.get_prices(asset.unwrap(), records)
    }

    pub fn x_prices(
        e: &Env,
        base_asset: Asset,
        quote_asset: Asset,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        let base_asset = e.get_asset_index(base_asset);
        if base_asset.is_none() {
            return None;
        }
        let quote_asset = e.get_asset_index(quote_asset);
        if quote_asset.is_none() {
            return None;
        }
        e.get_x_prices(base_asset.unwrap(), quote_asset.unwrap(), records)
    }

    pub fn twap(e: &Env, asset: Asset, records: u32) -> Option<i128> {
        let asset = e.get_asset_index(asset);
        if asset.is_none() {
            return None;
        }
        let prices_result: Option<Vec<PriceData>> = e.get_prices(asset.unwrap(), records);
        if prices_result.is_none() {
            return None;
        }

        let prices = prices_result.unwrap();

        let mut sum = 0;
        for price_data in prices.iter() {
            sum += price_data.price;
        }

        Some(sum / (prices.len() as i128))
    }

    pub fn x_twap(e: &Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<i128> {
        let base_asset = e.get_asset_index(base_asset);
        if base_asset.is_none() {
            return None;
        }
        let quote_asset = e.get_asset_index(quote_asset);
        if quote_asset.is_none() {
            return None;
        }
        let prices_result = e.get_x_prices(base_asset.unwrap(), quote_asset.unwrap(), records);
        if prices_result.is_none() {
            return None;
        }

        let prices = prices_result.unwrap();

        let mut sum = 0;
        for price_data in prices.iter() {
            sum += price_data.price;
        }

        Some(sum / (prices.len() as i128))
    }
}

fn is_asset_presented(assets: &Vec<Asset>, asset: &Asset) -> bool {
    for current_asset in assets.iter() {
        if &current_asset == asset {
            return true;
        }
    }
    false
}
