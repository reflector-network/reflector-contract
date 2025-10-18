#![no_std]
mod tests;

use oracle::price_oracle::PriceOracleContractBase;
use oracle::types::{Asset, ConfigData, FeeConfig, PriceData, PriceUpdate};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

const INITIAL_EXPIRATION_PERIOD: u32 = 180; //6 months
#[contract]
pub struct PulseOracleContract;

#[contractimpl]
impl PulseOracleContract {
    // Return base asset price is reported in
    //
    // # Returns
    //
    // Oracle base asset
    pub fn base(e: &Env) -> Asset {
        PriceOracleContractBase::base(e)
    }

    // Return number of decimal places used to represent price for all quoted assets
    //
    // # Returns
    //
    // Number of decimals places in quoted prices
    pub fn decimals(e: &Env) -> u32 {
        PriceOracleContractBase::decimals(e)
    }

    // Return default tick period timeframe (in seconds)
    //
    // # Returns
    //
    // Price feed resolution (in seconds)
    pub fn resolution(e: &Env) -> u32 {
        PriceOracleContractBase::resolution(e)
    }

    // Return historical records retention period (in seconds)
    //
    // # Returns
    //
    // History retention period (in seconds)
    pub fn history_retention_period(e: &Env) -> Option<u64> {
        PriceOracleContractBase::history_retention_period(e)
    }

    // Return price records cache size
    //
    // # Returns
    //
    // Price records cache size
    pub fn cache_size(e: &Env) -> u32 {
        PriceOracleContractBase::cache_size(e)
    }

    // Return all quoted assets
    //
    // # Returns
    //
    // Quoted assets
    pub fn assets(e: &Env) -> Vec<Asset> {
        PriceOracleContractBase::assets(e)
    }

    // Return most recent price update timestamp in seconds
    //
    // # Returns
    //
    // Timestamp of last recorded price update
    pub fn last_timestamp(e: &Env) -> u64 {
        PriceOracleContractBase::last_timestamp(e)
    }

    // Return current contract protocol version
    //
    // # Returns
    //
    // Contract protocol version
    pub fn version(e: &Env) -> u32 {
        PriceOracleContractBase::version(e)
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
        PriceOracleContractBase::expires(e, asset)
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
        PriceOracleContractBase::extend_asset_ttl(
            e,
            sponsor,
            asset,
            amount,
            INITIAL_EXPIRATION_PERIOD,
        );
    }

    // Return the fee token address daily price feed retainer fee amount
    //
    // # Returns
    //
    // Fee token address and daily price feed retainer fee amount
    pub fn fee_config(e: &Env) -> FeeConfig {
        PriceOracleContractBase::fee_config(e)
    }

    // Return contract admin address
    //
    // # Returns
    //
    // Contract admin account address
    pub fn admin(e: &Env) -> Option<Address> {
        PriceOracleContractBase::admin(e)
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
        PriceOracleContractBase::price(e, asset, timestamp)
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
        PriceOracleContractBase::lastprice(e, asset)
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
        PriceOracleContractBase::prices(e, asset, records)
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
        PriceOracleContractBase::x_last_price(e, base_asset, quote_asset)
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
        PriceOracleContractBase::x_price(e, base_asset, quote_asset, timestamp)
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
        PriceOracleContractBase::x_prices(e, base_asset, quote_asset, records)
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
        PriceOracleContractBase::twap(e, asset, records)
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
        PriceOracleContractBase::x_twap(e, base_asset, quote_asset, records)
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
        PriceOracleContractBase::config(e, config, INITIAL_EXPIRATION_PERIOD);
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
        PriceOracleContractBase::set_cache_size(e, cache_size);
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
        PriceOracleContractBase::add_assets(e, assets, INITIAL_EXPIRATION_PERIOD);
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
        PriceOracleContractBase::set_history_retention_period(e, period);
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
    pub fn set_fee_config(e: &Env, fee_config: FeeConfig) {
        PriceOracleContractBase::set_fee_config(e, fee_config, INITIAL_EXPIRATION_PERIOD);
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
    pub fn set_price(e: &Env, updates: PriceUpdate, timestamp: u64) {
        PriceOracleContractBase::set_price(e, updates, timestamp);
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
        PriceOracleContractBase::update_contract(e, wasm_hash);
    }
}
