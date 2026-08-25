#![cfg(test)]
extern crate std;

use crate::access::TrackEvent;
use crate::{BeamOracleContract, BeamOracleContractClient};
use oracle::init_contract_with_admin;
use oracle::testutils::{generate_updates, normalize_price, register_token, set_ledger_timestamp};
use oracle::types::{Asset, FeeConfig};
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{Address, Event, Map, Vec};

//daily per-asset access rate used across tests
const FEE: i128 = 1_000_000;

#[test]
fn track_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let sponsor = Address::generate(&env);
    let account = Address::generate(&env);
    fee_token.mint(&sponsor, &100_000_000);

    let asset0 = init_data.assets.get_unchecked(0);
    let asset1 = init_data.assets.get_unchecked(1);
    let day = 24 * 60 * 60u64;

    //track two assets: 60M split evenly is 30M per asset, 30 days each
    //ledger timestamp is 900 seconds in the test env
    let ttls = client.track(
        &sponsor,
        &account,
        &Vec::from_array(&env, [asset0.clone(), asset1.clone()]),
        &60_000_000,
    );
    let expected = 900 + 30 * day;
    assert_eq!(ttls, Vec::from_array(&env, [expected, expected]));

    //track event is published
    let expected_event = TrackEvent {
        consumer: account.clone(),
        sponsor: sponsor.clone(),
        amount: 60_000_000,
        assets: {
            let mut tracked = Map::new(&env);
            tracked.set(asset0.clone(), expected);
            tracked.set(asset1.clone(), expected);
            tracked
        },
    };
    assert_eq!(
        env.events().all().events().last().unwrap(),
        &expected_event.to_xdr(&env, &client.address)
    );

    //the full amount is burned from the sponsor
    assert_eq!(fee_token.balance(&sponsor), 40_000_000);

    //the view reflects the purchased access
    assert_eq!(
        client.tracked_until(
            &account,
            &Vec::from_array(&env, [asset0.clone(), asset1.clone()])
        ),
        Vec::from_array(&env, [expected, expected])
    );
    //feed expiration is bumped to cover the purchased access
    assert_eq!(client.expires(&asset0), Some(expected));

    //tracking again stacks on top of the active access
    let ttls = client.track(
        &sponsor,
        &account,
        &Vec::from_array(&env, [asset0.clone()]),
        &10_000_000,
    );
    let extended = 900 + 40 * day;
    assert_eq!(ttls, Vec::from_array(&env, [extended]));
    assert_eq!(client.expires(&asset0), Some(extended));
    //the other asset keeps its earlier expiration
    assert_eq!(
        client.tracked_until(
            &account,
            &Vec::from_array(&env, [asset0.clone(), asset1.clone()])
        ),
        Vec::from_array(&env, [extended, expected])
    );
    assert_eq!(fee_token.balance(&sponsor), 30_000_000);
}

#[test]
fn track_dust_remainder_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let sponsor = Address::generate(&env);
    let account = Address::generate(&env);
    fee_token.mint(&sponsor, &3_001);

    let asset0 = init_data.assets.get_unchecked(0);
    let asset1 = init_data.assets.get_unchecked(1);

    //odd amount over two assets: share floors to 1500, the remainder is burned anyway
    //duration = 1500 * 86_400_000 / 1_000_000 = 129_600 ms;
    //new ttl = 900_000 + 129_600 = 1_029_600 ms -> 1029 seconds (floor)
    let ttls = client.track(
        &sponsor,
        &account,
        &Vec::from_array(&env, [asset0, asset1]),
        &3_001,
    );
    assert_eq!(ttls, Vec::from_array(&env, [1029u64, 1029u64]));
    //the full amount is burned, including the indivisible remainder
    assert_eq!(fee_token.balance(&sponsor), 0);
}

