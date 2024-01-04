#![no_std]

mod extensions;
mod test;
mod types;

use extensions::i128_extensions::I128Extensions;
use extensions::{env_extensions::EnvExtensions, u64_extensions::U64Extensions};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, BytesN, Env, Vec};
use types::asset::Asset;
use types::error::Error;
use types::{config_data::ConfigData, price_data::PriceData};

#[contract]
pub struct PriceOracleContract;

#[contractimpl]
impl PriceOracleContract {
    // Returns the base asset the price is reported in.
    //
    // # Returns
    //
    // Base asset for the contract
    pub fn base(e: Env) -> Asset {
        e.get_base_asset()
    }

    // Returns the number of decimal places used to represent price for all assets quoted by the oracle.
    //
    // # Returns
    //
    // Number of decimals places in quoted prices
    pub fn decimals(e: Env) -> u32 {
        e.get_decimals()
    }

    // Returns the default tick period timeframe (in seconds).
    //
    // # Returns
    //
    // Price feed resolution (in seconds)
    pub fn resolution(e: Env) -> u32 {
        e.get_resolution() / 1000
    }

    // Returns the historical records retention period (in seconds).
    //
    // # Returns
    //
    // History retention period (in seconds)
    pub fn period(e: Env) -> Option<u64> {
        e.get_retention_period()
    }

    // Returns all assets quoted by the contract.
    //
    // # Returns
    //
    // Assets quoted by the contract
    pub fn assets(e: Env) -> Vec<Asset> {
        e.get_assets()
    }

    // Returns the most recent price update timestamp.
    //
    // # Returns
    //
    // Timestamp of the last recorded price update
    pub fn last_timestamp(e: Env) -> u64 {
        e.get_last_timestamp()
    }

