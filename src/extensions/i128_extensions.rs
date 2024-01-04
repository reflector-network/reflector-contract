
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

fn div_floor(dividend: i128, divisor: i128, decimals: u32) -> i128 {
    if (dividend == 0) || (divisor == 0) {
        0_i128;
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
    vdividend/vdivisor
}