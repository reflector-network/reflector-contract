#![no_std]

mod test;

use shared::price_oracle::PriceOracle;
use shared::types::asset::Asset;
use shared::types::price_update_item::PriceUpdateItem;
use shared::types::{config_data::ConfigData, price_data::PriceData};
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

#[contract]
pub struct PriceOracleContract;

#[contractimpl]
impl PriceOracleContract {
    //Admin section

    /// Configures the contract with the given parameters. Can only be called by the admin.
    /// 
    /// # Arguments
    /// 
    /// * `user` - The admin address.
    /// * `config` - The configuration parameters.
    /// 
    /// # Panics
    /// 
    /// Panics if the caller is not the admin. 
    pub fn config(e: Env, user: Address, config: ConfigData) {
        PriceOracle::config(&e, user, config)
    }

    /// Returns the configuration version of the contract.
    /// 
    /// # Returns
    /// 
    /// The configuration version.
    pub fn config_version(e: Env) -> u32 {
        PriceOracle::config_version(&e)
    }

    /// Adds the given assets to the contract. Can only be called by the admin.
    /// 
    /// # Arguments
    /// 
    /// * `user` - The admin address.
    /// * `assets` - The assets to add.
    /// 
    /// # Panics
    /// 
    /// Panics if the caller is not the admin, or if the assets are already added.
    pub fn add_assets(e: Env, user: Address, assets: Vec<Asset>) {
        PriceOracle::add_assets(&e, user, assets)
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
    pub fn set_price(e: Env, user: Address, updates: Vec<PriceUpdateItem>, timestamp: u64) {
        PriceOracle::set_price(&e, user, updates, timestamp)
    }

    //end of admin section

    /// Returns the contract admin address.
    /// 
    /// # Returns
    /// 
    /// The admin address.
    pub fn admin(e: Env) -> Address {
        PriceOracle::admin(&e)
    }

    /// Returns the base asset.
    /// 
    /// # Returns
    /// 
    /// The base asset.
    pub fn base(e: Env) -> Asset {
        PriceOracle::base(&e)
    }

    /// Returns the number of decimals for the prices.
    /// 
    /// # Returns
    /// 
    /// The number of decimals.
    pub fn decimals(e: Env) -> u32 {
        PriceOracle::decimals(&e)
    }

    /// Returns the prices resolution.
    /// 
    /// # Returns
    /// 
    /// The prices resolution.
    pub fn resolution(e: Env) -> u32 {
        PriceOracle::resolution(&e)
    }

    /// Returns the retention period of the prices in seconds.
    /// 
    /// # Returns
    /// 
    /// The retention period.
    pub fn period(e: Env) -> Option<u64> {
        PriceOracle::period(&e)
    }

    /// Returns the assets supported by the contract.
    /// 
    /// # Returns
    /// 
    /// The assets supported by the contract or None if no assets are supported.
    pub fn assets(e: Env) -> Vec<Asset> {
        PriceOracle::assets(&e)
    }

    /// Returns the timestamp of the last price update.
    /// 
    /// # Returns
    /// 
    /// The timestamp of the last price update.
    pub fn last_timestamp(e: Env) -> u64 {
        PriceOracle::last_timestamp(&e)
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
        PriceOracle::price(&e, asset, timestamp)
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
        PriceOracle::lastprice(&e, asset)
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
        PriceOracle::x_price(&e, base_asset, quote_asset, timestamp)
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
        PriceOracle::x_last_price(&e, base_asset, quote_asset)
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
        PriceOracle::prices(&e, asset, records)
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
        PriceOracle::x_prices(&e, base_asset, quote_asset, records)
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
        PriceOracle::twap(&e, asset, records)
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
        PriceOracle::x_twap(&e, base_asset, quote_asset, records)
    }
}