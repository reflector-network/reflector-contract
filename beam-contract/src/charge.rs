use crate::{settings, types::invocation::Invocation};
use shared::types::fee_config::FeeConfig;
use soroban_sdk::{Address, Env};

const SCALE: u64 = 1_000_000;
const CROSS_PRICE_KOEF: u64 = 2_000_000;
const TWAP_KOEF: u64 = 1_500_000;
const CROSS_TWAP_KOEF: u64 = 3_000_000;
const ROUND_FEE_KOEF: u64 = 1_100_000;

fn mul_scaled(value: u64, koef: u64) -> u64 {
    value * koef / SCALE
}

pub fn calc_fee(
    base_fee: u64,
    invocation: Invocation,
    rounds: u32,
) -> u64 {
    let mut koef = 1_000_000;
    match invocation {
        Invocation::Price => {}
        Invocation::Twap => {
            koef = TWAP_KOEF;
        }
        Invocation::CrossPrice => {
            koef = CROSS_PRICE_KOEF;
        }
        Invocation::CrossTwap => {
            koef = CROSS_TWAP_KOEF;
        }
    }
    let mut fee = mul_scaled(base_fee, koef);
    if rounds > 1 {
        fee = mul_scaled(fee, ROUND_FEE_KOEF);
    }
    fee
}

pub fn charge_fee(
    e: &Env,
    caller: &Address,
    invocation: Invocation,
    rounds: u32,
) {
    let fee_config = settings::get_invocation_config(e);
    match fee_config {
        FeeConfig::None => return,
        FeeConfig::Some((fee_token, base_fee)) => {
            let fee = calc_fee(base_fee as u64, invocation, rounds) as i128;
            let token = soroban_sdk::token::Client::new(e, &fee_token);
            token.transfer(caller, &e.current_contract_address(), &fee);
        },
    }
}