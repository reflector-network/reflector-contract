use crate::types::{
    asset::Asset, config_data::ConfigData, error::Error, retention_config::RetentionConfig,
};
use soroban_sdk::{Address, Env};

const RETENTION_PERIOD_KEY: &str = "period";
const BASE_KEY: &str = "base_asset";
const DECIMALS_KEY: &str = "decimals";
const RESOLUTION_KEY: &str = "resolution";
const RETENTION_KEY: &str = "retention";
const CACHE_SIZE_KEY: &str = "cache_size";

const XRF_TOKEN_ADDRESS: &str = "CBLLEW7HD2RWATVSMLAGWM4G3WCHSHDJ25ALP4DI6LULV5TU35N2CIZA";
const DEFAULT_RETENTION_FEE: i128 = 100_000_000;

#[inline]
pub fn init(e: &Env, config: &ConfigData) {
    //do not allow to initialize more than once
    if e.storage().instance().has(&RETENTION_PERIOD_KEY) {
        e.panic_with_error(Error::AlreadyInitialized);
    }
    let instance = e.storage().instance();
    //initialized only once and cannot be changed in the future
    instance.set(&BASE_KEY, &config.base_asset);
    instance.set(&DECIMALS_KEY, &config.decimals);
    set_resolution(e, config.resolution);
    set_history_retention_period(e, config.history_retention_period);
    set_cache_size(e, config.cache_size);
    set_retention_config(e, &config.retention_config);
}

#[inline]
pub fn get_base_asset(e: &Env) -> Asset {
    e.storage().instance().get(&BASE_KEY).unwrap()
}

#[inline]
pub fn get_decimals(e: &Env) -> u32 {
    e.storage().instance().get(&DECIMALS_KEY).unwrap()
}

#[inline]
pub fn get_resolution(e: &Env) -> u32 {
    e.storage().instance().get(&RESOLUTION_KEY).unwrap()
}

#[inline]
pub fn set_resolution(e: &Env, resolution: u32) {
    e.storage().instance().set(&RESOLUTION_KEY, &resolution)
}

#[inline]
pub fn get_history_retention_period(e: &Env) -> u64 {
    e.storage()
        .instance()
        .get(&RETENTION_PERIOD_KEY)
        .unwrap_or_default()
}

#[inline]
pub fn set_history_retention_period(e: &Env, retention_period: u64) {
    e.storage()
        .instance()
        .set(&RETENTION_PERIOD_KEY, &retention_period);
}

#[inline]
pub fn get_cache_size(e: &Env) -> u32 {
    e.storage().instance().get(&CACHE_SIZE_KEY).unwrap_or(2)
}

#[inline]
pub fn set_cache_size(e: &Env, cache_size: u32) {
    e.storage().instance().set(&CACHE_SIZE_KEY, &cache_size);
}

#[inline]
pub fn set_retention_config(e: &Env, retention_config: &RetentionConfig) {
    e.storage()
        .instance()
        .set(&RETENTION_KEY, &retention_config);
}

#[inline]
pub fn get_retention_config(e: &Env) -> RetentionConfig {
    e.storage()
        .instance()
        .get(&RETENTION_KEY)
        .unwrap_or_else(|| {
            RetentionConfig::Some((
                Address::from_str(e, XRF_TOKEN_ADDRESS),
                DEFAULT_RETENTION_FEE,
            ))
        })
}