#[test]
fn track_after_expiration_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let sponsor = Address::generate(&env);
    let account = Address::generate(&env);
    fee_token.mint(&sponsor, &100_000_000);

    let asset0 = init_data.assets.get_unchecked(0);
    let day = 24 * 60 * 60u64;
    client.track(
        &sponsor,
        &account,
        &Vec::from_array(&env, [asset0.clone()]),
        &1_000_000,
    );
    assert_eq!(
        client.tracked_until(&account, &Vec::from_array(&env, [asset0.clone()])),
        Vec::from_array(&env, [900 + day])
    );

    //well past the expiration the new access extends from now, not from the stale ttl
    set_ledger_timestamp(&env, 900 + 2 * day);
    let ttls = client.track(
        &sponsor,
        &account,
        &Vec::from_array(&env, [asset0.clone()]),
        &1_000_000,
    );
    assert_eq!(ttls, Vec::from_array(&env, [900 + 3 * day]));
    assert_eq!(
        client.tracked_until(&account, &Vec::from_array(&env, [asset0.clone()])),
        Vec::from_array(&env, [900 + 3 * day])
    );
}

#[test]
fn access_until_untracked_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    //unknown account has no access
    let account = Address::generate(&env);
    let asset0 = init_data.assets.get_unchecked(0);
    let asset1 = init_data.assets.get_unchecked(1);
    let day = 24 * 60 * 60u64;
    assert_eq!(
        client.tracked_until(&account, &Vec::from_array(&env, [asset0.clone()])),
        Vec::from_array(&env, [0u64])
    );

    fee_token.mint(&account, &1_000_000);
    client.track(
        &account,
        &account,
        &Vec::from_array(&env, [asset0.clone()]),
        &1_000_000,
    );
    //an account with access to one asset has none for another, and an unsupported
    //asset reports no access instead of panicking - results are positional
    let unknown = Asset::Stellar(Address::generate(&env));
    assert_eq!(
        client.tracked_until(
            &account,
            &Vec::from_array(&env, [asset1.clone(), asset0.clone(), unknown.clone()])
        ),
        Vec::from_array(&env, [0, 900 + day, 0])
    );

    //empty asset list yields an empty result
    assert_eq!(
        client.tracked_until(&account, &Vec::new(&env)),
        Vec::<u64>::new(&env)
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn track_zero_amount_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    let asset0 = init_data.assets.get_unchecked(0);
    client.track(&account, &account, &Vec::from_array(&env, [asset0]), &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn track_dust_amount_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    fee_token.mint(&account, &1);
    let asset0 = init_data.assets.get_unchecked(0);
    let asset1 = init_data.assets.get_unchecked(1);
    //1 unit over two assets: per-asset share is 0, which buys no time - rejected
    client.track(
        &account,
        &account,
        &Vec::from_array(&env, [asset0, asset1]),
        &1,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn track_excessive_amount_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    let asset0 = init_data.assets.get_unchecked(0);
    //the duration overflows the u64 timestamp range - rejected instead of silently truncated
    client.track(
        &account,
        &account,
        &Vec::from_array(&env, [asset0]),
        &300_000_000_000_000_000,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #103)")]
fn track_empty_assets_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    client.track(&account, &account, &Vec::new(&env), &1_000_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #103)")]
fn track_duplicate_assets_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    fee_token.mint(&account, &10_000_000);
    let asset0 = init_data.assets.get_unchecked(0);
    client.track(
        &account,
        &account,
        &Vec::from_array(&env, [asset0.clone(), asset0.clone()]),
        &10_000_000,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #103)")]
fn track_unsupported_asset_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    fee_token.mint(&account, &10_000_000);
    let unknown = Asset::Stellar(Address::generate(&env));
    client.track(
        &account,
        &account,
        &Vec::from_array(&env, [unknown]),
        &10_000_000,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn track_without_fee_config_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);

    //fee config is FeeConfig::None by default in the test env
    let account = Address::generate(&env);
    let asset0 = init_data.assets.get_unchecked(0);
    client.track(
        &account,
        &account,
        &Vec::from_array(&env, [asset0]),
        &1_000_000,
    );
}

//read the raw access entry size to observe records that the public interface hides
fn access_entry_size(
    env: &soroban_sdk::Env,
    client: &BeamOracleContractClient,
    address: &Address,
) -> u32 {
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .get::<Address, Map<u32, u64>>(address)
            .map(|access| access.len())
            .unwrap_or(0)
    })
}

#[test]
fn track_prunes_expired_access_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    fee_token.mint(&account, &100_000_000);
    let asset0 = init_data.assets.get_unchecked(0);
    let asset1 = init_data.assets.get_unchecked(1);
    let day = 24 * 60 * 60u64;

    //one day of access for the first asset, thirty days for the second one
    client.track(
        &account,
        &account,
        &Vec::from_array(&env, [asset0.clone()]),
        &FEE,
    );
    client.track(
        &account,
        &account,
        &Vec::from_array(&env, [asset1.clone()]),
        &(30 * FEE),
    );
    assert_eq!(access_entry_size(&env, &client, &account), 2);

    //once the first asset access lapses, the next track drops the stale record
    set_ledger_timestamp(&env, 900 + 2 * day);
    client.track(
        &account,
        &account,
        &Vec::from_array(&env, [asset1.clone()]),
        &FEE,
    );
    assert_eq!(access_entry_size(&env, &client, &account), 1);
    //pruning does not touch the record that is still active
    assert_eq!(
        client.tracked_until(
            &account,
            &Vec::from_array(&env, [asset1.clone(), asset0.clone()])
        ),
        Vec::from_array(&env, [900 + 31 * day, 0])
    );
}

//the tests above run with env-level auth mocking, which would hide a missing
//require_auth call - these two invoke the contract without any mocked auth
#[test]
fn track_requires_sponsor_auth_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, false);
    let fee_token = register_token(&env, &init_data.admin);
    client
        .mock_all_auths()
        .set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let sponsor = Address::generate(&env);
    let account = Address::generate(&env);
    fee_token.mock_all_auths().mint(&sponsor, &10_000_000);
    let assets = Vec::from_array(&env, [init_data.assets.get_unchecked(0)]);

    //an unsigned request is rejected before anything is burned or granted
    assert!(client.try_track(&sponsor, &account, &assets, &FEE).is_err());
    assert_eq!(fee_token.balance(&sponsor), 10_000_000);
    assert_eq!(
        client.tracked_until(&account, &assets),
        Vec::from_array(&env, [0u64])
    );

    //the same request goes through once the sponsor authorizes it
    client
        .mock_all_auths()
        .track(&sponsor, &account, &assets, &FEE);
    assert_eq!(fee_token.balance(&sponsor), 10_000_000 - FEE);
}

