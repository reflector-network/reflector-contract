#![no_std]

use soroban_sdk::{contractimpl, Address, BytesN, Env, Vec};
mod oracle {
    soroban_sdk::contractimport!(file = "../../target/wasm32-unknown-unknown/release/se_price_oracle.wasm");
}

use oracle::{Client as PriceOracleClient, PriceData};

pub struct PriceOracleConsumerContract;

#[contractimpl]
impl PriceOracleConsumerContract {
    pub fn admin(e: Env, contract_id: BytesN<32>) -> Address {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.admin()
    }

    pub fn base(e: Env, contract_id: BytesN<32>) -> Address {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.base()
    }

    pub fn decimals(e: Env, contract_id: BytesN<32>) -> u32 {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.decimals()
    }

    pub fn resolution(e: Env, contract_id: BytesN<32>) -> u32 {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.resolution()
    }

    pub fn period(e: Env, contract_id: BytesN<32>) -> Option<u64> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.period()
    }

    pub fn assets(e: Env, contract_id: BytesN<32>) -> Option<Vec<Address>> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.assets()
    }

    pub fn price(
        e: Env,
        contract_id: BytesN<32>,
        asset: Address,
        timestamp: u64,
    ) -> Option<PriceData> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.price(&asset, &timestamp)
    }

    pub fn lastprice(e: Env, contract_id: BytesN<32>, asset: Address) -> Option<PriceData> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.lastprice(&asset)
    }

    pub fn x_price(
        e: Env,
        contract_id: BytesN<32>,
        base_asset: Address,
        quote_asset: Address,
        timestamp: u64,
    ) -> Option<PriceData> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.x_price(&base_asset, &quote_asset, &timestamp)
    }

    pub fn x_last_price(
        e: Env,
        contract_id: BytesN<32>,
        base_asset: Address,
        quote_asset: Address,
    ) -> Option<PriceData> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.x_last_price(&base_asset, &quote_asset)
    }

    pub fn prices(
        e: Env,
        contract_id: BytesN<32>,
        asset: Address,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.prices(&asset, &records)
    }

    pub fn x_prices(
        e: Env,
        contract_id: BytesN<32>,
        base_asset: Address,
        quote_asset: Address,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.x_prices(&base_asset, &quote_asset, &records)
    }

    pub fn twap(e: Env, contract_id: BytesN<32>, asset: Address, records: u32) -> Option<i128> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.twap(&asset, &records)
    }

    pub fn x_twap(
        e: Env,
        contract_id: BytesN<32>,
        base_asset: Address,
        quote_asset: Address,
        records: u32,
    ) -> Option<i128> {
        let contract = PriceOracleClient::new(&e, &contract_id);
        contract.x_twap(&base_asset, &quote_asset, &records)
    }
}
