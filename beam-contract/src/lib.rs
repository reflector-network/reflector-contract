#![no_std]
mod cost;
mod tests;

use cost::{charge_invocation_fee, load_costs_config, set_costs_config, InvocationComplexity};
use oracle::price_oracle::PriceOracleContractBase;
use oracle::settings;
use oracle::types::{Asset, ConfigData, FeeConfig, PriceData, PriceUpdate};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

#[contract]
pub struct BeamOracleContract;

#[contractimpl]
impl BeamOracleContract {
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

    // Extends asset expiration date by a given amount of tokens.
    //
    // # Arguments
    //
    // * `sponsor` - Address that sponsors price feed
    // * `asset` - Quoted asset
    // * `amount` - Amount of tokens to burn for extending the expiration date
    //
    // # Panics
    //
    // Panics if asset is not supported or if retention config is malformed/missing
    pub fn extend_asset_ttl(e: &Env, sponsor: Address, asset: Asset, amount: i128) {
        PriceOracleContractBase::extend_asset_ttl(e, sponsor, asset, amount, 0);
    }

    // Return fee token address daily price feed retainer fee amount
    //
    // # Returns
    //
    // Fee token address and daily price feed retainer fee amount
    pub fn fee_config(e: &Env) -> FeeConfig {
        PriceOracleContractBase::fee_config(e)
    }

    // Retrieve current invocation costs config
    //
    // # Returns
    //
    // Invocation costs categorized by complexity
    pub fn invocation_costs(e: &Env) -> Vec<u64> {
        load_costs_config(e)
    }

    // Estimate invocation cost based on its complexity
    //
    // # Arguments
    //
    // * `invocation` - Invocation type (single price check, cross-price, TWAP, etc.)
    // * `periods` - Number of requested history periods
    //
    // # Returns
    //
    // Amount of fee tokens required to pay for invocation
    pub fn estimate_cost(e: &Env, invocation: InvocationComplexity, periods: u32) -> i128 {
        let fee_config = settings::get_fee_config(e);
        cost::estimate_invocation_cost(e, invocation, periods, fee_config)
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
    // * `caller` - Caller that covers invocation cost
    // * `asset` - Asset to quote
    // * `timestamp` - Timestamp in seconds
    //
    // # Returns
    //
    // Price record for given asset at given timestamp or None if not found
    pub fn price(e: &Env, caller: Address, asset: Asset, timestamp: u64) -> Option<PriceData> {
        caller.require_auth();
        charge_invocation_fee(e, &caller, InvocationComplexity::Price, 1);
        PriceOracleContractBase::price(e, asset, timestamp)
    }

    // Returns most recent price for an asset
    //
    // # Arguments
    //
    // * `caller` - Caller that covers invocation cost
    // * `asset` - Asset to quote
    //
    // # Returns
    //
    // Most recent price for given asset or None if asset is not supported
    pub fn lastprice(e: &Env, caller: Address, asset: Asset) -> Option<PriceData> {
        caller.require_auth();
        charge_invocation_fee(e, &caller, InvocationComplexity::Price, 1);
        PriceOracleContractBase::lastprice(e, asset)
    }

    // Return last N price records for given asset
    //
    // # Arguments
    //
    // * `caller` - Caller that covers invocation cost
    // * `asset` - Asset to quote
    // * `records` - Number of records to return
    //
    // # Returns
    //
    // Prices for given asset or None if asset is not supported
    pub fn prices(e: &Env, caller: Address, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        caller.require_auth();
        charge_invocation_fee(e, &caller, InvocationComplexity::Price, records);
        PriceOracleContractBase::prices(e, asset, records)
    }

    // Returns most recent cross price record for pair of assets
    //
    // # Arguments
    //
    // * `caller` - Caller that covers invocation cost
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    //
    // # Returns
    //
    // Recent cross price (base_asset_price/quote_asset_price) for given assets or None if there were no records found
    pub fn x_last_price(
        e: &Env,
        caller: Address,
        base_asset: Asset,
        quote_asset: Asset,
    ) -> Option<PriceData> {
        caller.require_auth();
        charge_invocation_fee(e, &caller, InvocationComplexity::CrossPrice, 1);
        PriceOracleContractBase::x_last_price(e, base_asset, quote_asset)
    }

    // Return cross price for pair of assets at specific timestamp
    //
    // # Arguments
    //
    // * `caller` - Caller that covers invocation cost
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    // * `timestamp` - Timestamp
    //
    // # Returns
    //
    // Cross price (base_asset_price/quote_asset_price) at given timestamp or None if there were no records found for quoted assets
    pub fn x_price(
        e: &Env,
        caller: Address,
        base_asset: Asset,
        quote_asset: Asset,
        timestamp: u64,
    ) -> Option<PriceData> {
        caller.require_auth();
        charge_invocation_fee(e, &caller, InvocationComplexity::CrossPrice, 1);
        PriceOracleContractBase::x_price(e, base_asset, quote_asset, timestamp)
    }

    // Returns last N cross price records of for pair of assets
    //
    // # Arguments
    //
    // * `caller` - Caller that covers invocation cost
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    // * `records` - Number of records to fetch
    //
    // # Returns
    //
    // Last N cross prices (base_asset_price/quote_asset_price) or None if there were no records found for quoted assets
    pub fn x_prices(
        e: &Env,
        caller: Address,
        base_asset: Asset,
        quote_asset: Asset,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        caller.require_auth();
        charge_invocation_fee(e, &caller, InvocationComplexity::CrossPrice, records);
        PriceOracleContractBase::x_prices(e, base_asset, quote_asset, records)
    }

    // Returns time-weighted average price for given asset over N recent records
    //
    // # Arguments
    //
    // * `caller` - Caller that covers invocation cost
    // * `asset` - Asset to quote
    // * `records` - Number of records to process
    //
    // # Returns
    //
    // TWAP for the given asset over N recent records or None if asset is not supported
    pub fn twap(e: &Env, caller: Address, asset: Asset, records: u32) -> Option<i128> {
        caller.require_auth();
        charge_invocation_fee(e, &caller, InvocationComplexity::Twap, 1);
        PriceOracleContractBase::twap(e, asset, records)
    }

    // Returns time-weighted average cross price for given asset pair over N recent records
    //
    // # Arguments
    //
    // * `caller` - Caller that covers invocation cost
    // * `base_asset` - Base asset
    // * `quote_asset` - Quote asset
    // * `records` - Number of records to process
    //
    // # Returns
    //
    // TWAP (base_asset_price/quote_asset_price) or None if assets are not supported
    pub fn x_twap(
        e: &Env,
        caller: Address,
        base_asset: Asset,
        quote_asset: Asset,
        records: u32,
    ) -> Option<i128> {
        caller.require_auth();
        charge_invocation_fee(e, &caller, InvocationComplexity::CrossTwap, records);
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
        PriceOracleContractBase::config(e, config, 0);
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
        PriceOracleContractBase::add_assets(e, assets, 0);
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
    pub fn set_fee_config(e: &Env, config: FeeConfig) {
        PriceOracleContractBase::set_fee_config(e, config, 0);
    }

    // Update costs configuration per each invocation category
    // Requires admin authorization
    //
    // # Arguments
    //
    // * `config` - Invocation costs for different invocation categories
    //
    // # Panics
    //
    // Panics if not authorized or not initialized yet
    pub fn set_invocation_costs_config(e: &Env, config: Vec<u64>) {
        set_costs_config(e, &config);
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
