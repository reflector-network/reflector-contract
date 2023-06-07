#![no_std]

mod test;
mod extensions;

use shared::constants::Constants;
use shared::price_oracle::PriceOracle;
use shared::extensions::{env_extensions::EnvExtensions};
use shared::types::asset::Asset;
use shared::types::{error::Error, config_data::ConfigData, price_data::PriceData};
use extensions::env_balance_extensions::EnvBalanceExtensions;
use soroban_sdk::{contractimpl, panic_with_error, Address, BytesN, Env, Vec};

mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

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
        let base_fee = config.base_fee;
        PriceOracle::config(&e, user, config);
        e.set_base_fee(base_fee);
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

    /// Sets the fee for the contract. Can only be called by the admin.
    /// 
    /// # Arguments
    /// 
    /// * `user` - The admin address.
    /// * `fee` - The fee to set.
    /// 
    /// # Panics
    /// 
    /// Panics if the caller is not the admin.
    pub fn set_fee(e: Env, user: Address, fee: i128) {
        e.panic_if_not_admin(&user);
        e.set_base_fee(fee);
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
        PriceOracle::set_price(&e, user, updates, timestamp)
    }

    //end of admin section

    //Balance section

    /// Deposits the given amount of fee asset to the current contract address. Can only be called by the user.
    /// 
    /// # Arguments
    /// 
    /// * `user` - The user address.
    /// * `account` - The contract address to deposit to.
    /// * `asset` - The fee asset to deposit.
    /// * `amount` - The amount to deposit.
    /// 
    /// # Panics
    /// 
    /// Panics if the amount is invalid, or if the fee asset is invalid, or if transfer fails.
    pub fn deposit(e: Env, user: Address, account: BytesN<32>, asset: Address, amount: i128) {
        user.require_auth();
        if amount <= 0 {
            panic_with_error!(&e, Error::InvalidDepositAmount);
        }
        let fee_asset = fee_asset(&e);
        if fee_asset != asset {
            panic_with_error!(&e, Error::InvalidFeeAsset);
        }
        let token = token::Client::new(&e, &asset.contract_id().unwrap());
        token.xfer(&user, &e.current_contract_address(), &amount);
        e.try_inc_balance(account, amount);
    }

    /// Returns the balance of the given account.
    pub fn balance(e: Env, account: BytesN<32>) -> Option<i128> {
        e.get_balance(account)
    }

    /// Returns the fee asset of the contract.
    pub fn fee_asset(e: Env) -> Address {
        fee_asset(&e)
    }

    /// Returns the base fee of the contract.
    pub fn base_fee(e: Env) -> Option<i128> {
        e.get_base_fee()
    }

    //end of balance section

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
    /// The base asset address.
    pub fn base(e: Env) -> Address {
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


    /// Returns the prices for the given asset at the given timestamp.
    /// 
    /// # Arguments
    /// 
    /// * `asset` - The asset.
    /// * `timestamp` - The timestamp.
    /// 
    /// # Panics
    /// 
    /// If invoker is not authorized, or if the invoker does not have enough balance.
    /// 
    /// # Returns
    /// 
    /// The prices for the given asset at the given timestamp or None if the asset is not supported, or if the timestamp is invalid. 
    pub fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData> {
        let invoker = get_invoker_or_panic(&e);
        charge_or_panic(&e, invoker, 1);
        let price = PriceOracle::price(&e, asset, timestamp);
        if price.is_none() {
            return None;
        }
        price
    }

    /// Returns the last price for the given asset.
    /// 
    /// # Arguments
    /// 
    /// * `asset` - The asset.
    /// 
    /// # Panics
    /// 
    /// If invoker is not authorized, or if the invoker does not have enough balance.
    /// 
    /// # Returns
    /// 
    /// The last price for the given asset or None if the asset is not supported.
    pub fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        let invoker = get_invoker_or_panic(&e);
        charge_or_panic(&e, invoker, 1);
        let price = PriceOracle::lastprice(&e, asset);
        if price.is_none() {
            return None;
        }
        price
    }

    /// Returns the cross price for the given assets at the given timestamp.
    /// 
    /// # Arguments
    /// 
    /// * `base_asset` - The base asset.
    /// * `quote_asset` - The quote asset.
    /// * `timestamp` - The timestamp.
    /// 
    /// # Panics
    /// 
    /// If invoker is not authorized, or if the invoker does not have enough balance.
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
        let invoker = get_invoker_or_panic(&e);
        charge_or_panic(&e, invoker, 2);
        let price = PriceOracle::x_price(&e, base_asset, quote_asset, timestamp);
        if price.is_none() {
            return None;
        }
        price
    }

    /// Returns the last cross price for the given assets.
    /// 
    /// # Arguments
    /// 
    /// * `base_asset` - The base asset.
    /// * `quote_asset` - The quote asset.
    /// 
    /// # Panics
    /// 
    /// If invoker is not authorized, or if the invoker does not have enough balance.
    /// 
    /// # Returns
    /// 
    /// The last cross price for the given assets or None if the assets are not supported.
    pub fn x_last_price(e: Env, base_asset: Asset, quote_asset: Asset) -> Option<PriceData> {
        let invoker = get_invoker_or_panic(&e);
        charge_or_panic(&e, invoker, 2);
        let price = PriceOracle::x_last_price(&e, base_asset, quote_asset);
        if price.is_none() {
            return None;
        }
        price
    }

    /// Returns the stack of prices for the given asset.
    /// 
    /// # Arguments
    /// 
    /// * `asset` - The asset.
    /// * `records` - The number of records to return.
    /// 
    /// # Panics
    /// 
    /// If invoker is not authorized, or if the invoker does not have enough balance.
    /// 
    /// # Returns
    /// 
    /// The prices for the given asset or None if the asset is not supported. If there are fewer records than requested, the returned vector will be shorter.
    pub fn prices(e: Env, asset: Asset, records: u32) -> Option<Vec<PriceData>> {
        let invoker = get_invoker_or_panic(&e);
        charge_or_panic(&e, invoker, records); //TODO: check price multiplier
        let price =  PriceOracle::prices(&e, asset, records);
        if price.is_none() {
            return None;
        }
        price
    }

    /// Returns the stack of cross prices for the given assets.
    /// 
    /// # Arguments
    /// 
    /// * `base_asset` - The base asset.
    /// * `quote_asset` - The quote asset.
    /// 
    /// # Panics
    /// 
    /// If invoker is not authorized, or if the invoker does not have enough balance.
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
        let invoker = get_invoker_or_panic(&e);
        charge_or_panic(&e, invoker, records * 2);//TODO: check price multiplier
        let prices = PriceOracle::x_prices(&e, base_asset, quote_asset, records);
        if prices.is_none() {
            return None;
        }
        prices
    }

    /// Returns the time-weighted average price for the given asset over the given number of records.
    /// 
    /// # Arguments
    /// 
    /// * `asset` - The asset.
    /// * `records` - The number of records to use.
    /// 
    /// # Panics
    /// 
    /// If invoker is not authorized, or if the invoker does not have enough balance.
    /// 
    /// # Returns
    /// 
    /// The time-weighted average price for the given asset over the given number of records or None if the asset is not supported.
    pub fn twap(e: Env, asset: Asset, records: u32) -> Option<i128> {
        let invoker = get_invoker_or_panic(&e);
        charge_or_panic(&e, invoker, records);
        let prices = PriceOracle::twap(&e, asset, records);
        if prices.is_none() {
            return None;
        }
        prices
    }

    /// Returns the time-weighted average cross price for the given assets over the given number of records.
    /// 
    /// # Arguments
    /// 
    /// * `base_asset` - The base asset.
    /// * `quote_asset` - The quote asset.
    /// 
    /// # Panics
    /// 
    /// If invoker is not authorized, or if the invoker does not have enough balance.
    /// 
    /// # Returns
    /// 
    /// The time-weighted average cross price for the given assets over the given number of records or None if the assets are not supported.
    pub fn x_twap(e: Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<i128> {
        let invoker = get_invoker_or_panic(&e);
        charge_or_panic(&e, invoker, records);
        let prices = PriceOracle::x_twap(&e, base_asset, quote_asset, records);
        if prices.is_none() {
            return None;
        }
        prices
    }
}

fn fee_asset(e: &Env) -> Address {
    let bytes = BytesN::from_array(e, &Constants::FEE_ASSET);
    Address::from_contract_id(&e, &bytes)
}

fn get_invoker_or_panic(e: &Env) -> BytesN<32> {
    let invoker = e.invoker();
    if invoker.is_none() {
        panic_with_error!(e, Error::Unauthorized)
    }
    invoker.unwrap()
}

fn charge_or_panic(e: &Env, account: BytesN<32>, multiplier: u32) {
    let base_fee = e.get_base_fee().unwrap_or_else(||0);
    let amount = -(base_fee * multiplier as i128);
    if !e.try_inc_balance(account, amount) { 
        panic_with_error!(&e, Error::InsufficientBalance) 
    }
}