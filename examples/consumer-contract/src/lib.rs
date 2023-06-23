#![no_std]

use soroban_sdk::{contractimpl, Address, Env, Vec};

/// Import the price oracle contract
mod oracle {
    soroban_sdk::contractimport!(file = "./se_price_oracle.wasm");
}

use oracle::{Client as PriceOracleClient, PriceData, Asset};

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
    pub fn admin(e: Env, contract_id: Address) -> Address {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.admin()
    }

    /// Returns the base asset of the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// 
    /// # Returns
    /// 
    /// * `Asset` - the base asset of the price oracle contract
    pub fn base(e: Env, contract_id: Address) -> Asset {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.base()
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
    pub fn decimals(e: Env, contract_id: Address) -> u32 {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.decimals()
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
    pub fn resolution(e: Env, contract_id: Address) -> u32 {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.resolution()
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
    pub fn period(e: Env, contract_id: Address) -> Option<u64> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.period()
    }

    /// Returns the supported assets of the price oracle contract
    /// 
    /// # Arguments
    /// 
    /// * `contract_id` - the contract id of the price oracle contract
    /// 
    /// # Returns
    /// 
    /// * `Vec<Asset>` - the assets of the price oracle contract
    pub fn assets(e: Env, contract_id: Address) -> Vec<Asset> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.assets()
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
        contract_id: Address,
        asset: Asset,
        timestamp: u64,
    ) -> Option<PriceData> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.price(&asset, &timestamp)
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
    pub fn lastprice(e: Env, contract_id: Address, asset: Asset) -> Option<PriceData> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.lastprice(&asset)
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
        contract_id: Address,
        base_asset: Asset,
        quote_asset: Asset,
        timestamp: u64,
    ) -> Option<PriceData> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.x_price(&base_asset, &quote_asset, &timestamp)
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
        contract_id: Address,
        base_asset: Asset,
        quote_asset: Asset,
    ) -> Option<PriceData> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.x_last_price(&base_asset, &quote_asset)
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
        contract_id: Address,
        asset: Asset,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.prices(&asset, &records)
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
        contract_id: Address,
        base_asset: Asset,
        quote_asset: Asset,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.x_prices(&base_asset, &quote_asset, &records)
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
    pub fn twap(e: Env, contract_id: Address, asset: Asset, records: u32) -> Option<i128> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.twap(&asset, &records)
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
        contract_id: Address,
        base_asset: Asset,
        quote_asset: Asset,
        records: u32,
    ) -> Option<i128> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.x_twap(&base_asset, &quote_asset, &records)
    }
}
