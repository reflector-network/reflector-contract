pub trait I128Extensions {
    // Divides two i128 numbers, considering decimal places.
    //
    // Arguments:
    // - self: The dividend.
    // - y: The divisor. Should not be zero; will cause panic if zero.
    // - decimals: Number of decimal places for division.
    //
    // Behavior:
    // - Rounds up towards zero for negative results.
    //
    // Panic:
    // - If dividend (self) or divisor (y) is zero.
    //
    // Returns:
    // - Division result with specified rounding behavior.
    fn fixed_div_floor(self, y: i128, decimals: u32) -> i128;

    // Encodes a pair of values (u64 and u8) into a single u128 value.
    //
    // Arguments:
    // - val_u64: The first value, a u64, to be encoded.
    // - val_u8: The second value, a u8, to be encoded.
    //
    // Returns:
    // - A u128 value combining the u64 and u8 values.
    fn encode_to_u128(val_u64: u64, val_u8: u8) -> u128;

    // Decodes a u128 value into a tuple of (u64, u8).
    //
    // Arguments:
    // - val: The u128 value to be decoded.
    //
    // Returns:
    // - A tuple consisting of a u64 and u8, extracted from the u128 value.
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
