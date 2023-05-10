#![no_std]

use soroban_sdk::{contractimpl, contracttype, Address, BytesN, Env, FromVal, RawVal, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}

pub struct PriceOracleConsumerContract;

#[contractimpl]
impl PriceOracleConsumerContract {
    pub fn admin(e: Env, contract_id: BytesN<32>) -> Address {
        e.invoke_contract(&contract_id, &Symbol::short("admin"), Vec::new(&e))
    }

    pub fn base(e: Env, contract_id: BytesN<32>) -> Address {
        e.invoke_contract(&contract_id, &Symbol::short("base"), Vec::new(&e))
    }

    pub fn decimals(e: Env, contract_id: BytesN<32>) -> u32 {
        e.invoke_contract(&contract_id, &Symbol::short("decimals"), Vec::new(&e))
    }

    pub fn resolution(e: Env, contract_id: BytesN<32>) -> u32 {
        e.invoke_contract(&contract_id, &Symbol::short("resolution"), Vec::new(&e))
    }

    pub fn period(e: Env, contract_id: BytesN<32>) -> Option<u64> {
        e.invoke_contract(&contract_id, &Symbol::short("period"), Vec::new(&e))
    }

    pub fn assets(e: Env, contract_id: BytesN<32>) -> Option<Vec<Address>> {
        e.invoke_contract(&contract_id, &Symbol::short("assets"), Vec::new(&e))
    }

    pub fn price(
        e: Env,
        contract_id: BytesN<32>,
        asset: Address,
        timestamp: u64,
    ) -> Option<PriceData> {
        let args: Vec<RawVal> =
            Vec::from_array(&e, [asset.to_raw(), RawVal::from_val(&e, &timestamp)]);
        e.invoke_contract(&contract_id, &Symbol::short("price"), args)
    }

    pub fn lastprice(e: Env, contract_id: BytesN<32>, asset: Address) -> Option<PriceData> {
        let args: Vec<RawVal> = Vec::from_array(&e, [asset.to_raw()]);
        e.invoke_contract(&contract_id, &Symbol::short("lastprice"), args)
    }

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
        e.invoke_contract(&contract_id, &Symbol::short("x_price"), args)
    }

    pub fn x_last_price(
        e: Env,
        contract_id: BytesN<32>,
        base_asset: Address,
        quote_asset: Address,
    ) -> Option<PriceData> {
        let args: Vec<RawVal> = Vec::from_array(&e, [base_asset.to_raw(), quote_asset.to_raw()]);
        e.invoke_contract(&contract_id, &Symbol::short("x_last_price"), args)
    }

    pub fn prices(
        e: Env,
        contract_id: BytesN<32>,
        asset: Address,
        records: u32,
    ) -> Option<Vec<PriceData>> {
        let args: Vec<RawVal> =
            Vec::from_array(&e, [asset.to_raw(), RawVal::from_val(&e, &records)]);
        e.invoke_contract(&contract_id, &Symbol::short("prices"), args)
    }

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
        e.invoke_contract(&contract_id, &Symbol::short("x_prices"), args)
    }

    pub fn twap(e: Env, contract_id: BytesN<32>, asset: Address, records: u32) -> Option<i128> {
        let args: Vec<RawVal> =
            Vec::from_array(&e, [asset.to_raw(), RawVal::from_val(&e, &records)]);
        e.invoke_contract(&contract_id, &Symbol::short("twap"), args)
    }

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
        e.invoke_contract(&contract_id, &Symbol::short("x_twap"), args)
    }
}
