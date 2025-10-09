#![no_std]

mod assets;
mod auth;
mod events;
mod prices;
mod protocol;
mod settings;
mod test;
mod timestamps;
mod types;

use soroban_sdk::{contractevent, panic_with_error, Address, BytesN, Env, Val, Vec};
use types::{asset::Asset, error::Error, retention_config::RetentionConfig};
use types::{config_data::ConfigData, price_data::PriceData};

const CURRENT_PROTOCOL: u32 = 2; //current protocol version

#[contractevent(topics = ["REFLECTOR", "update"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateEvent {
    #[topic]
    pub timestamp: u64,
    pub update_data: Vec<(Val, i128)>,
}

#[soroban_sdk::contract]
pub struct PriceOracleContract;

#[soroban_sdk::contractimpl]
impl PriceOracleContract {
    // Return base asset price is reported in
    //
    // # Returns
    //
    // Oracle base asset
    pub fn base(e: &Env) -> Asset {
        settings::get_base_asset(e)
    }

    // Return number of decimal places used to represent price for all quoted assets
    //
    // # Returns
    //
    // Number of decimals places in quoted prices
    pub fn decimals(e: &Env) -> u32 {
        settings::get_decimals(e)
    }

    // Return default tick period timeframe (in seconds)
    //
    // # Returns
    //
    // Price feed resolution (in seconds)
    pub fn resolution(e: &Env) -> u32 {
        settings::get_resolution(e) / 1000
    }

    // Return historical records retention period (in seconds)
    //
    // # Returns
    //
    // History retention period (in seconds)
    pub fn history_retention_period(e: &Env) -> Option<u64> {
        let period: u64 = settings::get_history_retention_period(e);
        if period == 0 {
            None
        } else {
            Some(period / 1000) //convert to seconds
        }
    }

    // Return price records cache size
    //
    // # Returns
    //
    // Price records cache size
    pub fn cache_size(e: &Env) -> u32 {
        settings::get_cache_size(e)
    }

    // Return all quoted assets
    //
    // # Returns
    //
    // Quoted assets
    pub fn assets(e: &Env) -> Vec<Asset> {
        assets::load_all_assets(e)
    }

    // Return most recent price update timestamp in seconds
    //
    // # Returns
    //
    // Timestamp of last recorded price update
    pub fn last_timestamp(e: &Env) -> u64 {
        prices::get_last_timestamp(e) / 1000 //convert to seconds
    }

    // Returns price  for an asset at specific timestamp
    //
    // # Arguments
    //
    // * `asset` - Asset to quote
    // * `timestamp` - Timestamp in seconds
    //
    // # Returns
    //
    // Price record for given asset at given timestamp or None if not found
    pub fn price(e: &Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        //normalize timestamp
        let ts = timestamps::normalize(e, timestamp * 1000);
        //resolve index for the asset
        let asset = assets::resolve_asset_index(e, &asset)?;
        prices::retrieve_asset_price_data(e, asset, ts)
    }

    // Returns most recent price for an asset
    //
    // # Arguments
    //
    // * `asset` - Asset to quote
    //
    // # Returns
    //
    // Most recent price for given asset or None if asset is not supported
    pub fn lastprice(e: &Env, asset: Asset) -> Option<PriceData> {
        //get the last timestamp
        let ts = prices::obtain_last_record_timestamp(&e);
        if ts == 0 {
            return None;
        }
        //get the price
        let asset = assets::resolve_asset_index(e, &asset)?;
        //resolve index for the asset
        prices::retrieve_asset_price_data(e, asset, ts)
    }

    // Return last N price records for given asset
    //
    // # Arguments
    //
    // * `asset` - Asset to quote
    // * `records` - Number of records to return
    //
    // # Returns
    //
    // Prices for given asset or None if asset is not supported
    pub fn prices(e: &Env, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        let asset_index = assets::resolve_asset_index(e, &asset)?; //get the asset index to avoid multiple calls
        prices::load_prices(
            &e,
            |timestamp| prices::retrieve_asset_price_data(e, asset_index, timestamp),
            records,
        )
    }

