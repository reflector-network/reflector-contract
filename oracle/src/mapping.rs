use soroban_sdk::{Bytes, Env, Vec};

// Each history record occupies 32 bytes in history mask, allowing to store information for up to 256 recent periods
const RECORD_SIZE: u32 = 32;
const RECORD_SIZE_USIZE: usize = 32;
const MAX_HISTORY_SIZE: usize = 8192; // 256 assets * 32 bytes

// Update history records containing a bitmask of all prices recorded within the last update period
pub fn update_history_mask(
    history_mask: Bytes,
    updates: &Vec<i128>,
    mut updates_delta: u32,
) -> Bytes {
    //create a buffer that can hold the entire history mask
    let mut buffer = [0u8; MAX_HISTORY_SIZE];
    //copy existing history mask into buffer
    let current_len = history_mask.len() as usize;
    history_mask.copy_into_slice(&mut buffer[..current_len]);

    //wipe entire history if the gap between updates is too large
    if updates_delta > 255 {
        buffer.fill(0);
        updates_delta = 1;
    }
    //this should never happen, but just in case
    let delta = if updates_delta < 1 { 1 } else { updates_delta };

    //iterate through all updates and update corresponding history records in the buffer
    for (asset_index, price) in updates.iter().enumerate() {
        let from = asset_index * RECORD_SIZE_USIZE;
        let to = from + RECORD_SIZE_USIZE;

        //256 bits as two 128 (since Rust doesn't have native 256-bit integer type)
        let mut hi = u128::from_be_bytes(buffer[from..from + 16].try_into().unwrap());
        let mut lo = u128::from_be_bytes(buffer[from + 16..to].try_into().unwrap());

        //shift left by delta periods, evicting bits older than 256 periods
        if delta >= 128 {
            hi = lo << (delta - 128);
            lo = 0;
        } else {
            hi = (hi << delta) | (lo >> (128 - delta));
            lo = lo << delta;
        }

        //set lowest bit if price found
        if price > 0 {
            lo |= 1;
        }

        //write back to buffer
        buffer[from..from + 16].copy_from_slice(&hi.to_be_bytes());
        buffer[from + 16..to].copy_from_slice(&lo.to_be_bytes());
    }

    //get total size of updated history mask based on the number of assets and return as Bytes
    let total_size = updates.len() as usize * RECORD_SIZE_USIZE;
    Bytes::from_slice(history_mask.env(), &buffer[..total_size])
}

// Update history records containing a bitmask of all prices recorded within the last update period
pub fn update_history_mask_legacy(
    e: &Env,
    mut history_mask: Bytes,
    updates: &Vec<i128>,
    mut updates_delta: u32,
) -> Bytes {
    //wipe entire history if the gap between updates is too large
    if updates_delta > 255 {
        history_mask = Bytes::new(e); //start with an empty mask
        updates_delta = 1;
    }
    if updates_delta < 1 {
        updates_delta = 1; //this should never happen, but just in case
    }
    //iterate through all updates
    for (asset_index, price) in updates.iter().enumerate() {
        //locate particular asset mask slice position within entire history record
        let offset = asset_index as u32 * RECORD_SIZE;
        //that's new asset, add to the mask
        if offset >= history_mask.len() {
            let empty = [0u8; 32];
            history_mask.extend_from_array(&empty);
        }
        //shift existing mask to the left by the number of periods since the last update
        //all mask bits older than 256 periods get evicted
        if updates_delta <= 255 {
            history_mask = shift_left(history_mask, offset, updates_delta);
        }
        //set corresponding bit if price found
        if price > 0 {
            history_mask = mark_updated(history_mask, offset);
        }
    }
    history_mask //return updated history
}

pub(crate) fn shift_left(mut mask: Bytes, offset: u32, shift: u32) -> Bytes {
    if shift == 0 {
        return mask;
    }
    //if shifting by full bytes or more than available
    if shift > 255 {
        //all bits shifted out, return zeros
        for i in 0..RECORD_SIZE {
            mask.set(offset + i, 0);
        }
        return mask;
    }

    let byte_shift = shift / 8;
    let bit_shift = (shift % 8) as u8;

    if bit_shift == 0 {
        //simple byte shift
        for i in 0..(RECORD_SIZE - byte_shift) {
            let byte = mask.get(offset + i + byte_shift).unwrap();
            mask.set(offset + i, byte);
        }
        //zero out the rest
        for i in (RECORD_SIZE - byte_shift)..RECORD_SIZE {
            mask.set(offset + i, 0);
        }
    } else {
        //shift with bit offset
        let carry_shift = 8 - bit_shift;
        for i in 0..(RECORD_SIZE - byte_shift) {
            let current = mask.get(offset + i + byte_shift).unwrap();
            let shifted = current << bit_shift;
            let carry = if i + byte_shift + 1 < RECORD_SIZE {
                mask.get(offset + i + byte_shift + 1).unwrap() >> carry_shift
            } else {
                0
            };
            mask.set(offset + i, shifted | carry);
        }
        //zero out the rest
        for i in (RECORD_SIZE - byte_shift)..RECORD_SIZE {
            mask.set(offset + i, 0);
        }
    }
    mask
}

pub(crate) fn mark_updated(mut mask: Bytes, offset: u32) -> Bytes {
    let mut carry = 1u8;
    //start from the last byte (least significant) and propagate carry
    for i in (0..RECORD_SIZE).rev() {
        if carry == 0 {
            break;
        }
        let byte = mask.get(offset + i).unwrap();
        let (new_byte, new_carry) = byte.overflowing_add(carry);
        mask.set(offset + i, new_byte);
        carry = if new_carry { 1 } else { 0 };
    }
    mask
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