#[test]
fn read_requires_consumer_auth_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, false);
    let fee_token = register_token(&env, &init_data.admin);
    client
        .mock_all_auths()
        .set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    fee_token.mock_all_auths().mint(&account, &FEE);
    let asset = init_data.assets.get_unchecked(0);
    let updates = generate_updates(&env, &init_data.assets, normalize_price(100));
    client.mock_all_auths().set_price(&updates.0, &600_000);
    client.mock_all_auths().track(
        &account,
        &account,
        &Vec::from_array(&env, [asset.clone()]),
        &FEE,
    );

    //the access is active
    assert_eq!(
        client.tracked_until(&account, &Vec::from_array(&env, [asset.clone()])),
        Vec::from_array(&env, [900 + 24 * 60 * 60])
    );
    //...yet every read still requires the consumer signature
    assert!(client.try_lastprice(&account, &asset).is_err());
    assert!(client.try_price(&account, &asset, &600).is_err());
    assert!(client.try_prices(&account, &asset, &1).is_err());
    //a signed read succeeds
    assert!(client
        .mock_all_auths()
        .lastprice(&account, &asset)
        .is_some());
}

#[test]
fn sponsor_pays_consumer_reads_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let sponsor = Address::generate(&env);
    let account = Address::generate(&env);
    fee_token.mint(&sponsor, &FEE);
    let asset = init_data.assets.get_unchecked(0);
    let updates = generate_updates(&env, &init_data.assets, normalize_price(100));
    client.set_price(&updates.0, &600_000);
    client.track(
        &sponsor,
        &account,
        &Vec::from_array(&env, [asset.clone()]),
        &FEE,
    );

    //the consumer reads prices with the access the sponsor paid for
    assert!(client.lastprice(&account, &asset).is_some());
    //while the sponsor gets no access of its own
    assert_eq!(
        client.tracked_until(&sponsor, &Vec::from_array(&env, [asset.clone()])),
        Vec::from_array(&env, [0u64])
    );
    assert_eq!(access_entry_size(&env, &client, &sponsor), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #102)")]
fn sponsor_read_denied_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let sponsor = Address::generate(&env);
    let account = Address::generate(&env);
    fee_token.mint(&sponsor, &FEE);
    let asset = init_data.assets.get_unchecked(0);
    client.track(
        &sponsor,
        &account,
        &Vec::from_array(&env, [asset.clone()]),
        &FEE,
    );

    //paying for someone else does not entitle the sponsor to read
    client.lastprice(&sponsor, &asset);
}