    // Returns most recent cross price record for pair of assets
    //
    // # Arguments
    //
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    //
    // # Returns
    //
    // Recent cross price (base_asset_price/quote_asset_price) for given assets or None if there were no records found
    pub fn x_last_price(e: &Env, base_asset: Asset, quote_asset: Asset) -> Option<PriceData> {
        let timestamp = prices::obtain_last_record_timestamp(&e);
        if timestamp == 0 {
            return None;
        }
        let decimals = settings::get_decimals(e);
        let asset_pair_indexes = assets::resolve_asset_pair_indexes(e, base_asset, quote_asset)?;
        prices::load_cross_price(&e, asset_pair_indexes, timestamp, decimals)
    }

    // Return cross price for pair of assets at specific timestamp
    //
    // # Arguments
    //
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    // * `timestamp` - Timestamp
    //
    // # Returns
    //
    // Cross price (base_asset_price/quote_asset_price) at given timestamp or None if there were no records found for quoted assets
    pub fn x_price(
        e: &Env,
        base_asset: Asset,
        quote_asset: Asset,
        timestamp: u64,
    ) -> Option<PriceData> {
        //convert to milliseconds and normalize
        let ts = timestamps::normalize(e, timestamp * 1000);
        let decimals = settings::get_decimals(e);
        let asset_pair_indexes = assets::resolve_asset_pair_indexes(e, base_asset, quote_asset)?;
        prices::load_cross_price(e, asset_pair_indexes, ts, decimals)
    }

