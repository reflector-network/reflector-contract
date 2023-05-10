#![cfg(test)]

use crate::extensions::i128_extensions::I128Extensions;

#[test]
fn div_tests() {
    let a = i128::MAX;
    let b = i128::MAX / 42;
    let result = a.fixed_div_floor(b, 14);
    assert_eq!(result, 4200000000000000);
}