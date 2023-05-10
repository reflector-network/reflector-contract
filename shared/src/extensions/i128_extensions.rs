use core::cmp;

pub trait I128Extensions {
    fn fixed_div_floor(self, y: i128, decimals: u32) -> i128;
}

impl I128Extensions for i128 {
    fn fixed_div_floor(self, y: i128, decimals: u32) -> i128 {
        div_floor(self, y, decimals)
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