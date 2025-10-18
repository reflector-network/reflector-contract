use crate::types::{PriceData, PriceUpdate};
use crate::{mapping, protocol, settings, timestamps};
use soroban_sdk::{Bytes, Env, Vec};

const CACHE_KEY: &str = "cache";
const LAST_TIMESTAMP_KEY: &str = "last_timestamp";
const HISTORY_KEY: &str = "history";

fn normalize_price_data(price: i128, timestamp: u64) -> PriceData {
    PriceData {
        price,
        timestamp: timestamp / 1000, //convert to seconds
    }
}

// Get last known record timestamp
pub fn obtain_last_record_timestamp(e: &Env) -> u64 {
    let last_timestamp = get_last_timestamp(e);
    let ledger_timestamp = timestamps::ledger_timestamp(&e);
    let resolution = settings::get_resolution(e) as u64;
    if last_timestamp == 0 //no prices yet
        || last_timestamp > ledger_timestamp //last timestamp is in the future
        || ledger_timestamp - last_timestamp >= resolution * 2
    //last timestamp is too far in the past, so we cannot return the last price
    {
        return 0;
    }
    last_timestamp
}

// Retrieve price from record for specific asset
pub fn retrieve_asset_price_data(e: &Env, asset: u32, timestamp: u64) -> Option<PriceData> {
    //if protocol version < 2, use legacy method
    if !protocol::at_latest_protocol_version(e) {
        let price = get_price_v1(e, asset as u8, timestamp)?;
        return Some(normalize_price_data(price, timestamp));
    }
    let last = get_last_timestamp(e);
    //get the timestamp index in the bitmask
    if last < timestamp {
        return None;
    }
    let mut period = 0;
    if last > timestamp {
        period = (last - timestamp) / settings::get_resolution(e) as u64;
    }
    if period > 255 {
        return None; //we cannot track more than 256 updates in the bitmask
    }
    if !has_price(e, asset, period as u32) {
        return None; //no price record
    }
    //load the prices for the timestamp
    let record = load_history_record(e, timestamp)?;
    //get price for the asset index
    let price = extract_single_update_record_price(&record, asset);
    Some(normalize_price_data(price, timestamp))
}

// Extract prices for all assets from update record
pub fn extract_update_record_prices(e: &Env, update: &PriceUpdate, total: u32) -> Vec<i128> {
    let mut res = Vec::new(&e);
    let mut update_index = 0;
    for asset_index in 0..total {
        let mut price = 0;
        if mapping::check_period_updated(&update.mask, asset_index) {
            //set price from the update record
            price = update.prices.get_unchecked(update_index);
            update_index += 1;
        }
        res.push_back(price);
    }
    res
}

fn extract_single_update_record_price(update: &PriceUpdate, asset_index: u32) -> i128 {
    let mut update_index = 0;
    for asset in 0..asset_index + 1 {
        if mapping::check_period_updated(&update.mask, asset) {
            if asset == asset_index {
                return update.prices.get_unchecked(update_index);
            }
            update_index += 1;
        }
    }
    0
}

// Load last update timestamp
pub fn get_last_timestamp(e: &Env) -> u64 {
    //get the marker
    e.storage()
        .instance()
        .get(&LAST_TIMESTAMP_KEY)
        .unwrap_or_default()
}

// Store last update timestamp
pub fn set_last_timestamp(e: &Env, timestamp: u64) {
    e.storage().instance().set(&LAST_TIMESTAMP_KEY, &timestamp);
}

// Load history mask containing the map of all periods that had price updates
fn get_history_map(e: &Env) -> Bytes {
    e.storage()
        .instance()
        .get(&HISTORY_KEY)
        .unwrap_or_else(|| Bytes::new(e))
}

//
pub fn update_history_mask(e: &Env, prices: &Vec<i128>, timestamp: u64) {
    //load state
    let last_timestamp = get_last_timestamp(e);
    let mut history_map = get_history_map(e);
    let resolution = settings::get_resolution(e) as u64;
    //find the delta in updates
    let mut update_delta = 0;
    if last_timestamp > 0 && timestamp > last_timestamp {
        update_delta = (timestamp - last_timestamp) / resolution;
    }
    //add missing intervals
    if update_delta > 1 {
        for _ in 1..update_delta {
            let mut empty_prices = Vec::new(e);
            for _ in 0..prices.len() {
                empty_prices.push_back(0i128);
            }
            history_map = mapping::update_history_mask(e, history_map, &empty_prices);
        }
    }

    //update the position mask
    history_map = mapping::update_history_mask(e, history_map, prices);

    //store updated timestamps
    e.storage().instance().set(&HISTORY_KEY, &history_map);
}

pub fn has_price(e: &Env, asset_index: u32, periods_ago: u32) -> bool {
    let timestamps = get_history_map(e);
    mapping::check_history_updated(&timestamps, asset_index, periods_ago)
}

// Load prices for a given timestamp
pub fn load_history_record(e: &Env, timestamp: u64) -> Option<PriceUpdate> {
    //check if the timestamp is in the cache
    let cache = load_price_records_cache(e);
    if cache.is_some() {
        //check the cache first
        for (ts, prices) in cache.unwrap() {
            if ts == timestamp {
                return Some(prices);
            }
        }
    }
    //get the price from the temporary storage
    e.storage().temporary().get(&timestamp)
}

