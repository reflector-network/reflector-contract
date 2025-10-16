#![cfg(test)]
extern crate alloc;
extern crate std;

use super::*;
use std::panic::{self, AssertUnwindSafe};

#[test]
fn div_tests() {
    let test_cases = [
        (154467226919499, 133928752749774, 115335373284703),
        (
            i128::MAX / 100,
            231731687303715884105728,
            734216306110962248249052545,
        ),
        (231731687303715884105728, i128::MAX / 100, 13),
        // -1 expected result for errors
        (1, 0, -1),
        (0, 1, -1),
        (0, 0, -1),
        (-1, 0, -1),
        (0, -1, -1),
        (-1, -1, -1),
    ];

    for (a, b, expected) in test_cases.iter() {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            prices::fixed_div_floor(a.clone(), *b, 14)
        }));
        if expected == &-1 {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), *expected);
        }
    }
}