    // Returns last N cross price records of for pair of assets
    //
    // # Arguments
    //
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    // * `records` - Number of records to fetch
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
        let asset_pair_indexes = assets::resolve_asset_pair_indexes(&e, base_asset, quote_asset)?;
        let decimals = settings::get_decimals(e);
        prices::load_prices(
            &e,
            |timestamp| prices::load_cross_price(&e, asset_pair_indexes, timestamp, decimals),
            records,
        )
    }

    // Returns time-weighted average price for given asset over N recent records
    //
    // # Arguments
    //
    // * `asset` - Asset to quote
    // * `records` - Number of records to process
    //
    // # Returns
    //
    // TWAP for the given asset over N recent records or None if asset is not supported
    pub fn twap(e: &Env, asset: Asset, records: u32) -> Option<i128> {
        let asset_index = assets::resolve_asset_index(e, &asset)?; //get the asset index to avoid multiple calls
        prices::calculate_twap(
            &e,
            |timestamp| prices::retrieve_asset_price_data(e, asset_index, timestamp),
            records,
        )
    }

    // Returns time-weighted average cross price for given asset pair over N recent records
    //
    // # Arguments
    //
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    // * `records` - Number of records to process
    //
    // # Returns
    //
    // TWAP (base_asset_price/quote_asset_price) or None if assets are not supported
    pub fn x_twap(e: &Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<i128> {
        //get asset index to avoid multiple calls
        let asset_pair_indexes = assets::resolve_asset_pair_indexes(&e, base_asset, quote_asset)?;
        let decimals = settings::get_decimals(e);
        prices::calculate_twap(
            &e,
            |timestamp| prices::load_cross_price(&e, asset_pair_indexes, timestamp, decimals),
            records,
        )
    }

    // Return current contract protocol version
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

    // Return expiration date for a given asset
    //
    // # Arguments
    //
    // * `asset` - Quoted asset
    //
    // # Returns
    //
    // Asset expiration timestamp or None if asset is not supported
    //
    // # Panics
    //
    // Panics if asset is not supported
    pub fn expires(e: &Env, asset: Asset) -> Option<u64> {
        assets::expires(e, asset)
    }

    // Extends the asset expiration date by a given amount of tokens.
    //
    // # Arguments
    //
    // * `sponsor` - Address that sponsors price feed
    // * `asset` - Quoted asset
    // * `amount` - Amount of tokens to burn for extending the expiration date
    //
    // # Panics
    //
    // Panics if the asset is not supported or if retention config is malformed/missing
    pub fn extend_asset_ttl(e: &Env, sponsor: Address, asset: Asset, amount: i128) {
        //check sponsor authorization
        sponsor.require_auth();
        assets::extend_ttl(e, sponsor, asset, amount);
    }

    // Return the fee token address daily price feed retainer fee amount
    //
    // # Returns
    //
    // Fee token address and daily price feed retainer fee amount
    pub fn retention_config(e: &Env) -> RetentionConfig {
        settings::get_retention_config(e)
    }

    // Return contract admin address
    //
    // # Returns
    //
    // Contract admin account address
    pub fn admin(e: &Env) -> Option<Address> {
        auth::get_admin(e)
    }

    /* Admin section */

    // Initializes contract configuration
    // Requires admin authorization
    // # Arguments
    //
    // * `config` - Configuration parameters
    //
    // # Panics
    //
    // Panics if not authorized or if contract is already initialized
    pub fn config(e: &Env, config: ConfigData) {
        //should be invoked by admin
        config.admin.require_auth();
        //apply settings
        settings::init(e, &config);
        auth::set_admin(e, &config.admin);
        protocol::set_protocol_version(e, CURRENT_PROTOCOL);
        //add initial assets
        assets::add_assets(&e, config.assets);
    }

    // Update contract cache size
    // Requires admin authorization
    //
    // # Arguments
    //
    // * `cache_size` - New cache size (number of rounds stored in cache)
    //
    // # Panics
    //
    // Panics if not authorized
    pub fn set_cache_size(e: &Env, cache_size: u32) {
        auth::panic_if_not_admin(e);
        settings::set_cache_size(e, cache_size);
    }

    // Adds given assets to the contract quoted assets list
    // Requires admin authorization
    //
    // # Arguments
    //
    // * `assets` - Assets to add
    //
    // # Panics
    //
    // Panics if not authorized, any of the assets were added earlier, or assets limit exceeded
    pub fn add_assets(e: &Env, assets: Vec<Asset>) {
        auth::panic_if_not_admin(e);
        assets::add_assets(&e, assets);
    }

    // Sets history retention period for the prices
    // Requires admin authorization
    //
    // # Arguments
    //
    // * `period` - History retention period (in seconds)
    //
    // # Panics
    //
    // Panics if not authorized
    pub fn set_history_retention_period(e: &Env, period: u64) {
        auth::panic_if_not_admin(e);
        settings::set_history_retention_period(e, period);
    }

    // Set fee token address and daily price feed retainer fee amount
    // Requires admin authorization
    //
    // # Arguments
    //
    // * `fee_config` - Fee token address and fee amount
    //
    // # Panics
    //
    // Panics if not authorized or not initialized yet
    pub fn set_retention_config(e: &Env, retention_config: RetentionConfig) {
        auth::panic_if_not_admin(e);
        settings::set_retention_config(e, &retention_config);
        assets::init_expiration_config(e);
    }

    // Record new price feed history snapshot
    // Requires admin authorization
    //
    // # Arguments
    //
    // * `updates` - Price feed snapshot
    // * `timestamp` - History snapshot timestamp
    //
    // # Panics
    //
    // Panics if not authorized or price snapshot record is invalid
    pub fn set_price(e: &Env, updates: Vec<i128>, timestamp: u64) {
        auth::panic_if_not_admin(e);
        if updates.len() == 0 {
            return; //skip empty updates
        }
        //validate record timestamp
        let ledger_timestamp = timestamps::ledger_timestamp(&e);
        if timestamp == 0 || !timestamps::is_valid(e, timestamp) || timestamp > ledger_timestamp {
            panic_with_error!(&e, Error::InvalidTimestamp);
        }
        //prepare and publish update event
        events::publish_update_event(e, &updates, timestamp);
        //store new prices
        prices::store_prices(e, &updates, timestamp);
    }

    // Update contract source code
    // Requires admin authorization
    //
    // # Arguments
    //
    // * `wasm_hash` - WASM hash of the contract source code
    //
    // # Panics
    //
    // Panics if not authorized
    pub fn update_contract(e: &Env, wasm_hash: BytesN<32>) {
        auth::panic_if_not_admin(e);
        e.deployer().update_current_contract_wasm(wasm_hash);
    }
}
