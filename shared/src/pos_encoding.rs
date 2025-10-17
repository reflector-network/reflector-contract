use soroban_sdk::{Bytes, Env, Vec, U256};

const RECORD_SIZE: u32 = 32;

pub fn update_position_mask(e: &Env, mut mask: Bytes, updates: &Vec<i128>) -> Bytes {
    let one = U256::from_u32(e, 1);
    for (asset_index, price) in updates.iter().enumerate() {
        let from = asset_index as u32 * RECORD_SIZE;
        let to = from + RECORD_SIZE;
        let mut bitmask = if mask.len() >= to {
            let encoded = mask.slice(from..to);
            U256::from_be_bytes(e, &encoded)
        } else {
            U256::from_u32(e, 0)
        };
        bitmask = bitmask.shl(1);
        if price > 0 {
            //set bit if price found
            bitmask = bitmask.add(&one);
        }
        let encoded = bitmask.to_be_bytes();
        if mask.len() <= from {
            mask.append(&encoded);
        } else {
            for i in 0..RECORD_SIZE {
                mask.set(from + i, encoded.get(i).unwrap());
            }
        }
    }
    mask
}


pub fn had_update(mask: &Bytes, asset_index: u32, period: u32) -> bool {
    let from = asset_index * RECORD_SIZE + (RECORD_SIZE - 1 - period / 8);
    let bit = 1 << (period % 8);
    let bytemask = mask.get(from).unwrap_or_default();
    bytemask & bit == bit
}

#[inline]
pub fn locate_update_record_mask_position(asset_index: u32) -> (u32, u8) {
    let byte = asset_index / 8;
    let bitmask = 1 << (asset_index % 8);
    (byte, bitmask)
}

pub fn check_update_record_mask(mask: &Bytes, asset_index: u32) -> bool {
    let (byte, bitmask) = locate_update_record_mask_position(asset_index);
    let bytemask = mask.get(byte).unwrap_or_default();
    bytemask & bitmask == bitmask
}
