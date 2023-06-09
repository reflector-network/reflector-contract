#![no_std]

use soroban_sdk::{
    contracterror, contractimpl, contracttype, panic_with_error, Address, BytesN, Env, FromVal,
    RawVal, Symbol, Vec,
};

/// PriceData is a struct that contains the price and timestamp and can be mapped to the price oracle contract type
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}

/// Error is a enum that contains the error codes that can be returned by the price oracle contract
#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    Unauthorized = 1,
    InsufficientBalance = 13,
    CustomUnauthorized = 101,
    CustomInsufficientBalance = 113,
}

pub struct PriceOracleConsumerContract;

#[contractimpl]
impl PriceOracleConsumerContract {

    /// Returns the admin address of the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// 
    /// # Returns
    /// 
    /// * `Address` - the admin address of the price oracle contract
    pub fn admin(e: Env, contract_id: BytesN<32>) -> Address {
        e.invoke_contract(&contract_id, &Symbol::short("admin"), Vec::new(&e))
    }

    /// Returns the base asset of the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// 
    /// # Returns
    /// 
    /// * `Address` - the base asset of the price oracle contract
    pub fn base(e: Env, contract_id: BytesN<32>) -> Address {
        e.invoke_contract(&contract_id, &Symbol::short("base"), Vec::new(&e))
    }

    /// Returns the decimals of the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// 
    /// # Returns
    /// 
    /// * `u32` - the decimals of the price oracle contract
    pub fn decimals(e: Env, contract_id: BytesN<32>) -> u32 {
        e.invoke_contract(&contract_id, &Symbol::short("decimals"), Vec::new(&e))
    }

    /// Returns the prices resolution of the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// 
    /// # Returns
    /// 
    /// * `u32` - the prices resolution of the price oracle contract
    pub fn resolution(e: Env, contract_id: BytesN<32>) -> u32 {
        e.invoke_contract(&contract_id, &Symbol::short("resolution"), Vec::new(&e))
    }

    /// Returns the retention period of the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// 
    /// # Returns
    /// 
    /// * `u64` - the retention period of the price oracle contract
    pub fn period(e: Env, contract_id: BytesN<32>) -> Option<u64> {
        e.invoke_contract(&contract_id, &Symbol::short("period"), Vec::new(&e))
    }

    /// Returns the supported assets of the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// 
    /// # Returns
    /// 
    /// * `Vec<Address>` - the assets of the price oracle contract
    pub fn assets(e: Env, contract_id: BytesN<32>) -> Option<Vec<Address>> {
        e.invoke_contract(&contract_id, &Symbol::short("assets"), Vec::new(&e))
    }

    /// Returns the price of the asset at the given timestamp that is stored in the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// * `asset` - the asset to get the price for
    /// * `timestamp` - the timestamp to get the price for
    /// 
    /// # Returns
    /// 
    /// * `Option<PriceData>` - the price of the asset at the given timestamp
    /// 
    /// # Panics
    /// 
    /// * If the contract call fails or the contract returns an error
    pub fn price(
        e: Env,
        contract_id: BytesN<32>,
        asset: Address,
        timestamp: u64,
    ) -> Option<PriceData> {
        let args: Vec<RawVal> =
            Vec::from_array(&e, [asset.to_raw(), RawVal::from_val(&e, &timestamp)]);
        match e.try_invoke_contract::<Option<PriceData>, Error>(
            &contract_id,
            &Symbol::short("price"),
            args,
        ) {
            Ok(result) => result.unwrap(),
            Err(err) => match err.unwrap() {
                Error::Unauthorized => panic_with_error!(e, Error::CustomUnauthorized),
                Error::InsufficientBalance => {
                    panic_with_error!(e, Error::CustomInsufficientBalance)
                }
                _ => panic_with_error!(e, err.unwrap()),
            },
        }
    }

