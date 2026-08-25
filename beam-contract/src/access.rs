use oracle::types::{Asset, Error, FeeConfig};
use oracle::{assets, settings, timestamps};
use soroban_sdk::{contracterror, contractevent, token::TokenClient, Address, Env, Map, Vec};

//milliseconds in a day, mirrors the private constant in oracle::assets
const DAY: i128 = 86_400_000;
//approximate ledger close time in milliseconds
const LEDGER_TIME: u64 = 5000;
//minimum rent extension for touched entries (~30 days of ledgers)
const MIN_EXTENSION: u32 = 518_400;

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
// Access-specific contract errors
pub enum AccessError {
    // Caller cannot read prices
    AccessDenied = 102,
    // Invalid tracking request
    InvalidRequest = 103,
}

#[contractevent(topics = ["REFLECTOR", "track"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackEvent {
    #[topic]
    pub consumer: Address,
    pub sponsor: Address,
    pub amount: i128,
    // Tracked assets mapped to new access expiration timestamps (in seconds)
    pub assets: Map<Asset, u64>,
}

// Purchase access to asset price feeds (XRF token amount charged from the sponsor and split evenly
// between all assets)
pub fn track(
    e: &Env,
    sponsor: Address,
    consumer: Address,
    track_assets: Vec<Asset>,
    amount: i128,
) -> Vec<u64> {
    sponsor.require_auth();
    if track_assets.is_empty() {
        e.panic_with_error(AccessError::InvalidRequest);
    }
    if amount <= 0 {
        e.panic_with_error(Error::InvalidAmount);
    }
    //convert the per-asset share to access time at the daily rate
    let (fee_token, fee) = load_fee_settings(e);
    let share = amount / track_assets.len() as i128;
    let duration = share * DAY / fee; //in milliseconds
    if duration <= 0 {
        e.panic_with_error(Error::InvalidAmount);
    }
    //reject durations that don't fit the timestamp range (as-casts truncate silently)
    let duration =
        u64::try_from(duration).unwrap_or_else(|_| e.panic_with_error(Error::InvalidAmount));
    //resolve and validate all requested assets before any side effects
    let all_assets = assets::load_all_assets(e);
    let mut indexes: Vec<u32> = Vec::new(e);
    let mut seen: Map<u32, bool> = Map::new(e);
    for asset in track_assets.iter() {
        //ensure the asset is supported
        let index = match all_assets.first_index_of(&asset) {
            Some(index) => index,
            None => {
                e.panic_with_error(AccessError::InvalidRequest);
            }
        };
        //reject duplicates
        if seen.contains_key(index) {
            e.panic_with_error(AccessError::InvalidRequest);
        }
        seen.set(index, true);
        indexes.push_back(index);
    }
    //burn the full amount from the sponsor, including the indivisible remainder
    TokenClient::new(e, &fee_token).burn(&sponsor, &amount);
    //extend access for every requested asset
    let now = timestamps::ledger_timestamp(e);
    let mut access = prune_expired_access(e, &consumer, now);
    let mut tracked = Map::new(e); //new expirations in seconds, for the event
    let mut result = Vec::new(e);
    let mut expirations = Vec::new(e); //feed expirations to bump, applied in one batch
    let mut horizon = 0u64;
    for (index, asset) in indexes.iter().zip(track_assets.iter()) {
        //extend from the remaining time if the access is still active
        let current = access.get(index).unwrap_or(0);
        let new_ttl = current.max(now) + duration;
        access.set(index, new_ttl);
        //make sure the feed itself outlives the purchased access
        expirations.push_back((index, new_ttl));
        tracked.set(asset, new_ttl / 1000);
        result.push_back(new_ttl / 1000);
        if new_ttl > horizon {
            horizon = new_ttl;
        }
    }
    assets::ensure_expirations(e, &expirations);
    save_access(e, &consumer, &access);
    extend_entry_ttl(e, &consumer, horizon);
    e.events().publish_event(&TrackEvent {
        consumer,
        sponsor,
        amount,
        assets: tracked,
    });
    result
}

// Return access expiration timestamp for the given consumer and assets (0 if no access was tracked)
pub fn access_until(e: &Env, address: Address, check_assets: Vec<Asset>) -> Vec<u64> {
    let access = load_access(e, &address).unwrap_or(Map::new(e));
    //load the asset list once and resolve every requested asset against it
    let all_assets = assets::load_all_assets(e);
    let mut res = Vec::new(e);
    for asset in check_assets.iter() {
        match all_assets.first_index_of(&asset) {
            Some(index) => res.push_back(access.get(index).unwrap_or(0) / 1000),
            None => res.push_back(0u64),
        }
    }
    res
}

// Verify that the consumer is entitled to read prices for the asset
pub fn check_access(e: &Env, consumer: &Address, asset: &Asset) {
    //unknown assets are denied outright
    if let Some(index) = assets::resolve_asset_index(e, asset) {
        let now = timestamps::ledger_timestamp(e);
        if let Some(access) = load_access(e, consumer) {
            if access.get(index).unwrap_or(0) >= now {
                return;
            }
        }
    }
    e.panic_with_error(AccessError::AccessDenied);
}

// Load tracked access entry for the address.
// Note: access entries occupy the bare-Address persistent keyspace of this
// contract - any future persistent storage must use a distinct wrapper key type
fn load_access(e: &Env, address: &Address) -> Option<Map<u32, u64>> {
    e.storage().persistent().get(address)
}

// Load the access entry for the address, dropping the records that have already expired.
fn prune_expired_access(e: &Env, address: &Address, now: u64) -> Map<u32, u64> {
    let mut pruned = Map::new(e);
    if let Some(access) = load_access(e, address) {
        for (index, ttl) in access.iter() {
            if ttl >= now {
                pruned.set(index, ttl);
            }
        }
    }
    pruned
}

// Save tracked access entry for the address
fn save_access(e: &Env, address: &Address, access: &Map<u32, u64>) {
    e.storage().persistent().set(address, access);
}

// Load fee token and daily per-asset rate from config
fn load_fee_settings(e: &Env) -> (Address, i128) {
    match settings::get_fee_config(e) {
        FeeConfig::Some(fee_data) => {
            if fee_data.1 <= 0 {
                e.panic_with_error(Error::InvalidConfig);
            }
            fee_data
        }
        FeeConfig::None => {
            e.panic_with_error(Error::InvalidConfig);
        }
    }
}

// Extend ledger rent for the entry to cover the access horizon
fn extend_entry_ttl(e: &Env, key: &Address, horizon: u64) {
    let now = timestamps::ledger_timestamp(e);
    let mut ledgers = MIN_EXTENSION;
    if horizon > now {
        let target = u32::try_from((horizon - now) / LEDGER_TIME).unwrap_or(u32::MAX);
        if target > ledgers {
            ledgers = target;
        }
    }
    //clamp to the maximum allowed entry lifetime
    let max = e.storage().max_ttl();
    if ledgers > max {
        ledgers = max;
    }
    e.storage().persistent().extend_ttl(key, ledgers, ledgers);
}
