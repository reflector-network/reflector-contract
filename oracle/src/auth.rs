use crate::types::Error;
use soroban_sdk::{panic_with_error, Address, Env};

//storage keys
const ADMIN_KEY: &str = "admin";

// Get current admin account address
#[inline]
pub fn get_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&ADMIN_KEY)
}

// Set current admin account address
#[inline]
pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&ADMIN_KEY, admin);
}

// Throw exception if call hasn't been authorized by admin
#[inline]
pub fn panic_if_not_admin(e: &Env) {
    let admin = get_admin(e);
    if admin.is_none() {
        panic_with_error!(e, Error::Unauthorized);
    }
    admin.unwrap().require_auth()
}
