#![cfg(test)]
extern crate alloc;
extern crate std;

use crate::types::{Asset, PriceUpdate};
use alloc::string::ToString;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, Env, Symbol, Vec};

pub(super) const RESOLUTION: u32 = 300_000;
pub(super) const DECIMALS: u32 = 14;

