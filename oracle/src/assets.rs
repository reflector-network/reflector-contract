use crate::types::{Asset, Error, FeeConfig};
use crate::{settings, timestamps};
use soroban_sdk::{panic_with_error, token::TokenClient, Address, Env, Vec};

const ASSET_LIMIT: u32 = 1000; //current limit

//storage keys
const ASSETS_KEY: &str = "assets";
const EXPIRATION_KEY: &str = "expiration";

fn get_expiration_timestamp(e: &Env, initial_expiration_period: u32) -> u64 {
    if initial_expiration_period > 0 {
        return timestamps::ledger_timestamp(&e)
            .checked_add(timestamps::days_to_milliseconds(initial_expiration_period))
            .unwrap();
    }
    0u64
}

// Get all contract assets
pub fn load_all_assets(e: &Env) -> Vec<Asset> {
    e.storage()
        .instance()
        .get(&ASSETS_KEY)
        .unwrap_or_else(|| Vec::new(e))
}

// Load asset index
pub fn resolve_asset_index(e: &Env, asset: &Asset) -> Option<u32> {
    let index: Option<u32>;
    match asset {
        Asset::Stellar(address) => {
            index = e.storage().instance().get(&address);
        }
        Asset::Other(symbol) => {
            index = e.storage().instance().get(&symbol);
        }
    }
    index
}

// Resolve indexes for a pair of assets
pub fn resolve_asset_pair_indexes(
    e: &Env,
    base_asset: Asset,
    quote_asset: Asset,
) -> Option<(u32, u32)> {
    let base_asset = resolve_asset_index(e, &base_asset)?;
    let quote_asset = resolve_asset_index(e, &quote_asset)?;
    Some((base_asset, quote_asset))
}

// Add assets to the oracle
pub fn add_assets(e: &Env, assets: Vec<Asset>, initial_expiration_period: u32) {
    //use default expiration period for new assets
    let expiration_timestamp = get_expiration_timestamp(e, initial_expiration_period);
    //load current state
    let mut asset_list = load_all_assets(e);
    let mut expiration = load_expiration_records(e);
    let is_fee_config_set = settings::get_fee_config(e) != FeeConfig::None;
    //for each new asset
    for asset in assets.iter() {
        //check if the asset has been already added
        if resolve_asset_index(e, &asset).is_some() {
            panic_with_error!(&e, Error::AssetAlreadyExists);
        }
        set_asset_index(e, &asset, asset_list.len());
        asset_list.push_back(asset);
        //if the fee is not set, we don't need to set the expiration
        if is_fee_config_set && expiration_timestamp > 0 {
            expiration.push_back(expiration_timestamp); //set expiration
        }
    }
    if asset_list.len() >= ASSET_LIMIT {
        panic_with_error!(&e, Error::AssetLimitExceeded);
    }
    //update assets list and expirations vector
    e.storage().instance().set(&ASSETS_KEY, &asset_list);
    set_expirations_records(e, &expiration);
}

// Retrieve expiration time for given asset
pub fn expires(e: &Env, asset: Asset) -> Option<u64> {
    let asset_index = resolve_asset_index(e, &asset);
    if asset_index.is_none() {
        e.panic_with_error(Error::AssetMissing);
    }
    let expirations = load_expiration_records(e);
    expirations.get(asset_index.unwrap())
}

// Initialize expiration records for all existing assets
pub fn init_expiration_config(e: &Env, initial_expiration_period: u32) {
    let mut expiration_records = load_expiration_records(e);
    if expiration_records.len() > 0 {
        return; // expiration values for existing price feeds already initialized
    }
    //init expiration, set INITIAL_EXPIRATION_PERIOD for all symbols by default
    let exp = get_expiration_timestamp(e, initial_expiration_period);
    //add records to the expirations vector
    let assets = load_all_assets(e);
    for _ in 0..assets.len() {
        expiration_records.push_back(exp);
    }
    set_expirations_records(e, &expiration_records);
}

// Extend time-to-live for given asset price feed
pub fn extend_ttl(
    e: &Env,
    sponsor: Address,
    asset: Asset,
    amount: i128,
    initial_expiration_period: u32,
) {
    //check if the amount is valid
    if amount <= 0 {
        e.panic_with_error(Error::InvalidAmount);
    }
    //ensure that the asset is supported
    let asset_index = resolve_asset_index(e, &asset);
    if asset_index.is_none() {
        e.panic_with_error(Error::AssetMissing);
    }
    let asset_index = asset_index.unwrap();
    //load required fee amount from retention config
    let (xrf, fee) = match settings::get_fee_config(e) {
        FeeConfig::Some(fee_data) => {
            if fee_data.1 <= 0 {
                e.panic_with_error(Error::InvalidConfigVersion);
            }
            fee_data
        }
        FeeConfig::None => {
            e.panic_with_error(Error::InvalidConfigVersion);
        }
    };
    //burn corresponding amount of fee tokens
    TokenClient::new(&e, &xrf).burn(&sponsor, &amount);
    //calculate extension period
    let bump = amount * 86400000 / fee; // in milliseconds
    if bump <= 0 {
        e.panic_with_error(Error::InvalidAmount);
    }
    //load expiration info
    let mut expiration = load_expiration_records(e);
    let now = timestamps::ledger_timestamp(&e);
    let mut asset_expiration = expiration
        .get(asset_index)
        .unwrap_or_else(|| now + timestamps::days_to_milliseconds(initial_expiration_period));
    //if the asset expiration is not set, or it's already expired - set it to now
    if asset_expiration == 0 || asset_expiration < now {
        asset_expiration = now;
    }
    //bump expiration
    asset_expiration = asset_expiration.checked_add(bump as u64).unwrap();
    //write into the vector that holds expiration dates for all symbols
    expiration.set(asset_index, asset_expiration);
    //update expiration records in instance storage
    set_expirations_records(e, &expiration)
}

// Load expiration data for all assets
fn load_expiration_records(e: &Env) -> Vec<u64> {
    e.storage()
        .instance()
        .get(&EXPIRATION_KEY)
        .unwrap_or_else(|| Vec::new(e))
}

// Set expiration data for all assets
fn set_expirations_records(e: &Env, expiration: &Vec<u64>) {
    e.storage().instance().set(&EXPIRATION_KEY, expiration)
}

// Store asset index
#[inline]
fn set_asset_index(e: &Env, asset: &Asset, index: u32) {
    match asset {
        Asset::Stellar(address) => {
            e.storage().instance().set(&address, &index);
        }
        Asset::Other(symbol) => {
            e.storage().instance().set(&symbol, &index);
        }
    }
}
