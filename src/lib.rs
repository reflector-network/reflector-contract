#![no_std]

mod extensions;
mod test;
mod types;

use extensions::{
    env_extensions::EnvExtensions, i128_extensions::I128Extensions, u64_extensions::U64Extensions,
};
use soroban_sdk::token::TokenClient;
use soroban_sdk::{contractevent, panic_with_error, Address, BytesN, Env, Val, Vec};
use types::{asset::Asset, error::Error};
use types::{config_data::ConfigData, price_data::PriceData};

use crate::types::retention_config::RetentionConfig;

const INITIAL_EXPIRATION_PERIOD: u32 = 180; //6 months
const CURRENT_PROTOCOL: u32 = 2; //current protocol version

#[contractevent(topics = ["REFLECTOR", "update"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateEvent {
    #[topic]
    pub timestamp: u64,
    // Fields not marked as topics will appear in the events data section.
    pub update_data: Vec<(Val, i128)>
}

#[soroban_sdk::contract]
pub struct PriceOracleContract;

#[soroban_sdk::contractimpl]
impl PriceOracleContract {
    // Returns the base asset the price is reported in.
    //
    // # Returns
    //
    // Base asset for the contract
    pub fn base(e: &Env) -> Asset {
        e.get_base_asset()
    }

    // Returns the number of decimal places used to represent price for all assets quoted by the oracle.
    //
    // # Returns
    //
    // Number of decimals places in quoted prices
    pub fn decimals(e: &Env) -> u32 {
        e.get_decimals()
    }

    // Returns the default tick period timeframe (in seconds).
    //
    // # Returns
    //
    // Price feed resolution (in seconds)
    pub fn resolution(e: &Env) -> u32 {
        e.get_resolution() / 1000
    }

    // Returns the historical records retention period (in seconds).
    //
    // # Returns
    //
    // History retention period (in seconds)
    pub fn history_retention_period(e: &Env) -> Option<u64> {
        let period: u64 = e.get_history_retention_period();
        if period == 0 {
            return None;
        } else {
            return Some(period / 1000); //convert to seconds
        }
    }

    // Returns the cache size for the prices.
    //
    // # Returns
    //
    // Cache size for the prices
    pub fn cache_size(e: &Env) -> u32 {
        e.get_cache_size()
    }

    // Returns all assets quoted by the contract.
    //
    // # Returns
    //
    // Assets quoted by the contract
    pub fn assets(e: &Env) -> Vec<Asset> {
        e.get_assets()
    }

