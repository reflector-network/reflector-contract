#![no_std]

mod test;
mod types;
mod extensions;
mod constants;

use extensions::i128_extensions::I128Extensions;
use extensions::{env_extensions::EnvExtensions, u64_extensions::U64Extensions};
use constants::Constants;
use types::error::Error;
use types::asset::Asset;
use types::{config_data::ConfigData, price_data::PriceData};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Vec, BytesN};

#[contract]
pub struct PriceOracleContract;

#[contractimpl]
impl PriceOracleContract {
    //Admin section


    /// Updates the contract with the given WASM hash. Can only be called by the admin.
    /// 
    /// # Arguments
    /// 
    /// * `user` - The admin address.
    /// * `wasm_hash` - The WASM hash.
    /// 
    /// # Panics
    /// 
    /// Panics if the caller is not the admin.
    pub fn update_contract(env: Env, user: Address, wasm_hash: BytesN<32>) {
        env.panic_if_not_admin(&user);
        env.deployer().update_current_contract_wasm(wasm_hash)
    }

    /// Returns the contract major version.
    /// 
    /// # Returns
    /// 
    /// The contract major version.
    pub fn version(_: Env) -> u32 {
        env!("CARGO_PKG_VERSION").split(".").next().unwrap().parse::<u32>().unwrap()
    }

    /// Configures the contract with the given parameters. Can only be called by the admin.
    /// 
    /// # Arguments
    /// 
    /// * `user` - The admin address.
    /// * `config` - The configuration parameters.
    /// 
    /// # Panics
    /// 
    /// Panics if the contract is already initialized, or if the version is invalid.
    pub fn config(e: Env, user: Address, config: ConfigData) {
        user.require_auth();
        if e.is_initialized() {
            e.panic_with_error(Error::AlreadyInitialized);
        }
        e.set_admin(&config.admin);
        e.set_retention_period(config.period);

        Self::__add_assets(&e, config.assets);
    }

    /// Bumps the contract instance storage to the given number of ledgers.
    /// 
    /// # Arguments
    /// 
    /// * `ledgers_to_live` - The number of ledgers to live.
    /// 
    /// # Panics
    /// 
    /// Panics if ledgers_to_live is invalid.
    pub fn bump(e: Env, ledgers_to_live: u32) {
        e.bump(ledgers_to_live);
    }

    fn __add_assets(e: &Env, assets: Vec<Asset>) {
        let mut presented_assets = e.get_assets();

        let mut assets_indexes: Vec<(Asset, u32)> = Vec::new(&e);
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

    /// Adds the given assets to the contract. Can only be called by the admin.
    /// 
    /// # Arguments
    /// 
    /// * `user` - The admin address.
    /// * `assets` - The assets to add.
    /// * `version` - The configuration version.
    /// 
    /// # Panics
    /// 
    /// Panics if the caller is not the admin, or if the assets are already added, or if the version is invalid.
    pub fn add_assets(e: Env, user: Address, assets: Vec<Asset>) {
        e.panic_if_not_admin(&user);
        Self::__add_assets(&e, assets);
    }

    /// Sets the retention period for the prices. Can only be called by the admin.
    /// 
    /// # Arguments
    /// 
    /// * `user` - The admin address.
    /// * `period` - The retention period.
    /// * `version` - The configuration version.
    /// 
    /// # Panics
    /// 
    /// Panics if the caller is not the admin, or if the period is invalid, or if the version is invalid.
    pub fn set_period(e: Env, user: Address, period: u64) {
        e.panic_if_not_admin(&user);
        e.set_retention_period(period);
    }

    /// Sets the prices for the assets. Can only be called by the admin.
    /// 
    /// # Arguments
    /// 
    /// * `user` - The admin address.
    /// * `updates` - The prices to set.
    /// * `timestamp` - The timestamp of the prices.
    /// 
    /// # Panics
    /// 
    /// Panics if the caller is not the admin, or if the prices are invalid.
    pub fn set_price(e: Env, user: Address, updates: Vec<i128>, timestamp: u64) {
        e.panic_if_not_admin(&user);

        let retention_period = e.get_retention_period().unwrap();

        let ledgers_to_live: u32 = ((retention_period / 1000 / 5) + 1) as u32;

        //get the last timestamp
        let last_timestamp = e.get_last_timestamp();

        //iterate over the updates
        for (i, price) in updates.iter().enumerate() {
            let asset = i as u8;
            //store the new price
            e.set_price(asset, price, timestamp, ledgers_to_live);
        }
        if timestamp > last_timestamp {
            e.set_last_timestamp(timestamp);
        }
    }

    //end of admin section

    /// Returns the contract admin address.
    /// 
    /// # Returns
    /// 
    /// The admin address.
    pub fn admin(e: Env) -> Option<Address> {
        e.get_admin()
    }

    /// Returns the base asset.
    /// 
    /// # Returns
    /// 
    /// The base asset.
    pub fn base(e: Env) -> Asset {
        e.get_base_asset()
    }

    /// Returns the number of decimals for the prices.
    /// 
    /// # Returns
    /// 
    /// The number of decimals.
    pub fn decimals(_e: Env) -> u32 {
        Constants::DECIMALS
    }

    /// Returns the prices resolution.
    /// 
    /// # Returns
    /// 
    /// The prices resolution.
    pub fn resolution(_e: Env) -> u32 {
        //return resolution in seconds
        Constants::RESOLUTION / 1000
    }

    /// Returns the retention period of the prices in seconds.
    /// 
    /// # Returns
    /// 
    /// The retention period.
    pub fn period(e: Env) -> Option<u64> {
        e.get_retention_period()
    }

    /// Returns the assets supported by the contract.
    /// 
    /// # Returns
    /// 
    /// The assets supported by the contract or None if no assets are supported.
    pub fn assets(e: Env) -> Vec<Asset> {
        e.get_assets()
    }

    /// Returns the timestamp of the last price update.
    /// 
    /// # Returns
    /// 
    /// The timestamp of the last price update.
    pub fn last_timestamp(e: Env) -> u64 {
        e.get_last_timestamp()
    }

    /// Returns the prices for the given asset at the given timestamp.
    /// 
    /// # Arguments
    /// 
    /// * `asset` - The asset.
    /// * `timestamp` - The timestamp.
    /// 
    /// # Returns
    /// 
    /// The prices for the given asset at the given timestamp or None if the asset is not supported, or if the timestamp is invalid. 
    pub fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        let normalized_timestamp = timestamp.get_normalized_timestamp(Constants::RESOLUTION.into());
        //get the price
        get_price_data(&e, asset, normalized_timestamp)
    }

