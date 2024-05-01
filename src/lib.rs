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
        let period = e.get_retention_period();
        if period == 0 {
            return None;
        } else {
            return Some(period / 1000); //convert to seconds
        }
    }

    // Returns all assets quoted by the contract.
    //
    // # Returns
    //
    // Assets quoted by the contract
    pub fn assets(e: Env) -> Vec<Asset> {
        e.get_assets()
    }

    // Returns the most recent price update timestamp in seconds.
    //
    // # Returns
    //
    // Timestamp of the last recorded price update
    pub fn last_timestamp(e: Env) -> u64 {
        e.get_last_timestamp() / 1000 //convert to seconds
    }

    // Returns price in base asset at specific timestamp.
    //
    // # Arguments
    //
    // * `asset` - Asset to quote
    // * `timestamp` - Timestamp in seconds
    //
    // # Returns
    //
    // Price record for the given asset at the given timestamp or None if the record was not found
    pub fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        let resolution = e.get_resolution();
        let normalized_timestamp = get_timestamp_in_ms(timestamp).get_normalized_timestamp(resolution.into());
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
        let timestamp = obtain_record_timestamp(&e);
        if timestamp == 0 {
            return None;
        }
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
        let asset_index = e.get_asset_index(&asset); //get the asset index to avoid multiple calls
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
        let timestamp = obtain_record_timestamp(&e);
        if timestamp == 0 {
            return None;
        }
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
        let normalized_timestamp = get_timestamp_in_ms(timestamp).get_normalized_timestamp(e.get_resolution().into());
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
        let asset_index = e.get_asset_index(&asset); //get the asset index to avoid multiple calls
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
    pub fn config(e: Env, config: ConfigData) {
        config.admin.require_auth();
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
    pub fn add_assets(e: Env, assets: Vec<Asset>) {
        e.panic_if_not_admin();
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
    pub fn set_period(e: Env, period: u64) {
        e.panic_if_not_admin();
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
    pub fn set_price(e: Env, updates: Vec<i128>, timestamp: u64) {
        e.panic_if_not_admin();
        let updates_len = updates.len();
        if updates_len == 0 || updates_len >= 256 {
            panic_with_error!(&e, Error::InvalidUpdateLength);
        }
        let timeframe: u64 = e.get_resolution().into();
        let ledger_timestamp = now(&e);
        if timestamp == 0
            || !timestamp.is_valid_timestamp(timeframe)
            || timestamp > ledger_timestamp
        {
            panic_with_error!(&e, Error::InvalidTimestamp);
        }

        let retention_period = e.get_retention_period();

        let ledgers_to_live: u32 = ((retention_period / 1000 / 5) + 1) as u32;

        //get the last timestamp
        let last_timestamp = e.get_last_timestamp();

        //iterate over the updates
        for (i, price) in updates.iter().enumerate() {
            //don't store zero prices
            if price == 0 {
                continue;
            }
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
    pub fn update_contract(env: Env, wasm_hash: BytesN<32>) {
        env.panic_if_not_admin();
        env.deployer().update_current_contract_wasm(wasm_hash)
    }

    fn __add_assets(e: &Env, assets: Vec<Asset>) {
        let mut current_assets = e.get_assets();
        for asset in assets.iter() {
            //check if the asset has been already added
            if e.get_asset_index(&asset).is_some() {
                panic_with_error!(&e, Error::AssetAlreadyExists);
            }
            e.set_asset_index(&asset, current_assets.len());
            current_assets.push_back(asset);
        }
        if current_assets.len() >= 256 {
            panic_with_error!(&e, Error::AssetLimitExceeded);
        }
        e.set_assets(current_assets);
    }
}

fn prices<F: Fn(u64) -> Option<PriceData>>(
    e: &Env,
    get_price_fn: F,
    mut records: u32,
) -> Option<Vec<PriceData>> {
    // Check if the asset is valid
    let mut timestamp = obtain_record_timestamp(e);
    if timestamp == 0 {
        return None;
    }

    let mut prices = Vec::new(e);
    let resolution = e.get_resolution() as u64;

    // Limit the number of records to 20
    records = records.min(20);

    while records > 0 {
        if let Some(price) = get_price_fn(timestamp) {
            prices.push_back(price);
        }

        // Decrement records counter in every iteration
        records -= 1;

        if timestamp < resolution {
            break;
        }
        timestamp -= resolution;
    }

    if prices.is_empty() {
        None
    } else {
        Some(prices)
    }
}

fn get_timestamp_in_ms(timestamp: u64) -> u64 {
    timestamp * 1000 //convert to milliseconds
}

fn now(e: &Env) -> u64 {
    e.ledger().timestamp() * 1000 //convert to milliseconds
}

fn obtain_record_timestamp(e: &Env) -> u64 {
    let last_timestamp = e.get_last_timestamp();
    let ledger_timestamp = now(&e);
    let resolution = e.get_resolution() as u64;
    if last_timestamp == 0 //no prices yet
        || last_timestamp > ledger_timestamp //last timestamp is in the future
        || ledger_timestamp - last_timestamp >= resolution * 2
    //last timestamp is too far in the past, so we cannot return the last price
    {
        return 0;
    }
    last_timestamp
}

fn get_twap<F: Fn(u64) -> Option<PriceData>>(
    e: &Env,
    get_price_fn: F,
    records: u32,
) -> Option<i128> {
    let prices = prices(&e, get_price_fn, records)?;

    if prices.len() != records {
        return None;
    }

    let last_price_timestamp = prices.first()?.timestamp * 1000; //convert to milliseconds to match the timestamp format
    let timeframe = e.get_resolution() as u64;
    let current_time = now(&e);

    //check if the last price is too old
    if last_price_timestamp + timeframe + 60 * 1000 < current_time {
        return None;
    }

    let sum: i128 = prices.iter().map(|price_data| price_data.price).sum();
    Some(sum / prices.len() as i128)
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
        return Some(get_normalized_price_data(
            10i128.pow(decimals),
            timestamp,
        ));
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
    Some(get_normalized_price_data(
        base_asset_price
            .unwrap()
            .fixed_div_floor(quote_asset_price.unwrap(), decimals),
        timestamp,
    ))
}

fn get_asset_pair_indexes(e: &Env, base_asset: Asset, quote_asset: Asset) -> Option<(u8, u8)> {
    let base_asset = e.get_asset_index(&base_asset);
    if base_asset.is_none() {
        return None;
    }

    let quote_asset = e.get_asset_index(&quote_asset);
    if quote_asset.is_none() {
        return None;
    }

    Some((base_asset.unwrap(), quote_asset.unwrap()))
}

fn get_price_data(e: &Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
    let asset: Option<u8> = e.get_asset_index(&asset);
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
    Some(get_normalized_price_data(price.unwrap(), timestamp))
}

fn get_normalized_price_data(price: i128, timestamp: u64) -> PriceData {
    PriceData {
        price,
        timestamp: timestamp / 1000, //convert to seconds
    }
}
