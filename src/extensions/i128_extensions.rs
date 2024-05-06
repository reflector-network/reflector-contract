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
}

impl I128Extensions for i128 {
    fn fixed_div_floor(self, y: i128, decimals: u32) -> i128 {
        div_floor(self, y, decimals)
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
