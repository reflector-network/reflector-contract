use oracle::settings;
use oracle::types::FeeConfig;
use soroban_sdk::{token, Address, Env, Vec};

const COST_CONFIG_KEY: &str = "cost";
const SCALE: u64 = 10_000_000;

pub enum InvocationComplexity {
    Price = 0,
    Twap = 1,
    CrossPrice = 2,
    CrossTwap = 3,
}

// Update invocation costs config
#[inline]
pub fn set_costs_config(e: &Env, costs: &Vec<u64>) {
    e.storage().instance().set(&COST_CONFIG_KEY, &costs);
}

// Load config containing invocation costs
pub fn load_costs_config(e: &Env) -> Vec<u64> {
    e.storage()
        .instance()
        .get(&COST_CONFIG_KEY)
        .unwrap_or_else(|| {
            Vec::from_array(
                e,
                [2_000_000, 10_000_000, 15_000_000, 20_000_000, 30_000_000],
            )
        })
}

// Charge per-invocation fee
pub fn charge_invocation_fee(
    e: &Env,
    caller: &Address,
    invocation: InvocationComplexity,
    rounds: u32,
) {
    let fee_config = settings::get_fee_config(e);
    match fee_config {
        FeeConfig::None => return,
        FeeConfig::Some((fee_token, _)) => {
            //load rates
            let costs = load_costs_config(e);
            //calculate amount to charge
            let cost = estimate_invocation_cost(costs, invocation, rounds) as i128;
            //init fee token client
            let fee_client = token::Client::new(e, &fee_token);
            //burn tokens
            fee_client.burn(caller, &cost);
        }
    }
}

// Calculate invocation cost based on its complexity
pub fn estimate_invocation_cost(
    costs: Vec<u64>,
    invocation: InvocationComplexity,
    periods: u32,
) -> u64 {
    //resolve base cost based on the invocation type
    let i = invocation as u32 + 1;
    let mut cost = costs.get(i).unwrap_or_default();
    if cost < 1 {
        return 0;
    }
    //charge additional per each loaded period
    if periods > 1 {
        let period_cost = costs.get(0).unwrap_or_default();
        if period_cost > 0 {
            cost = mul_scaled(cost, SCALE + (periods - 1) as u64 * period_cost);
        }
    }
    cost
}

// Multiply two scaled values
#[inline(always)]
fn mul_scaled(value: u64, factor: u64) -> u64 {
    value * factor / SCALE
}
