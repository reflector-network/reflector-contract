use shared::{settings::XRF_TOKEN_ADDRESS, types::fee_config::FeeConfig};
use soroban_sdk::{Address, Env};

const INVOCATION_KEY: &str = "invocation";
const DEFAULT_INVOCATION_FEE: i128 = 100_000_000;

#[inline]
pub fn set_invocation_config(e: &Env, inv_config: &FeeConfig) {
    e.storage().instance().set(&INVOCATION_KEY, &inv_config);
}

#[inline]
pub fn get_invocation_config(e: &Env) -> FeeConfig {
    e.storage()
        .instance()
        .get(&INVOCATION_KEY)
        .unwrap_or_else(|| {
            FeeConfig::Some((
                Address::from_str(e, XRF_TOKEN_ADDRESS),
                DEFAULT_INVOCATION_FEE,
            ))
        })
}
