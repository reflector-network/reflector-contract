pub struct U128Helper(u128);

impl U128Helper {
    pub fn encode_price_record_key(val_u64: u64, val_u8: u8) -> u128 {
        (val_u64 as u128) << 64 | val_u8 as u128
    }
}