// Update prices stored in the oracle
pub fn store_prices(e: &Env, update: &PriceUpdate, timestamp: u64, update_v1: &Vec<i128>) {
    //get the last timestamp
    let last_timestamp = get_last_timestamp(e);
    //update the last timestamp
    if timestamp > last_timestamp {
        set_last_timestamp(e, timestamp);
    }

    //set the price
    let temps_storage = e.storage().temporary();
    temps_storage.set(&timestamp, &update);
    //update cache
    let cache_size = settings::get_cache_size(e);
    if cache_size > 0 {
        //if cache size is non-empty, store it in the instance
        let mut cache = load_price_records_cache(e).unwrap_or(Vec::new(&e));
        cache.push_front((timestamp, update.clone()));
        while cache.len() > cache_size {
            cache.pop_back(); //remove the oldest record if cache size exceeded
        }
        //write cache entry
        e.storage().instance().set(&CACHE_KEY, &cache);
    }
    //calculate TTL
    let retention_period = settings::get_history_retention_period(e);
    let ledgers_to_live = ((retention_period / 1000 / 5 + 1) * 2) as u32;
    //bump if needed
    if ledgers_to_live > 16 {
        //16 ledgers is the minimum extension period
        temps_storage.extend_ttl(&timestamp, ledgers_to_live, ledgers_to_live)
    }

    //if the protocol hasn't updated to the latest version yet
    if !protocol::at_latest_protocol_version(e) {
        store_price_v1(e, update_v1, timestamp, ledgers_to_live);
    }
}

// Load requested number of price records with a price function callback
pub fn load_prices<F: Fn(u64) -> Option<PriceData>>(
    e: &Env,
    get_price_fn: F,
    mut records: u32,
) -> Option<Vec<PriceData>> {
    let mut timestamp = obtain_last_record_timestamp(e);
    if timestamp == 0 {
        return None;
    }

    let mut prices = Vec::new(e);
    let resolution = settings::get_resolution(e) as u64;

    //limit the number of returned records to 20
    records = records.min(20);

    while records > 0 {
        //invoke price fetch callback for each record
        if let Some(price) = get_price_fn(timestamp) {
            prices.push_back(price);
        }
        if timestamp < resolution {
            break;
        }
        //decrement remaining records counter in every iteration
        records -= 1;
        timestamp -= resolution;
    }

    if prices.is_empty() {
        None
    } else {
        Some(prices)
    }
}

// Calculate TWAP approximation from loaded price range
pub fn calculate_twap<F: Fn(u64) -> Option<PriceData>>(
    e: &Env,
    get_price_fn: F,
    records: u32,
) -> Option<i128> {
    let prices = load_prices(&e, get_price_fn, records)?;

    if prices.len() != records {
        return None;
    }

    let last_price_timestamp = prices.first()?.timestamp * 1000; //convert to milliseconds to match the timestamp format
    let timeframe = settings::get_resolution(e) as u64;
    let current_time = timestamps::ledger_timestamp(&e);

    //check if the last price is too old
    if last_price_timestamp + timeframe + 60 * 1000 < current_time {
        return None;
    }

    let sum: i128 = prices.iter().map(|price_data| price_data.price).sum();
    Some(sum / prices.len() as i128)
}

// Load prices for a pair of assets
pub fn load_cross_price(
    e: &Env,
    asset_pair_indexes: (u32, u32),
    timestamp: u64,
    decimals: u32,
) -> Option<PriceData> {
    //get the asset indexes
    let (base_asset, quote_asset) = asset_pair_indexes;
    //check if the asset are the same
    if base_asset == quote_asset {
        return Some(normalize_price_data(10i128.pow(decimals), timestamp));
    }
    //get the price for base_asset
    let base_asset_price = retrieve_asset_price_data(e, base_asset, timestamp)?;
    //get the price for quote_asset
    let quote_asset_price = retrieve_asset_price_data(e, quote_asset, timestamp)?;

    //calculate the cross price
    Some(normalize_price_data(
        fixed_div_floor(base_asset_price.price, quote_asset_price.price, decimals),
        timestamp,
    ))
}

// Get cached records from the instance storage
fn load_price_records_cache(e: &Env) -> Option<Vec<(u64, PriceUpdate)>> {
    e.storage().instance().get(&CACHE_KEY)
}

// Update price in legacy format (deprecated)
pub fn store_price_v1(e: &Env, updates: &Vec<i128>, timestamp: u64, ledgers_to_live: u32) {
    //iterate over the updates
    for (i, price) in updates.iter().enumerate() {
        //ignore zero prices
        if price == 0 {
            continue;
        }
        let asset = i as u8;

        //build key for price record
        let data_key = format_price_key_v1(asset, timestamp);
        //store new price
        let temp_storage = e.storage().temporary();
        temp_storage.set(&data_key, &price);
        if ledgers_to_live > 16 {
            //16 ledgers is the minimum extension period
            temp_storage.extend_ttl(&data_key, ledgers_to_live, ledgers_to_live)
        }
    }
}

// Load price in legacy format (deprecated)
pub fn get_price_v1(e: &Env, asset: u8, timestamp: u64) -> Option<i128> {
    //load the price from temporary storage
    e.storage()
        .temporary()
        .get(&format_price_key_v1(asset, timestamp))
}

// (deprecated)
fn format_price_key_v1(asset: u8, timestamp: u64) -> u128 {
    (timestamp as u128) << 64 | asset as u128
}

// Div+floor with a specified precision
pub fn fixed_div_floor(dividend: i128, divisor: i128, decimals: u32) -> i128 {
    if dividend <= 0 || divisor <= 0 {
        panic!("invalid division arguments")
    }
    let ashift = core::cmp::min(38 - dividend.ilog10(), decimals);
    let bshift = core::cmp::max(decimals - ashift, 0);

    let mut vdividend = dividend;
    let mut vdivisor = divisor;
    if ashift > 0 {
        vdividend *= 10_i128.pow(ashift);
    }
    if bshift > 0 {
        vdivisor /= 10_i128.pow(bshift);
    }
    vdividend / vdivisor
}
