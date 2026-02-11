use soroban_sdk::{Bytes, Vec};

// Each history record occupies 32 bytes in history mask, allowing to store information for up to 256 recent periods
const RECORD_SIZE: u32 = 32;
const URECORD_SIZE: usize = 32;
const MAX_HISTORY_SIZE: usize = 256 * 32; // 256 assets * 32 bytes

// Update history records containing a bitmask of all prices recorded within the last update period
pub fn update_history_mask(
    history_mask: Bytes,
    updates: &Vec<i128>,
    mut updates_delta: u32,
) -> Bytes {
    //create a buffer that can hold the entire history mask
    let mut buffer = [0u8; MAX_HISTORY_SIZE];
    let mask_length = history_mask.len() as usize;

    if updates_delta < 1 {
        updates_delta = 1; //this should never happen, but just in case
    }
    if updates_delta > 255 {
        //entire history is obsolete - ignore
        updates_delta = 1; //reset delta to 1
    } else {
        //copy existing history mask into buffer
        history_mask.copy_into_slice(&mut buffer[..mask_length]);
    }
    for (asset_index, price) in updates.iter().enumerate() {
        //iterate through all updates and update corresponding history records in the buffer
        let offset = asset_index * URECORD_SIZE;

        //256 bits as two 128 parts
        let mut hi = u128::from_be_bytes(buffer[offset..offset + 16].try_into().unwrap());
        let mut lo = u128::from_be_bytes(buffer[offset + 16..offset + 32].try_into().unwrap());

        if lo > 0 || hi > 0 {
            //shift left by the number of periods
            (hi, lo) = if updates_delta < 128 {
                (
                    (hi << updates_delta) | (lo >> (128 - updates_delta)),
                    lo << updates_delta,
                )
            } else {
                (lo << (updates_delta & 0x7f), 0)
            };
        }

        //set lowest bit if price found
        if price > 0 {
            let added = lo.overflowing_add(1);
            lo = added.0;
            if added.1 {
                (hi, _) = hi.overflowing_add(1);
            }
        }
        //write back to buffer
        buffer[offset..offset + 16].copy_from_slice(&hi.to_be_bytes());
        buffer[offset + 16..offset + 32].copy_from_slice(&lo.to_be_bytes());
    }

    //get total size of updated history mask based on the number of assets and return as Bytes
    let updates_length = mask_length.max(updates.len() as usize * URECORD_SIZE);
    Bytes::from_slice(history_mask.env(), &buffer[..updates_length])
}

// Check whether asset price has been quoted for a certain period based on history records bitmask
pub fn check_history_updated(history_mask: &Bytes, asset_index: u32, period: u32) -> bool {
    //locate particular asset mask slice position within entire history record
    let from = asset_index * RECORD_SIZE + (RECORD_SIZE - 1 - period / 8);
    //and calculate specific bit that we need to check
    let bit = 1 << (period % 8);
    //retrieve byte from array
    let encoded_byte = history_mask.get(from).unwrap_or_default();
    //compare with bit mask
    encoded_byte & bit == bit
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