    /// Returns the last price of the asset that is stored in the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// * `asset` - the asset to get the price for
    /// 
    /// # Returns
    /// 
    /// * `Option<PriceData>` - the last price of the asset
    /// 
    /// # Panics
    /// 
    /// * If the contract call fails or the contract returns an error
    pub fn lastprice(e: Env, contract_id: BytesN<32>, asset: Address) -> Option<PriceData> {
        let args: Vec<RawVal> = Vec::from_array(&e, [asset.to_raw()]);
        match e.try_invoke_contract::<Option<PriceData>, Error>(
            &contract_id,
            &Symbol::short("lastprice"),
            args,
        ) {
            Ok(result) => result.unwrap(),
            Err(err) => match err.unwrap() {
                Error::Unauthorized => panic_with_error!(e, Error::CustomUnauthorized),
                Error::InsufficientBalance => {
                    panic_with_error!(e, Error::CustomInsufficientBalance)
                }
                _ => panic_with_error!(e, err.unwrap()),
            },
        }
    }

    /// Returns the cross price of the base asset and the quote asset at the given timestamp that is stored in the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// * `base_asset` - the base asset to get the price for
    /// * `quote_asset` - the quote asset to get the price for
    /// * `timestamp` - the timestamp to get the price for
    /// 
    /// # Returns
    /// 
    /// * `Option<PriceData>` - the cross price of the base asset and the quote asset at the given timestamp
    /// 
    /// # Panics
    /// 
    /// * If the contract call fails or the contract returns an error
    pub fn x_price(
        e: Env,
        contract_id: BytesN<32>,
        base_asset: Address,
        quote_asset: Address,
        timestamp: u64,
    ) -> Option<PriceData> {
        let args: Vec<RawVal> = Vec::from_array(
            &e,
            [
                base_asset.to_raw(),
                quote_asset.to_raw(),
                RawVal::from_val(&e, &timestamp),
            ],
        );
        match e.try_invoke_contract::<Option<PriceData>, Error>(
            &contract_id,
            &Symbol::short("x_price"),
            args,
        ) {
            Ok(result) => result.unwrap(),
            Err(err) => match err.unwrap() {
                Error::Unauthorized => panic_with_error!(e, Error::CustomUnauthorized),
                Error::InsufficientBalance => {
                    panic_with_error!(e, Error::CustomInsufficientBalance)
                }
                _ => panic_with_error!(e, err.unwrap()),
            },
        }
    }

    /// Returns the last cross price of the base asset and the quote asset that is stored in the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// * `base_asset` - the base asset to get the price for
    /// * `quote_asset` - the quote asset to get the price for
    /// 
    /// # Returns
    /// 
    /// * `Option<PriceData>` - the last cross price of the base asset and the quote asset
    /// 
    /// # Panics
    /// 
    /// * If the contract call fails or the contract returns an error
    pub fn x_last_price(
        e: Env,
        contract_id: BytesN<32>,
        base_asset: Address,
        quote_asset: Address,
    ) -> Option<PriceData> {
        let args: Vec<RawVal> = Vec::from_array(&e, [base_asset.to_raw(), quote_asset.to_raw()]);
        match e.try_invoke_contract::<Option<PriceData>, Error>(
            &contract_id,
            &Symbol::short("x_last_price"),
            args,
        ) {
            Ok(result) => result.unwrap(),
            Err(err) => match err.unwrap() {
                Error::Unauthorized => panic_with_error!(e, Error::CustomUnauthorized),
                Error::InsufficientBalance => {
                    panic_with_error!(e, Error::CustomInsufficientBalance)
                }
                _ => panic_with_error!(e, err.unwrap()),
            },
        }
    }

    /// Returns the last n prices of the asset that are stored in the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// * `asset` - the asset to get the prices for
    /// * `records` - the number of records to get
    /// 
    /// # Returns
    /// 
    /// * `Option<Vec<PriceData>>` - the last n prices of the asset
    /// 
    /// # Panics
    /// 
    /// * If the contract call fails or the contract returns an error
    pub fn prices(
        e: Env,
        contract_id: BytesN<32>,
        asset: Address,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        let args: Vec<RawVal> =
            Vec::from_array(&e, [asset.to_raw(), RawVal::from_val(&e, &records)]);
        match e.try_invoke_contract::<Option<Vec<PriceData>>, Error>(
            &contract_id,
            &Symbol::short("prices"),
            args,
        ) {
            Ok(result) => result.unwrap(),
            Err(err) => match err.unwrap() {
                Error::Unauthorized => panic_with_error!(e, Error::CustomUnauthorized),
                Error::InsufficientBalance => {
                    panic_with_error!(e, Error::CustomInsufficientBalance)
                }
                _ => panic_with_error!(e, err.unwrap()),
            },
        }
    }