    /// Returns the last price for the given asset.
    /// 
    /// # Arguments
    /// 
    /// * `asset` - The asset.
    /// 
    /// # Returns
    /// 
    /// The last price for the given asset or None if the asset is not supported.
    pub fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        //get the last timestamp
        let timestamp = e.get_last_timestamp();
        //get the price
        get_price_data(&e, asset, timestamp)
    }

    /// Returns the cross price for the given assets at the given timestamp.
    /// 
    /// # Arguments
    /// 
    /// * `base_asset` - The base asset.
    /// * `quote_asset` - The quote asset.
    /// * `timestamp` - The timestamp.
    /// 
    /// # Returns
    /// 
    /// The cross price for the given assets at the given timestamp or None if the assets are not supported, or if the timestamp is invalid.
    pub fn x_price(
        e: Env,
        base_asset: Asset,
        quote_asset: Asset,
        timestamp: u64,
    ) -> Option<PriceData> {
        let normalized_timestamp = timestamp.get_normalized_timestamp(Constants::RESOLUTION.into());
        get_x_price(&e, base_asset, quote_asset, normalized_timestamp)
    }

    /// Returns the last cross price for the given assets.
    /// 
    /// # Arguments
    /// 
    /// * `base_asset` - The base asset.
    /// * `quote_asset` - The quote asset.
    /// 
    /// # Returns
    /// 
    /// The last cross price for the given assets or None if the assets are not supported.
    pub fn x_last_price(e: Env, base_asset: Asset, quote_asset: Asset) -> Option<PriceData> {
        let timestamp = e.get_last_timestamp();
        get_x_price(&e, base_asset, quote_asset, timestamp)
    }

    /// Returns the stack of prices for the given asset.
    /// 
    /// # Arguments
    /// 
    /// * `asset` - The asset.
    /// * `records` - The number of records to return.
    /// 
    /// # Returns
    /// 
    /// The prices for the given asset or None if the asset is not supported. If there are fewer records than requested, the returned vector will be shorter.
    pub fn prices(e: Env, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        let asset_index = e.get_asset_index(asset); //get the asset index to avoid multiple calls
        if asset_index.is_none() {
            return None;
        }
        prices(
            &e,
            |timestamp| get_price_data_by_index(&e, asset_index.unwrap(), timestamp),
            records,
        )
    }

    /// Returns the stack of cross prices for the given assets.
    /// 
    /// # Arguments
    /// 
    /// * `base_asset` - The base asset.
    /// * `quote_asset` - The quote asset.
    /// 
    /// # Returns
    /// 
    /// The cross prices for the given assets or None if the assets are not supported. If there are fewer records than requested, the returned vector will be shorter.
    pub fn x_prices(
        e: Env,
        base_asset: Asset,
        quote_asset: Asset,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        let asset_pair_indexes = get_asset_pair_indexes(&e, base_asset, quote_asset);
        if asset_pair_indexes.is_none() {
            return None;
        }
        prices(
            &e,
            |timestamp| get_x_price_by_indexes(&e, asset_pair_indexes.unwrap(), timestamp),
            records,
        )
    }

    /// Returns the time-weighted average price for the given asset over the given number of records.
    /// 
    /// # Arguments
    /// 
    /// * `asset` - The asset.
    /// * `records` - The number of records to use.
    /// 
    /// # Returns
    /// 
    /// The time-weighted average price for the given asset over the given number of records or None if the asset is not supported.
    pub fn twap(e: Env, asset: Asset, records: u32) -> Option<i128> {
        let asset_index = e.get_asset_index(asset); //get the asset index to avoid multiple calls
        if asset_index.is_none() {
            return None;
        }
        get_twap(
            &e,
            |timestamp| get_price_data_by_index(&e, asset_index.unwrap(), timestamp),
            records,
        )
    }

    /// Returns the time-weighted average cross price for the given assets over the given number of records.
    /// 
    /// # Arguments
    /// 
    /// * `base_asset` - The base asset.
    /// * `quote_asset` - The quote asset.
    /// 
    /// # Returns
    /// 
    /// The time-weighted average cross price for the given assets over the given number of records or None if the assets are not supported.
    pub fn x_twap(e: Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<i128> {
        let asset_pair_indexes = get_asset_pair_indexes(&e, base_asset, quote_asset); //get the asset index to avoid multiple calls
        if asset_pair_indexes.is_none() {
            return None;
        }
        get_twap(
            &e,
            |timestamp| get_x_price_by_indexes(&e, asset_pair_indexes.unwrap(), timestamp),
            records,
        )
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

fn prices<F: Fn(u64) -> Option<PriceData>>(
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
        prices.push_back(price.unwrap());
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

fn get_twap<F: Fn(u64) -> Option<PriceData>>(
    e: &Env,
    get_price_fn: F,
    records: u32,
) -> Option<i128> {
    let prices_result = prices(
        &e,
        get_price_fn,
        records,
    );
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

fn get_x_price(e: &Env, base_asset: Asset, quote_asset: Asset, timestamp: u64) -> Option<PriceData> {
    let asset_pair_indexes = get_asset_pair_indexes(e, base_asset, quote_asset);
    if asset_pair_indexes.is_none() {
        return None;
    }
    get_x_price_by_indexes(e, asset_pair_indexes.unwrap(), timestamp)
}

fn get_x_price_by_indexes(e: &Env, asset_pair_indexes: (u8, u8), timestamp: u64) -> Option<PriceData> {
    let (base_asset, quote_asset) = asset_pair_indexes;
    //check if the asset are the same
    if base_asset == quote_asset {
        return Some(PriceData { price: 10i128.pow(Constants::DECIMALS), timestamp });
    }

    //get the price for base_asset
    let base_asset_price = e.get_price(base_asset, timestamp);
    if base_asset_price.is_none() {
        return None;
    }

    //get the price for quote_asset
    let quote_asset_price = e.get_price(quote_asset, timestamp);
    if quote_asset_price.is_none() {
        return None;
    }

    //calculate the cross price
    Some(PriceData {
        price: base_asset_price
            .unwrap()
            .fixed_div_floor(quote_asset_price.unwrap(), Constants::DECIMALS),
        timestamp,
    })
}

fn get_asset_pair_indexes(e: &Env, base_asset: Asset, quote_asset: Asset) -> Option<(u8, u8)> {
    let base_asset = e.get_asset_index(base_asset);
    if base_asset.is_none() {
        return None;
    }

    let quote_asset = e.get_asset_index(quote_asset);
    if quote_asset.is_none() {
        return None;
    }

    Some((base_asset.unwrap(), quote_asset.unwrap()))
}

fn get_price_data(e: &Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
    let asset: Option<u8> = e.get_asset_index(asset);
    if asset.is_none() {
        return None;
    }
    get_price_data_by_index(e, asset.unwrap(), timestamp)
}


fn get_price_data_by_index(e: &Env, asset: u8, timestamp: u64) -> Option<PriceData> {
    let price = e.get_price(asset, timestamp);
    if price.is_none() {
        return None;
    }
    Some(PriceData {
        price: price.unwrap(),
        timestamp,
    })
}