    // Returns price in base asset at specific timestamp.
    //
    // # Arguments
    //
    // * `asset` - Asset to quote
    // * `timestamp` - Timestamp
    //
    // # Returns
    //
    // Price record for the given asset at the given timestamp or None if the record was not found
    pub fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        let resolution = e.get_resolution();
        let normalized_timestamp = timestamp.get_normalized_timestamp(resolution.into());
        //get the price
        get_price_data(&e, asset, normalized_timestamp)
    }

    // Returns the most recent price for an asset.
    //
    // # Arguments
    //
    // * `asset` - Asset to quote
    //
    // # Returns
    //
    // The most recent price for the given asset or None if the asset is not supported
    pub fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        //get the last timestamp
        let timestamp = e.get_last_timestamp();
        //get the price
        get_price_data(&e, asset, timestamp)
    }

    // Returns last N price records for the given asset.
    //
    // # Arguments
    //
    // * `asset` - Asset to quote
    // * `records` - Number of records to return
    //
    // # Returns
    //
    // Prices for the given asset or None if the asset is not supported
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

    // Returns the most recent cross price record for the pair of assets.
    //
    // # Arguments
    //
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    //
    // # Returns
    //
    // The most recent cross price (base_asset_price/quote_asset_price) for the given assets or None if if there were no records found for quoted asset
    pub fn x_last_price(e: Env, base_asset: Asset, quote_asset: Asset) -> Option<PriceData> {
        let timestamp = e.get_last_timestamp();
        let decimals = e.get_decimals();
        get_x_price(&e, base_asset, quote_asset, timestamp, decimals)
    }

    // Returns the cross price for the pair of assets at specific timestamp.
    //
    // # Arguments
    //
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    // * `timestamp` - Timestamp
    //
    // # Returns
    //
    // Cross price (base_asset_price/quote_asset_price) at the given timestamp or None if there were no records found for quoted assets at specific timestamp
    pub fn x_price(
        e: Env,
        base_asset: Asset,
        quote_asset: Asset,
        timestamp: u64,
    ) -> Option<PriceData> {
        let normalized_timestamp = timestamp.get_normalized_timestamp(e.get_resolution().into());
        let decimals = e.get_decimals();
        get_x_price(&e, base_asset, quote_asset, normalized_timestamp, decimals)
    }

    // Returns last N cross price records of for the pair of assets.
    //
    // # Arguments
    //
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    //
    // # Returns
    //
    // Last N cross prices (base_asset_price/quote_asset_price) or None if there were no records found for quoted assets
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
        let decimals = e.get_decimals();
        prices(
            &e,
            |timestamp| {
                get_x_price_by_indexes(&e, asset_pair_indexes.unwrap(), timestamp, decimals)
            },
            records,
        )
    }

    // Returns the time-weighted average price for the given asset over N recent records.
    //
    // # Arguments
    //
    // * `asset` - Asset to quote
    // * `records` - Number of records to process
    //
    // # Returns
    //
    // TWAP for the given asset over N recent records or None if the asset is not supported
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

    // Returns the time-weighted average cross price for the given asset pair over N recent records.
    //
    // # Arguments
    //
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    //
    // # Returns
    //
    // TWAP (base_asset_price/quote_asset_price) or None if the assets are not supported.
    pub fn x_twap(e: Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<i128> {
        //get asset index to avoid multiple calls
        let asset_pair_indexes = get_asset_pair_indexes(&e, base_asset, quote_asset);
        if asset_pair_indexes.is_none() {
            return None;
        }
        let decimals = e.get_decimals();
        get_twap(
            &e,
            |timestamp| {
                get_x_price_by_indexes(&e, asset_pair_indexes.unwrap(), timestamp, decimals)
            },
            records,
        )
    }

    // Returns current protocol version of the contract.
    //
    // # Returns
    //
    // Contract protocol version
    pub fn version(_e: Env) -> u32 {
        env!("CARGO_PKG_VERSION")
            .split(".")
            .next()
            .unwrap()
            .parse::<u32>()
            .unwrap()
    }

    //Admin section

    // Returns admin address of the contract.
    //
    // # Returns
    //
    // Contract admin account address
    pub fn admin(e: Env) -> Option<Address> {
        e.get_admin()
    }

    // Updates the contract configuration parameters. Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `admin` - Admin account address
    // * `config` - Configuration parameters
    //
    // # Panics
    //
    // Panics if the contract is already initialized, or if the version is invalid
    pub fn config(e: Env, admin: Address, config: ConfigData) {
        admin.require_auth();
        if e.is_initialized() {
            e.panic_with_error(Error::AlreadyInitialized);
        }
        e.set_admin(&config.admin);
        e.set_base_asset(&config.base_asset);
        e.set_decimals(config.decimals);
        e.set_resolution(config.resolution);
        e.set_retention_period(config.period);

        Self::__add_assets(&e, config.assets);
    }

    // Bumps the contract instance storage expiration to the given number of ledgers.
    //
    // # Arguments
    //
    // * `ledgers_to_live` - Extension period specified in ledgers count
    //
    // # Panics
    //
    // Panics if ledgers_to_live is invalid
    pub fn bump(e: Env, ledgers_to_live: u32) {
        e.bump(ledgers_to_live);
    }

    // Adds given assets to the contract quoted assets list. Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `admin` - Admin account address
    // * `assets` - Assets to add
    // * `version` - Configuration protocol version
    //
    // # Panics
    //
    // Panics if the caller doesn't match admin address, or if the assets are already added
    pub fn add_assets(e: Env, admin: Address, assets: Vec<Asset>) {
        e.panic_if_not_admin(&admin);
        Self::__add_assets(&e, assets);
    }

    // Sets history retention period for the prices. Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `admin` - Admin account address
    // * `period` - History retention period (in seconds)
    // * `version` - Configuration protocol version
    //
    // # Panics
    //
    // Panics if the caller doesn't match admin address, or if the period/version is invalid
    pub fn set_period(e: Env, admin: Address, period: u64) {
        e.panic_if_not_admin(&admin);
        e.set_retention_period(period);
    }

    // Record new price feed history snapshot. Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `admin` - Admin account address
    // * `updates` - Price feed snapshot
    // * `timestamp` - History snapshot timestamp
    //
    // # Panics
    //
    // Panics if the caller doesn't match admin address, or if the price snapshot record is invalid
    pub fn set_price(e: Env, admin: Address, updates: Vec<i128>, timestamp: u64) {
        e.panic_if_not_admin(&admin);

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

    // Updates the contract source code. Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `admin` - Admin account address
    // * `wasm_hash` - WASM hash of the contract source code
    //
    // # Panics
    //
    // Panics if the caller doesn't match admin address
    pub fn update_contract(env: Env, admin: Address, wasm_hash: BytesN<32>) {
        env.panic_if_not_admin(&admin);
        env.deployer().update_current_contract_wasm(wasm_hash)
    }

    fn __add_assets(e: &Env, assets: Vec<Asset>) {
        let mut presented_assets = e.get_assets();

        let mut assets_indexes: Vec<(Asset, u32)> = Vec::new(&e);
        for asset in assets.iter() {
            //check if the asset has been already added
            if has_asset(&presented_assets, &asset) {
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
}

fn has_asset(assets: &Vec<Asset>, asset: &Asset) -> bool {
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
    let resolution = e.get_resolution() as u64;

    let mut records = records;
    if records > 20 {
        records = 20;
    }

    for _ in 0..records {
        let price = get_price_fn(timestamp);
        if price.is_none() {
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
    let prices_result = prices(&e, get_price_fn, records);
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

fn get_x_price(
    e: &Env,
    base_asset: Asset,
    quote_asset: Asset,
    timestamp: u64,
    decimals: u32,
) -> Option<PriceData> {
    let asset_pair_indexes = get_asset_pair_indexes(e, base_asset, quote_asset);
    if asset_pair_indexes.is_none() {
        return None;
    }
    get_x_price_by_indexes(e, asset_pair_indexes.unwrap(), timestamp, decimals)
}

fn get_x_price_by_indexes(
    e: &Env,
    asset_pair_indexes: (u8, u8),
    timestamp: u64,
    decimals: u32,
) -> Option<PriceData> {
    let (base_asset, quote_asset) = asset_pair_indexes;
    //check if the asset are the same
    if base_asset == quote_asset {
        return Some(PriceData {
            price: 10i128.pow(decimals),
            timestamp,
        });
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
            .fixed_div_floor(quote_asset_price.unwrap(), decimals),
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