//seed prices and a funded subscriber with access to the first asset until the given timestamp
fn seed_subscriber(
    env: &soroban_sdk::Env,
    client: &BeamOracleContractClient,
    init_data: &oracle::types::ConfigData,
    fee_token: &soroban_sdk::token::StellarAssetClient,
    account: &Address,
    days: u32,
) -> Asset {
    //record prices for all assets
    let updates = generate_updates(env, &init_data.assets, normalize_price(100));
    client.set_price(&updates.0, &600_000);
    //fund the account and purchase access to the first asset
    let amount = FEE * days as i128;
    fee_token.mint(account, &amount);
    let asset = init_data.assets.get_unchecked(0);
    client.track(
        account,
        account,
        &Vec::from_array(env, [asset.clone()]),
        &amount,
    );
    asset
}

#[test]
fn read_with_access_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    let asset = seed_subscriber(&env, &client, &init_data, &fee_token, &account, 30);

    //all three read functions work while access is active
    assert!(client.lastprice(&account, &asset).is_some());
    //historical lookup is allowed while access is active (price at 600 seconds)
    assert!(client.price(&account, &asset, &600).is_some());
    assert!(client.prices(&account, &asset, &1).is_some());
}

#[test]
fn read_at_exact_expiration_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    let day = 24 * 60 * 60u64;
    let asset = seed_subscriber(&env, &client, &init_data, &fee_token, &account, 30);

    //access is still granted at exactly the expiration timestamp (ttl >= now)
    set_ledger_timestamp(&env, 900 + 30 * day);
    //refresh prices at the boundary so the read is not confounded by the staleness gate
    let updates = generate_updates(&env, &init_data.assets, normalize_price(100));
    client.set_price(&updates.0, &((900 + 30 * day) * 1000));
    assert!(client.lastprice(&account, &asset).is_some());
}

#[test]
#[should_panic(expected = "Error(Contract, #102)")]
fn read_expired_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    let day = 24 * 60 * 60u64;
    let asset = seed_subscriber(&env, &client, &init_data, &fee_token, &account, 30);

    //jump past the access expiration
    set_ledger_timestamp(&env, 900 + 30 * day + 1);
    client.lastprice(&account, &asset);
}

#[test]
#[should_panic(expected = "Error(Contract, #102)")]
fn read_unsubscribed_asset_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    //subscribed to asset 0 only
    seed_subscriber(&env, &client, &init_data, &fee_token, &account, 30);

    client.lastprice(&account, &init_data.assets.get_unchecked(1));
}

#[test]
#[should_panic(expected = "Error(Contract, #102)")]
fn read_without_access_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    //no track - read must be denied
    client.lastprice(&Address::generate(&env), &init_data.assets.get_unchecked(0));
}

//each read fn must be gated independently - a missing check in one of them
//would not be caught by the lastprice-only denial tests above
#[test]
#[should_panic(expected = "Error(Contract, #102)")]
fn read_price_without_access_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    client.price(
        &Address::generate(&env),
        &init_data.assets.get_unchecked(0),
        &600,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #102)")]
fn read_prices_without_access_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    client.prices(
        &Address::generate(&env),
        &init_data.assets.get_unchecked(0),
        &1,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #102)")]
fn read_unsupported_asset_test() {
    let (env, client, init_data) =
        init_contract_with_admin!(BeamOracleContract, BeamOracleContractClient, true);
    let fee_token = register_token(&env, &init_data.admin);
    client.set_fee_config(&FeeConfig::Some((fee_token.address.clone(), FEE)));

    let account = Address::generate(&env);
    seed_subscriber(&env, &client, &init_data, &fee_token, &account, 30);

    client.lastprice(&account, &Asset::Stellar(Address::generate(&env)));
}
