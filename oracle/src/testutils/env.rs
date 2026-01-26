use soroban_sdk::{
    testutils::{Ledger, LedgerInfo},
    token::StellarAssetClient,
    Address,
};

pub fn register_token<'a>(env: &soroban_sdk::Env, admin: &Address) -> StellarAssetClient<'a> {
    let asset_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let fee_asset = asset_contract.address();
    StellarAssetClient::new(&env, &fee_asset)
}

pub fn set_ledger_timestamp(env: &soroban_sdk::Env, timestamp: u64) {
    let ledger_info = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp,
        ..ledger_info
    });
}
