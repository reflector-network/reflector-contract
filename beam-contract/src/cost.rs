use oracle::settings;
use oracle::types::FeeConfig;
use soroban_sdk::{contracttype, token, Address, Env, Vec};

const COST_CONFIG_KEY: &str = "cost";
const SCALE: i128 = 10_000_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvocationComplexity {
    //Multiplicator for number of requested periods, not utilized directly for cost calculation
    NModifier = 0,
    //Single asset price record request
    Price = 1,
    //TWAP approximation over N records
    Twap = 2,
    //Cross-price calculation for two assets
    CrossPrice = 3,
    //TWAP approximation over N records for cross-price quote
    CrossTwap = 4,
}
//invocation cost config is stored as vector with indexes corresponding to InvocationComplexity

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
                e, // RecordsModifier, Price, Twap, CrossPrice, CrossTwap
                [2_000_000, 10_000_000, 15_000_000, 20_000_000, 30_000_000],
            )
        })
}

// Charge per-invocation fee
pub fn charge_invocation_fee(
    e: &Env,
    caller: &Address,
    invocation: InvocationComplexity,
    periods: u32,
) {
    //load fee config
    let fee_config = settings::get_fee_config(e);
    if let FeeConfig::Some((fee_token, _)) = fee_config.clone() {
        //calculate amount to charge
        let cost = estimate_invocation_cost(e, invocation, periods, fee_config);
        if cost <= 0 {
            return;
        }
        //init fee token client
        let fee_client = token::Client::new(e, &fee_token);
        //burn tokens
        fee_client.burn(caller, &cost);
    }
}

// Estimate invocation cost based on its complexity and fee config
pub fn estimate_invocation_cost(
    e: &Env,
    invocation: InvocationComplexity,
    periods: u32,
    fee_config: FeeConfig,
) -> i128 {
    match fee_config {
        FeeConfig::None => 0,
        FeeConfig::Some(_) => {
            //load rates
            let costs = load_costs_config(e);
            //calculate amount to charge
            //resolve base cost based on the invocation type
            let mut cost = costs.get(invocation as u32).unwrap_or_default() as i128;
            if cost < 1 {
                return 0;
            }
            //charge additional per each loaded period
            if periods > 1 {
                let period_modifier = costs
                    .get(InvocationComplexity::NModifier as u32)
                    .unwrap_or_default() as i128;
                if period_modifier > 0 {
                    cost = cost * (SCALE + (periods - 1) as i128 * period_modifier) / SCALE;
                }
            }
            cost
        }
    }
}
