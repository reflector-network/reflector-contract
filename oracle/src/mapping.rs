use soroban_sdk::{Bytes, Env, Vec, U256};

// Each history record occupies 32 bytes in history mask, allowing to store information for up to 256 recent periods
const RECORD_SIZE: u32 = 32;

// Update history records containing a bitmask of all prices recorded within the last update period
pub fn update_history_mask(e: &Env, mut history_mask: Bytes, updates: &Vec<i128>) -> Bytes {
    let one = U256::from_u32(e, 1);
    //iterate through all updates
    for (asset_index, price) in updates.iter().enumerate() {
        //locate particular asset mask slice position within entire history record
        let from = asset_index as u32 * RECORD_SIZE;
        let to = from + RECORD_SIZE;
        //retrieve previous asset mask
        let mut bitmask = if history_mask.len() >= to {
            let encoded = history_mask.slice(from..to);
            U256::from_be_bytes(e, &encoded)
        } else {
            U256::from_u32(e, 0) //no previous records for this asset found
        };
        //shift existing mask, all mask bits older than 256 periods get evicted
        bitmask = bitmask.shl(1);
        //set corresponding bit if price found
        if price > 0 {
            bitmask = bitmask.add(&one);
        }
        //encode into bytes again
        let encoded = bitmask.to_be_bytes();
        //write to the history
        if history_mask.len() <= from {
            //that's new asset, add to the mask
            history_mask.append(&encoded);
        } else {
            //replace bytes
            for i in 0..RECORD_SIZE {
                history_mask.set(from + i, encoded.get(i).unwrap());
            }
        }
    }
    history_mask //return updated history
}

// Check whether asset price has been quoted for a certain period based on history records bitmask
pub fn check_history_updated(history_mask: &Bytes, asset_index: u32, period: u32) -> bool {
    //locate particular asset mask slice position within entire history record
    let from = asset_index * RECORD_SIZE + (RECORD_SIZE - 1 - period / 8);
    //and calculate specific bit that we need to check
    let bit = 1 << (period % 8);
    //retrieve byte from array
    let bytemask = history_mask.get(from).unwrap_or_default();
    //compare with bit mask
    bytemask & bit == bit
}

// Check whether price update record contains update for given asset by its index
pub fn check_period_updated(period_mask: &Bytes, asset_index: u32) -> bool {
    //calculate byte position and bit index to check
    let (byte, bitmask) = resolve_period_update_mask_position(asset_index);
    //retrieve byte from array
    let bytemask = period_mask.get(byte).unwrap_or_default();
    //compare with bit mask
    bytemask & bitmask == bitmask
}

// Calculate byte position and bit index to check in 256-bit update record mask
#[inline]
pub fn resolve_period_update_mask_position(asset_index: u32) -> (u32, u8) {
    //locate particular asset mask position within update record
    let byte = asset_index / 8;
    //and calculate specific bit that we need to check
    let bitmask = 1 << (asset_index % 8);
    (byte, bitmask)
}
