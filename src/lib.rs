#![no_std]

mod extensions;
mod test;
mod types;

use extensions::{
    env_extensions::EnvExtensions, i128_extensions::I128Extensions, u64_extensions::U64Extensions,
};
use soroban_sdk::token::TokenClient;
use soroban_sdk::{panic_with_error, symbol_short, Address, BytesN, Env, Symbol, Vec};
use types::{asset::Asset, error::Error};
use types::{config_data::ConfigData, price_data::PriceData};

const REFLECTOR: Symbol = symbol_short!("reflector");
const DEFAULT_EXPIRATION_PERIOD: u32 = 365; //days in year

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
    pub fn period(e: &Env) -> Option<u64> {
        let period = e.get_retention_period();
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
        let asset_index = e.get_asset_index(&asset); //get the asset index to avoid multiple calls
        if asset_index.is_none() {
            return None;
        }
        if !is_legacy_expired(&e, now(&e)) {
            return prices(
                &e,
                |timestamp| get_price_data_by_index_legacy(e, asset_index.unwrap() as u8, timestamp),
                records,
            );
        }
        prices(
            &e,
            |timestamp| get_price_data_by_index(asset_index.unwrap(), timestamp, &e.get_prices(timestamp)),
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
    pub fn twap(e: &Env, asset: Asset, records: u32) -> Option<i128> {
        let asset_index = e.get_asset_index(&asset); //get the asset index to avoid multiple calls
        if asset_index.is_none() {
            return None;
        }

        if !is_legacy_expired(&e, now(&e)) {
            return get_twap(
                &e,
                |timestamp| get_price_data_by_index_legacy(e, asset_index.unwrap() as u8, timestamp),
                records,
            );
        }

        get_twap(
            &e,
            |timestamp| get_price_data_by_index(asset_index.unwrap(), timestamp, &e.get_prices(timestamp)),
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
    pub fn expires(e: &Env, asset: Asset) -> Option<u64> {
        let asset_index = e.get_asset_index(&asset);
        if asset_index.is_none() {
            e.panic_with_error(Error::AssetMissing);
        }
        let expirations = e.get_expiration();
        let asset_index = asset_index.unwrap() as u32;
        expirations.get(asset_index)
    }

    // Extends the asset expiration date by a given number of days.
    //
    // # Arguments
    //
    // * `sponsor` - Sponsor account address that burns tokens
    // * `asset` - Quoted asset
    // * `days` - Number of days to add
    //
    // # Panics
    //
    // Panics if the asset is not supported, or if the fee token or fee itself are not set
    pub fn extend(e: &Env, sponsor: Address, asset: Asset, days: u32) {
        //check sponsor authorization
        sponsor.require_auth();
        //ensure that the asset is supported
        let asset_index = e.get_asset_index(&asset);
        if asset_index.is_none() {
            e.panic_with_error(Error::AssetMissing);
        }
        let asset_index = asset_index.unwrap() as u32;
        //ensure that the fee token and fee are set
        let fee_data = e.get_retention_config();
        if fee_data.is_none() {
            e.panic_with_error(Error::InvalidConfigVersion);
        }
        let (fee_token, fee) = fee_data.unwrap();

        //calculate amount of tokens to charge
        let charge = fee.checked_mul(days.into()).unwrap();
        if charge == 0 {
            return;
        }

        //burn the corresponding amount of fee tokens
        TokenClient::new(&e, &fee_token).burn(&sponsor, &charge);

        //load expiration info
        let mut expiration = e.get_expiration();
        let mut asset_expiration = expiration.get(asset_index).unwrap_or_default();
        let now = now(&e);
        //if the asset expiration is not set, or it's already expired - set it to now
        if asset_expiration == 0 || asset_expiration < now {
            asset_expiration = now;
        }
        //bump expiration
        asset_expiration = asset_expiration
            .checked_add(days_to_milliseconds(days))
            .unwrap();
        //write the vector that holds expiration dates for all symbols
        expiration.set(asset_index, asset_expiration);
        //update instance
        e.set_expiration(&expiration)
    }

    // Estimates the cost of asset retention bump
    //
    // # Arguments
    //
    // * `days` - Number of days
    //
    // # Returns
    //
    // Amount that will be charged for the expiration bump for a given number of days
    //
    // # Panics
    //
    // Panics if the retention config hasn't been initialized
    pub fn estimate_extend(e: &Env, days: u32) -> i128 {
        let fee_data = e.get_retention_config();
        if fee_data.is_none() {
            e.panic_with_error(Error::InvalidConfigVersion);
        }
        let (_, fee) = fee_data.unwrap();

        fee.checked_mul(days.into()).unwrap()
    }

    // Returns the fee token address and daily retainer fee amount.
    //
    // # Returns
    //
    // Fee token address and daily price feed retainer fee amount
    pub fn retention_config(e: &Env) -> Option<(Address, i128)> {
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
        e.set_retention_period(config.period);
        e.set_cache_size(config.cache_size);
        //set update timestamp to 1 to indicate that contract is already v2
        e.set_v2_update_ts(1);

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
    pub fn set_period(e: &Env, period: u64) {
        e.panic_if_not_admin();
        e.set_retention_period(period);
    }

    // Sets the fee token address and daily retainer fee amount.
    // Can be invoked only by the admin account.
    //
    // # Arguments
    //
    // * `fee_config` - Fee token address and fee amount
    //
    // # Panics
    //
    // Panics if the caller doesn't match admin address, or not initialized yet
    pub fn set_retention_config(e: &Env, retention_config: (Address, i128)) {
        e.panic_if_not_admin();
        e.set_retention_config(retention_config);
        let mut expiration = e.get_expiration();
        if expiration.len() > 0 {
            return; // expiration values for existing price feeds already initialized
        }
        //init expiration, set 365 days for all symbols by default
        let exp = now(&e)
            .checked_add(days_to_milliseconds(DEFAULT_EXPIRATION_PERIOD))
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

        let retention_period = e.get_retention_period();

        //TODO: compute ledgers per second, for now we assume 5 seconds per ledger
        let ledgers_to_live: u32 = ((retention_period / 1000 / 5) + 1) as u32;

        ensure_v2_update_ts(&e);
        //update legacy for 24 hours after v2 update
        if !is_legacy_expired(&e, timestamp) {
            update_price_legacy(&e, &updates, timestamp, ledgers_to_live);
        }

        //get the last timestamp
        let last_timestamp = e.get_last_timestamp();

        //store the new prices
        e.set_prices(&updates, timestamp, ledgers_to_live);

        //update the cache
        let mut cache = e.get_cache().unwrap_or(Vec::new(&e));
        let cache_size = e.get_cache_size();
        cache.push_front((timestamp, updates.clone()));
        while cache.len() > cache_size {
            cache.pop_back(); //remove the oldest record if cache size exceeded
        }
        if cache_size > 0 { //if cache size is set, update the cache
            e.set_cache(cache);
        }

        //update the last timestamp
        if timestamp > last_timestamp {
            e.set_last_timestamp(timestamp);
        }

        //publish the price updates
        e.events().publish(
            (REFLECTOR, symbol_short!("prices"), symbol_short!("update")),
            updates,
        );
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
        env.deployer().update_current_contract_wasm(wasm_hash)
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
    let asset_pair_indexes = get_asset_pair_indexes(e, base_asset, quote_asset);
    if asset_pair_indexes.is_none() {
        return None;
    }
    get_x_price_by_indexes(e, asset_pair_indexes.unwrap(), timestamp, decimals)
}

fn get_x_price_by_indexes(
    e: &Env,
    asset_pair_indexes: (u32, u32),
    timestamp: u64,
    decimals: u32,
) -> Option<PriceData> {

    if !is_legacy_expired(e, now(e)) {
        return get_x_price_by_indexes_legacy(e, asset_pair_indexes, timestamp, decimals);
    }

    let (base_asset, quote_asset) = asset_pair_indexes;
    //check if the asset are the same
    if base_asset == quote_asset {
        return Some(get_normalized_price_data(10i128.pow(decimals), timestamp));
    }
    
    let prices = e.get_prices(timestamp);

    //get the price for base_asset
    let base_asset_price = get_price_data_by_index(base_asset, timestamp, &prices);
    if base_asset_price.is_none() {
        return None;
    }

    //get the price for quote_asset
    let quote_asset_price = get_price_data_by_index(quote_asset, timestamp, &prices);
    if quote_asset_price.is_none() {
        return None;
    }

    //calculate the cross price
    Some(get_normalized_price_data(
        base_asset_price
            .unwrap()
            .price
            .fixed_div_floor(quote_asset_price.unwrap().price, decimals),
        timestamp,
    ))
}

fn get_asset_pair_indexes(e: &Env, base_asset: Asset, quote_asset: Asset) -> Option<(u32, u32)> {
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
    let asset: Option<u32> = e.get_asset_index(&asset);
    if asset.is_none() {
        return None;
    }
    if !is_legacy_expired(e, now(e)) {
        return get_price_data_by_index_legacy(e, asset.unwrap() as u8, timestamp);
    }
    get_price_data_by_index(asset.unwrap(), timestamp, &e.get_prices(timestamp))
}

fn get_price_data_by_index(asset: u32, timestamp: u64, prices: &Option<Vec<i128>>) -> Option<PriceData> {
    if prices.is_none() {
        return None;
    }
    let asset = asset as u32;
    let prices = prices.as_ref().unwrap();
    if prices.len() <= asset {
        return None;
    }
    let price = prices.get_unchecked(asset);
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
        .checked_add(days_to_milliseconds(DEFAULT_EXPIRATION_PERIOD))
        .unwrap();
    let mut current_assets = e.get_assets();
    let mut expiration = e.get_expiration();
    let retention_config_initialized = e.get_retention_config().is_some();
    for asset in assets.iter() {
        //check if the asset has been already added
        if e.get_asset_index(&asset).is_some() {
            panic_with_error!(&e, Error::AssetAlreadyExists);
        }
        e.set_asset_index(&asset, current_assets.len());
        current_assets.push_back(asset);

        //if the fee is not initialized, we don't need to set the expiration
        if retention_config_initialized {
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

fn ensure_v2_update_ts(e: &Env) {
    //ensure that the v2 update timestamp is set
    if e.get_v2_update_ts() == 0 {
        e.set_v2_update_ts(now(e));
    }
}

fn is_legacy_expired(e: &Env, timestamp: u64) -> bool {
    e.get_v2_update_ts() + days_to_milliseconds(1) < timestamp
}

fn update_price_legacy(e: &Env, updates: &Vec<i128>, timestamp: u64, ledgers_to_live: u32) {
    let assets = e.get_assets();
    let mut asset_prices: Vec<(Asset, i128)> = Vec::new(&e);
    //iterate over the updates
    for (i, price) in updates.iter().enumerate() {
        let asset = assets.get(i as u32);
        if asset.is_some() {
            //asset can be None if the asset was added but the update is not applied yet
            asset_prices.push_back((asset.unwrap(), price));
        }
        //don't store zero prices
        if price == 0 {
            continue;
        }
        let asset = i as u8;
        //store the new price
        e.set_price(asset, price, timestamp, ledgers_to_live);
    }
}

fn get_price_data_by_index_legacy(e: &Env, asset: u8, timestamp: u64) -> Option<PriceData> {
    let price = e.get_price(asset, timestamp);
    if price.is_none() {
        return None;
    }
    Some(get_normalized_price_data(price.unwrap(), timestamp))
}

fn get_x_price_by_indexes_legacy(
    e: &Env,
    asset_pair_indexes: (u32, u32),
    timestamp: u64,
    decimals: u32,
) -> Option<PriceData> {
    let (base_asset, quote_asset) = asset_pair_indexes;

    //get the price for base_asset
    let base_asset_price = get_price_data_by_index_legacy(e, base_asset as u8, timestamp);
    if base_asset_price.is_none() {
        return None;
    }

    //get the price for quote_asset
    let quote_asset_price = get_price_data_by_index_legacy(e, quote_asset as u8, timestamp);
    if quote_asset_price.is_none() {
        return None;
    }

    //calculate the cross price
    Some(get_normalized_price_data(
        base_asset_price
            .unwrap()
            .price
            .fixed_div_floor(quote_asset_price.unwrap().price, decimals),
        timestamp,
    ))
}