    // Returns the most recent price update timestamp in seconds.
    //
    // # Returns
    //
    // Timestamp of the last recorded price update
    pub fn last_timestamp(e: &Env) -> u64 {
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
    pub fn price(e: &Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        let resolution = e.get_resolution();
        let normalized_timestamp = //convert to milliseconds and normalize
            (timestamp * 1000).get_normalized_timestamp(resolution.into());
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
    pub fn lastprice(e: &Env, asset: Asset) -> Option<PriceData> {
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
    pub fn prices(e: &Env, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        let asset_index = e.get_asset_index(&asset)?; //get the asset index to avoid multiple calls
        prices(
            &e,
            |timestamp| get_price_data_by_index(e, asset_index, timestamp, &e.get_prices(timestamp)),
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
    pub fn x_last_price(e: &Env, base_asset: Asset, quote_asset: Asset) -> Option<PriceData> {
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
        e: &Env,
        base_asset: Asset,
        quote_asset: Asset,
        timestamp: u64,
    ) -> Option<PriceData> {
        let normalized_timestamp = //convert to milliseconds and normalize
            (timestamp * 1000).get_normalized_timestamp(e.get_resolution().into());
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
        e: &Env,
        base_asset: Asset,
        quote_asset: Asset,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        let asset_pair_indexes = get_asset_pair_indexes(&e, base_asset, quote_asset)?;
        let decimals = e.get_decimals();
        prices(
            &e,
            |timestamp| {
                get_x_price_by_indexes(&e, asset_pair_indexes, timestamp, decimals)
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
    pub fn twap(e: &Env, asset: Asset, records: u32) -> Option<i128> {
        let asset_index = e.get_asset_index(&asset)?; //get the asset index to avoid multiple calls
        get_twap(
            &e,
            |timestamp| get_price_data_by_index(e, asset_index, timestamp, &e.get_prices(timestamp)),
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
    pub fn x_twap(e: &Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<i128> {
        //get asset index to avoid multiple calls
        let asset_pair_indexes = get_asset_pair_indexes(&e, base_asset, quote_asset)?;
        let decimals = e.get_decimals();
        get_twap(
            &e,
            |timestamp| {
                get_x_price_by_indexes(&e, asset_pair_indexes, timestamp, decimals)
            },
            records,
        )
    }

    // Returns current protocol version of the contract.
    //
    // # Returns
    //
    // Contract protocol version
    pub fn version(_e: &Env) -> u32 {
        env!("CARGO_PKG_VERSION")
            .split(".")
            .next()
            .unwrap()
            .parse::<u32>()
            .unwrap()
    }

    // Returns the expiration date for a given asset.
    //
    // # Arguments
    //
    // * `asset` - Quoted asset
    //
    // # Returns
    //
    // Asset expiration timestamp or None if the asset is not supported
    //
    // # Panics
    //
    // Panics if the asset is not supported
    pub fn expires(e: &Env, asset: Asset) -> Option<u64> {
        let asset_index = e.get_asset_index(&asset);
        if asset_index.is_none() {
            e.panic_with_error(Error::AssetMissing);
        }
        let expirations = e.get_expiration();
        expirations.get(asset_index.unwrap() as u32)
    }

    // Extends the asset expiration date by a given amount of tokens.
    //
    // # Arguments
    //
    // * `sponsor` - Sponsor account address that burns tokens
    // * `asset` - Quoted asset
    // * `amount` - Amount of tokens to burn for extending the expiration date
    //
    // # Panics
    //
    // Panics if the asset is not supported, or if the fee token or fee itself are not set
    pub fn extend_asset_ttl(e: &Env, sponsor: Address, asset: Asset, amount: i128) {
        //check sponsor authorization
        sponsor.require_auth();
        //check if the amount is valid
        if amount <= 0 {
            e.panic_with_error(Error::InvalidAmount);
        }
        //ensure that the asset is supported
        let asset_index = e.get_asset_index(&asset);
        if asset_index.is_none() {
            e.panic_with_error(Error::AssetMissing);
        }
        let asset_index = asset_index.unwrap() as u32;
        
        let (token, fee) = match e.get_retention_config() {
            RetentionConfig::Some(fee_data) => {
                if fee_data.1 <= 0 {
                    e.panic_with_error(Error::InvalidConfigVersion);
                }
                fee_data
            }
            RetentionConfig::None => {
                e.panic_with_error(Error::InvalidConfigVersion);
            }
        };

        //get minutes to extend
        let bump = amount * 86400000 / fee; // result in milliseconds
        if bump <= 0 {
            e.panic_with_error(Error::InvalidAmount);
        }

        //burn the corresponding amount of fee tokens
        TokenClient::new(&e, &token).burn(&sponsor, &amount);

        //load expiration info
        let mut expiration = e.get_expiration();
        let now = now(&e);
        let mut asset_expiration = expiration
            .get(asset_index)
            .unwrap_or_else(|| now + days_to_milliseconds(INITIAL_EXPIRATION_PERIOD));
        //if the asset expiration is not set, or it's already expired - set it to now
        if asset_expiration == 0 || asset_expiration < now {
            asset_expiration = now;
        }
        //bump expiration
        asset_expiration = asset_expiration
            .checked_add(bump as u64)
            .unwrap();
        //write into the vector that holds expiration dates for all symbols
        expiration.set(asset_index, asset_expiration);
        //update instance
        e.set_expiration(&expiration)
    }

    // Returns the fee token address and daily retainer fee amount.
    //
    // # Returns
    //
    // Fee token address and daily price feed retainer fee amount
    pub fn retention_config(e: &Env) -> RetentionConfig {
        e.get_retention_config()
    }

    // Returns admin address of the contract.
    //
    // # Returns
    //
    // Contract admin account address
    pub fn admin(e: &Env) -> Option<Address> {
        e.get_admin()
    }

    //Admin section

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
    pub fn config(e: &Env, config: ConfigData) {
        config.admin.require_auth();
        if e.is_initialized() {
            e.panic_with_error(Error::AlreadyInitialized);
        }
        e.set_admin(&config.admin);
        e.set_base_asset(&config.base_asset);
        e.set_decimals(config.decimals);
        e.set_resolution(config.resolution);
        e.set_history_retention_period(config.history_retention_period);
        e.set_cache_size(config.cache_size);
        e.set_retention_config(config.retention_config);
        //set protocol version to current
        e.set_protocol_version(CURRENT_PROTOCOL);
        //add assets
        add_assets(&e, config.assets);
    }

    pub fn set_cache_size(e: &Env, cache_size: u32) {
        e.panic_if_not_admin();
        e.set_cache_size(cache_size);
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
    pub fn add_assets(e: &Env, assets: Vec<Asset>) {
        e.panic_if_not_admin();
        add_assets(&e, assets);
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
    pub fn set_history_retention_period(e: &Env, period: u64) {
        e.panic_if_not_admin();
        e.set_history_retention_period(period);
    }

    // Sets the fee token address and daily asset feed retainer fee amount.
    // Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `fee_config` - Fee token address and fee amount
    //
    // # Panics
    //
    // Panics if the caller doesn't match admin address, or not initialized yet
    pub fn set_retention_config(e: &Env, retention_config: RetentionConfig) {
        e.panic_if_not_admin();
        e.set_retention_config(retention_config);
        let mut expiration = e.get_expiration();
        if expiration.len() > 0 {
            return; // expiration values for existing price feeds already initialized
        }
        //init expiration, set INITIAL_EXPIRATION_PERIOD for all symbols by default
        let exp = now(&e)
            .checked_add(days_to_milliseconds(INITIAL_EXPIRATION_PERIOD))
            .unwrap();
        let assets = e.get_assets();
        for _ in 0..assets.len() {
            expiration.push_back(exp);
        }
        e.set_expiration(&expiration);
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
    pub fn set_price(e: &Env, updates: Vec<i128>, timestamp: u64) {
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

        let retention_period = e.get_history_retention_period();

        let ledgers_to_live = ((retention_period / 1000 / 5 + 1) * 2) as u32;

        update_price_v1(&e, &updates, timestamp, ledger_timestamp, ledgers_to_live);

        //get the last timestamp
        let last_timestamp = e.get_last_timestamp();

        //store new prices in v2 format
        e.set_prices(&updates, timestamp, ledgers_to_live);

        //update the cache
        let cache_size = e.get_cache_size();
        if cache_size > 0 { //if cache size is non-empty, store it in the instance
            let mut cache = e.get_cache().unwrap_or(Vec::new(&e));
            cache.push_front((timestamp, updates.clone()));
            while cache.len() > cache_size {
                cache.pop_back(); //remove the oldest record if cache size exceeded
            }
            e.set_cache(cache);
        }

        //update the last timestamp
        if timestamp > last_timestamp {
            e.set_last_timestamp(timestamp);
        }

        //load all registered assets
        let assets = e.get_assets();
        //event updates
        let mut event_updates = Vec::new(&e);
        for (index, asset) in assets.iter().enumerate() {
            let price = updates.get(index as u32).unwrap_or(0i128);
            if price == 0 {
                continue; //skip zero prices
            }
            let symbol = match asset {
                Asset::Stellar(address) => {
                    address.to_val()
                },
                Asset::Other(symbol) => {
                    symbol.to_val()
                }
            };
            event_updates.push_back((symbol, price));
        }

        //publish the price updates
        let event = UpdateEvent {
            timestamp,
            update_data: event_updates,
        };
        e.events().publish_event(&event);
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
    pub fn update_contract(env: &Env, wasm_hash: BytesN<32>) {
        env.panic_if_not_admin();
        env.deployer().update_current_contract_wasm(wasm_hash);
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
    let asset_pair_indexes = get_asset_pair_indexes(e, base_asset, quote_asset)?;
    get_x_price_by_indexes(e, asset_pair_indexes, timestamp, decimals)
}

fn get_x_price_by_indexes(
    e: &Env,
    asset_pair_indexes: (u32, u32),
    timestamp: u64,
    decimals: u32,
) -> Option<PriceData> {
    //get the asset indexes
    let (base_asset, quote_asset) = asset_pair_indexes;
    //check if the asset are the same
    if base_asset == quote_asset {
        return Some(get_normalized_price_data(10i128.pow(decimals), timestamp));
    }
    
    let prices = e.get_prices(timestamp);

    //get the price for base_asset
    let base_asset_price = get_price_data_by_index(e, base_asset, timestamp, &prices)?;

    //get the price for quote_asset
    let quote_asset_price = get_price_data_by_index(e, quote_asset, timestamp, &prices)?;

    //calculate the cross price
    Some(get_normalized_price_data(
        base_asset_price
            .price
            .fixed_div_floor(quote_asset_price.price, decimals),
        timestamp,
    ))
}

fn get_asset_pair_indexes(e: &Env, base_asset: Asset, quote_asset: Asset) -> Option<(u32, u32)> {
    let base_asset = e.get_asset_index(&base_asset)?;

    let quote_asset = e.get_asset_index(&quote_asset)?;

    Some((base_asset, quote_asset))
}

fn get_price_data(e: &Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
    let asset = e.get_asset_index(&asset)?;
    get_price_data_by_index(e, asset, timestamp, &e.get_prices(timestamp))
}

fn get_price_data_by_index(e: &Env, asset: u32, timestamp: u64, prices: &Option<Vec<i128>>) -> Option<PriceData> {
    //if the protocol version is not current, use legacy method
    if !is_current_protocol_version(e, now(e)) {
        let price = e.get_price_v1(asset as u8, timestamp)?;
        return Some(get_normalized_price_data(price, timestamp));
    }
    if prices.is_none() {
        return None;
    }
    let asset = asset as u32;
    let prices = prices.as_ref().unwrap();
    if prices.len() <= asset {
        return None;
    }
    let price = prices.get(asset)?;
    if price == 0 {
        return None;
    }
    Some(get_normalized_price_data(price, timestamp))
}

fn get_normalized_price_data(price: i128, timestamp: u64) -> PriceData {
    PriceData {
        price,
        timestamp: timestamp / 1000, //convert to seconds
    }
}

fn add_assets(e: &Env, assets: Vec<Asset>) {
    //use default expiration period for new assets
    let expiration_timestamp = now(&e)
        .checked_add(days_to_milliseconds(INITIAL_EXPIRATION_PERIOD))
        .unwrap();
    let mut current_assets = e.get_assets();
    let mut expiration = e.get_expiration();
    let is_retention_config_set = e.get_retention_config() != RetentionConfig::None;
    for asset in assets.iter() {
        //check if the asset has been already added
        if e.get_asset_index(&asset).is_some() {
            panic_with_error!(&e, Error::AssetAlreadyExists);
        }
        e.set_asset_index(&asset, current_assets.len());
        current_assets.push_back(asset);

        //if the fee is not set, we don't need to set the expiration
        if is_retention_config_set {
            expiration.push_back(expiration_timestamp); //set expiration
        }
    }
    if current_assets.len() >= 256 {
        panic_with_error!(&e, Error::AssetLimitExceeded);
    }
    e.set_assets(current_assets);
    e.set_expiration(&expiration);
}

fn days_to_milliseconds(days: u32) -> u64 {
    (days as u64) * 24 * 60 * 60 * 1000 //convert to milliseconds
}

fn is_current_protocol_version(e: &Env, now: u64) -> bool {
    let protocol = e.get_protocol_version();
    if protocol == CURRENT_PROTOCOL {
        return true;
    }
    let update_ts = e.get_update_ts();
    if update_ts == 0 {
        e.set_update_ts(now); //set update timestamp to now if not set
        return false;
    } else if update_ts + days_to_milliseconds(1) < now {
        e.set_protocol_version(CURRENT_PROTOCOL); //set protocol to current if the update timestamp is older than 1 day
        e.set_update_ts(0); // reset update timestamp
        return true;
    }
    false
}

fn update_price_v1(e: &Env, updates: &Vec<i128>, timestamp: u64, ledger_timestamp: u64, ledgers_to_live: u32) {
    //if the protocol version is current, we can skip the legacy update
    if !is_current_protocol_version(e, ledger_timestamp) {
        return;
    }
    //iterate over the updates
    for (i, price) in updates.iter().enumerate() {
        //don't store zero prices
        if price == 0 {
            continue;
        }
        let asset = i as u8;
        //store the new price
        e.set_price_v1(asset, price, timestamp, ledgers_to_live);
    }
}