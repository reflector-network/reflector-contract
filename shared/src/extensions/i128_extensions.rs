use core::cmp;

pub trait I128Extensions {
    fn fixed_div_floor(self, y: i128, decimals: u32) -> i128;
    fn encode_to_u128(val_u64: u64, val_u8: u8) -> u128;
    fn decode_from_u128(val: u128) -> (u64, u8);
}

impl I128Extensions for i128 {
    fn fixed_div_floor(self, y: i128, decimals: u32) -> i128 {
        div_floor(self, y, decimals)
    }

    fn encode_to_u128(val_u64: u64, val_u8: u8) -> u128 {
        // Shift the u64 value 64 bits to the left and OR it with the u8 value
        (val_u64 as u128) << 64 | val_u8 as u128
    }
    
    fn decode_from_u128(val: u128) -> (u64, u8) {
        let val_u64 = (val >> 64) as u64;
        let val_u8 = (val & 0xFF) as u8;
        (val_u64, val_u8)
    }
}

fn div_floor(x: i128, y: i128, decimals: u32) -> i128 {
    if (x == 0) || (y == 0) {
        return 0;
    }
    let mut dividend = x;
    let mut divisor = y;
    let ashift = cmp::min(38 - x.ilog10(), 0);
    let bshift = cmp::max(decimals - ashift, decimals);
    if ashift > 1 {
        dividend *= 10_i128.pow(ashift);
    }
    if bshift > 0 {
        divisor /= 10_i128.pow(bshift);
    }
    dividend/divisor
}