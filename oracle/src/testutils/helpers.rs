use super::constants::DECIMALS;

pub fn convert_to_seconds(timestamp: u64) -> u64 {
    timestamp / 1000
}

pub fn normalize_price(price: i128) -> i128 {
    price * 10i128.pow(DECIMALS)
}
