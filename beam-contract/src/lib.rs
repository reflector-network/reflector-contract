#![no_std]
mod access;

use oracle::price_oracle::PriceOracleContractBase;
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

    // Return current contract version (from package)
    //
    // # Returns
    //
    // Contract version
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
    // Asset expiration timestamp (in seconds) or None if asset is not supported
    //
    // # Panics
    //
    // Panics if asset is not supported
    pub fn expires(e: &Env, asset: Asset) -> Option<u64> {
        PriceOracleContractBase::expires(e, asset)
    }

    // Purchase access to asset price feeds (XRF token amount charged from the sponsor and split evenly
    // between all assets). Feed expiration is automatically extended to cover the purchased access.
    //
    // # Arguments
    //
    // * `sponsor` - Address that pays for the access
    // * `consumer` - Address that receives the access
    // * `assets` - Assets to track (must be supported, no duplicates)
    // * `amount` - Amount of fee tokens to burn
    //
    // # Returns
    //
    // New access expiration UNIX timestamps (in seconds) for each requested asset
    //
    // # Panics
    //
    // Panics if the request or amount is invalid, or fee config is missing
    pub fn track(
        e: &Env,
        sponsor: Address,
        consumer: Address,
        assets: Vec<Asset>,
        amount: i128,
    ) -> Vec<u64> {
        access::track(e, sponsor, consumer, assets, amount)
    }

    // Return access expiration UNIX timestamp (in seconds) for the given consumer and assets
    //
    // # Arguments
    //
    // * `consumer` - Caller address to check access for
    // * `assets` - Assets to check
    //
    // # Returns
    //
    // Access expiration UNIX timestamp (in seconds) for each asset, 0 if no access was tracked
    pub fn tracked_until(e: &Env, consumer: Address, assets: Vec<Asset>) -> Vec<u64> {
        access::access_until(e, consumer, assets)
    }

    // Return fee token address daily price feed retainer fee amount
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
    // * `caller` - Caller with active access to the asset feed
    // * `asset` - Asset to quote
    // * `timestamp` - Timestamp in seconds
    //
    // # Returns
    //
    // Price record for given asset at given timestamp or None if not found
    //
    // # Panics
    //
    // Panics if the caller is not entitled to read the asset feed
    pub fn price(e: &Env, caller: Address, asset: Asset, timestamp: u64) -> Option<PriceData> {
        caller.require_auth();
        access::check_access(e, &caller, &asset);
        PriceOracleContractBase::price(e, asset, timestamp)
    }

    // Returns most recent price for an asset
    //
    // # Arguments
    //
    // * `caller` - Caller with active access to the asset feed
    // * `asset` - Asset to quote
    //
    // # Returns
    //
    // Most recent price for given asset or None if asset is not supported
    //
    // # Panics
    //
    // Panics if the caller is not entitled to read the asset feed
    pub fn lastprice(e: &Env, caller: Address, asset: Asset) -> Option<PriceData> {
        caller.require_auth();
        access::check_access(e, &caller, &asset);
        PriceOracleContractBase::lastprice(e, asset)
    }

    // Return last N price records for given asset
    //
    // # Arguments
    //
    // * `caller` - Caller with active access to the asset feed
    // * `asset` - Asset to quote
    // * `records` - Number of records to return
    //
    // # Returns
    //
    // Prices for given asset or None if asset is not supported
    //
    // # Panics
    //
    // Panics if the caller is not entitled to read the asset feed
    pub fn prices(e: &Env, caller: Address, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        caller.require_auth();
        access::check_access(e, &caller, &asset);
        PriceOracleContractBase::prices(e, asset, records)
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
    // * `period` - History retention period (in milliseconds)
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

#[cfg(test)]
mod tests;
