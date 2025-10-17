use crate::timestamps;
use soroban_sdk::Env;

//current protocol version
pub const CURRENT_PROTOCOL: u32 = 2;

//storage keys
const UPDATE_TS_KEY: &str = "protocol_update";
const PROTOCOL_KEY: &str = "protocol";

// Load current protocol version
#[inline(always)]
pub fn get_protocol_version(e: &Env) -> u32 {
    e.storage().instance().get(&PROTOCOL_KEY).unwrap_or(1)
}

// Set current protocol version
#[inline(always)]
pub fn set_protocol_version(e: &Env, protocol: u32) {
    e.storage().instance().set(&PROTOCOL_KEY, &protocol);
}

// Check whether the oracle already uses the latest protocol version and if not - schedule the upgrade
pub fn at_latest_protocol_version(e: &Env) -> bool {
    //load current protocol version
    let protocol = get_protocol_version(e);
    //already at the latest version
    if protocol == CURRENT_PROTOCOL {
        return true;
    }
    schedule_update(e)
}

// Schedule protocol update
fn schedule_update(e: &Env) -> bool {
    //get current ledger ts
    let ledger_timestamp = timestamps::ledger_timestamp(&e);
    let scheduled_update_ts = e.storage().instance().get(&UPDATE_TS_KEY).unwrap_or(0);
    if scheduled_update_ts == 0 {
        set_protocol_upgrade_timestamp(e, ledger_timestamp); //set update timestamp to now if not set
        return false;
    }
    //upgrade protocol to current version if the upgrade timestamp is older than 1 day
    if scheduled_update_ts + timestamps::days_to_milliseconds(1) < ledger_timestamp {
        set_protocol_version(e, CURRENT_PROTOCOL);
        set_protocol_upgrade_timestamp(e, 0); // reset update timestamp
        return true; //now we are at the latest protocol version
    }
    false
}

fn set_protocol_upgrade_timestamp(e: &Env, timestamp: u64) {
    e.storage().instance().set(&UPDATE_TS_KEY, &timestamp);
}