    /// Returns the last n cross prices of the base asset and the quote asset that are stored in the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// * `base_asset` - the base asset to get the prices for
    /// * `quote_asset` - the quote asset to get the prices for
    /// * `records` - the number of records to get
    /// 
    /// # Returns
    /// 
    /// * `Option<Vec<PriceData>>` - the last n cross prices of the base asset and the quote asset
    /// 
    /// # Panics
    /// 
    /// * If the contract call fails or the contract returns an error
    pub fn x_prices(
        e: Env,
        contract_id: BytesN<32>,
        base_asset: Address,
        quote_asset: Address,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        let args: Vec<RawVal> = Vec::from_array(
            &e,
            [
                base_asset.to_raw(),
                quote_asset.to_raw(),
                RawVal::from_val(&e, &records),
            ],
        );
        match e.try_invoke_contract::<Option<Vec<PriceData>>, Error>(
            &contract_id,
            &Symbol::short("x_prices"),
            args,
        ) {
            Ok(result) => result.unwrap(),
            Err(err) => match err.unwrap() {
                Error::Unauthorized => panic_with_error!(e, Error::CustomUnauthorized),
                Error::InsufficientBalance => {
                    panic_with_error!(e, Error::CustomInsufficientBalance)
                }
                _ => panic_with_error!(e, err.unwrap()),
            },
        }
    }

    /// Returns the time-weighted average price of the asset for the last n records that are stored in the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// * `asset` - the asset to get the price for
    /// * `records` - the number of records to get
    /// 
    /// # Returns
    /// 
    /// * `Option<i128>` - the time-weighted average price of the asset for the last n records
    /// 
    /// # Panics
    /// 
    /// * If the contract call fails or the contract returns an error
    pub fn twap(e: Env, contract_id: BytesN<32>, asset: Address, records: u32) -> Option<i128> {
        let args: Vec<RawVal> =
            Vec::from_array(&e, [asset.to_raw(), RawVal::from_val(&e, &records)]);
        match e.try_invoke_contract::<Option<i128>, Error>(
            &contract_id,
            &Symbol::short("twap"),
            args,
        ) {
            Ok(result) => result.unwrap(),
            Err(err) => match err.unwrap() {
                Error::Unauthorized => panic_with_error!(e, Error::CustomUnauthorized),
                Error::InsufficientBalance => {
                    panic_with_error!(e, Error::CustomInsufficientBalance)
                }
                _ => panic_with_error!(e, err.unwrap()),
            },
        }
    }

    /// Returns the time-weighted average price of the base asset and the quote asset for the last n records that are stored in the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// * `base_asset` - the base asset to get the price for
    /// * `quote_asset` - the quote asset to get the price for
    /// * `records` - the number of records to get
    /// 
    /// # Returns
    /// 
    /// * `Option<i128>` - the time-weighted average price of the base asset and the quote asset for the last n records
    /// 
    /// # Panics
    /// 
    /// * If the contract call fails or the contract returns an error
    pub fn x_twap(
        e: Env,
        contract_id: BytesN<32>,
        base_asset: Address,
        quote_asset: Address,
        records: u32,
    ) -> Option<i128> {
        let args: Vec<RawVal> = Vec::from_array(
            &e,
            [
                base_asset.to_raw(),
                quote_asset.to_raw(),
                RawVal::from_val(&e, &records),
            ],
        );
        match e.try_invoke_contract::<Option<i128>, Error>(
            &contract_id,
            &Symbol::short("x_twap"),
            args,
        ) {
            Ok(result) => result.unwrap(),
            Err(err) => match err.unwrap() {
                Error::Unauthorized => panic_with_error!(e, Error::CustomUnauthorized),
                Error::InsufficientBalance => {
                    panic_with_error!(e, Error::CustomInsufficientBalance)
                }
                _ => panic_with_error!(e, err.unwrap()),
            },
        }
    }
}
