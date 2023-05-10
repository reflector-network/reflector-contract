use soroban_sdk::{ Env, Address, panic_with_error, Vec, BytesN };
use crate::constants::Constants;
use crate::extensions::{env_extensions::EnvExtensions, u64_extensions::U64Extensions};
use crate::types::{ config_data::ConfigData, error::Error, price_data::PriceData };

pub struct PriceOracle;

impl PriceOracle {
    //Admin section

    pub fn config(e: &Env, user: Address, config: ConfigData) {
        e.panic_if_not_admin(&user);

        e.set_admin(&config.admin);
        e.set_retention_period(config.period);
        e.set_assets(config.assets);
    }

    pub fn add_assets(e: &Env, user: Address, assets: Vec<Address>) {
        e.panic_if_not_admin(&user);

        let mut presented_assets = e.get_assets();

        for asset in assets.iter() {
            let asset = asset.unwrap();
            //check if the asset is already added
            if is_asset_presented(&presented_assets, &asset) {
                panic_with_error!(&e, Error::AssetAlreadyPresented);
            }
            presented_assets.push_back(asset);
        }

        e.set_assets(presented_assets);
    }

    pub fn set_price(e: &Env, user: Address, updates: Vec<i128>, timestamp: u64) {
        e.panic_if_not_admin(&user);

        let assets = e.get_assets();
        let assets_len = assets.len();

        if updates.len() != assets_len {
            panic_with_error!(&e, Error::InvalidUpdatesLength);
        }

        let resolution = Constants::RESOLUTION;

        let retention_period = e.get_retention_period().unwrap();

        //get the last timestamp
        let last_timestamp = e.get_last_timestamp();

        let prev_timestamp = timestamp - (resolution as u64);

        //iterate over the updates
        for (i, price_data) in updates.iter().enumerate() {
            if !price_data.is_ok() {
                panic_with_error!(&e, Error::InvalidPriceValue);
            }

            let asset = assets.get_unchecked(i as u32).unwrap();

            let mut price = price_data.ok().unwrap();
            if price == 0 {
                if last_timestamp.is_none() {
                    panic_with_error!(&e, Error::NoPrevPrice);
                }
                //try to get previous price
                let prev_price = e.get_price(asset.clone(), prev_timestamp);
                if prev_price.is_none() {
                    panic_with_error!(&e, Error::NoPrevPrice);
                }
                price = prev_price.unwrap();
            }
            //store the new price
            e.set_price(asset.clone(), price, timestamp);

            //remove the old price
            e.try_delete_old_price(asset, timestamp, retention_period);
        }
        if last_timestamp.is_none() || timestamp > last_timestamp.unwrap() {
            e.set_last_timestamp(timestamp);
        }
    }

    //end of admin section

    pub fn admin(e: &Env) -> Address {
        e.get_admin()
    }

    pub fn base(e: &Env) -> Address {
        let bytes = BytesN::from_array(&e, &Constants::BASE);
        Address::from_contract_id(&e, &bytes)
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

    pub fn assets(e: &Env) -> Option<Vec<Address>> {
        let assets = e.get_assets();
        if assets.len() == 0 {
            return None;
        }
        Some(assets)
    }

    pub fn price(e: &Env, asset: Address, timestamp: u64) -> Option<PriceData> {
        let normalized_timestamp = timestamp.get_normalized_timestamp(Constants::RESOLUTION.into());

        //get the price
        let price = e.get_price(asset, normalized_timestamp);

        if price.is_none() {
            return None;
        }

        Some(PriceData {
            price: price.unwrap(),
            timestamp: normalized_timestamp,
        })
    }

    //Get the price for an asset.
    pub fn lastprice(e: &Env, asset: Address) -> Option<PriceData> {
        //get the last timestamp
        let timestamp = e.get_last_timestamp().unwrap_or(0);
        if timestamp == 0 {
            return None;
        }

        //get the price
        let price = e.get_price(asset, timestamp);
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
        base_asset: Address,
        quote_asset: Address,
        timestamp: u64,
    ) -> Option<PriceData> {
        let normalized_timestamp = timestamp.get_normalized_timestamp(Constants::RESOLUTION.into());

        let price = e.get_x_price(base_asset, quote_asset, normalized_timestamp);

        if price.is_none() {
            return None;
        }

        Some(PriceData {
            price: price.unwrap(),
            timestamp: normalized_timestamp,
        })
    }

    pub fn x_last_price(e: &Env, base_asset: Address, quote_asset: Address) -> Option<PriceData> {
        let timestamp = e.get_last_timestamp().unwrap_or(0);
        if timestamp == 0 {
            return None;
        }
        let price = e.get_x_price(base_asset, quote_asset, timestamp);

        if price.is_none() {
            return None;
        }

        Some(PriceData {
            price: price.unwrap(),
            timestamp,
        })
    }

    pub fn prices(e: &Env, asset: Address, records: u32) -> Option<Vec<PriceData>> {
        e.get_prices(asset, records)
    }

    pub fn x_prices(
        e: &Env,
        base_asset: Address,
        quote_asset: Address,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        e.get_x_prices(base_asset, quote_asset, records)
    }

    pub fn twap(e: &Env, asset: Address, records: u32) -> Option<i128> {
        let prices_result = e.get_prices(asset, records);
        if prices_result.is_none() {
            return None;
        }

        let prices = prices_result.unwrap();

        let mut sum = 0;
        for price in prices.iter() {
            let price_data = price.unwrap();
            sum += price_data.price;
        }

        Some(sum / (prices.len() as i128))
    }

    pub fn x_twap(e: &Env, base_asset: Address, quote_asset: Address, records: u32) -> Option<i128> {
        let prices_result = e.get_x_prices(base_asset, quote_asset, records);
        if prices_result.is_none() {
            return None;
        }

        let prices = prices_result.unwrap();

        let mut sum = 0;
        for price in prices.iter() {
            let price_data = price.unwrap();
            sum += price_data.price;
        }

        Some(sum / (prices.len() as i128))
    }
}

fn is_asset_presented(assets: &Vec<Address>, asset: &Address) -> bool {
    for a in assets.iter() {
        let a = a.unwrap();
        if &a == asset {
            return true;
        }
    }
    false
}