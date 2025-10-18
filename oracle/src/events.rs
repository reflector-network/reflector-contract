use crate::types::{Asset, Error};
use soroban_sdk::{contractevent, panic_with_error, Env, Val, Vec};

#[contractevent(topics = ["REFLECTOR", "update"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateEvent {
    #[topic]
    pub timestamp: u64,
    pub update_data: Vec<(Val, i128)>,
}

// Compose and publish price update event
#[inline]
pub fn publish_update_event(e: &Env, updates: &Vec<i128>, all_assets: &Vec<Asset>, timestamp: u64) {
    //validate length
    if all_assets.len() < updates.len() {
        panic_with_error!(&e, Error::AssetLimitExceeded);
    }
    //prepare update event
    let mut event_updates = Vec::new(&e);
    for (index, asset) in all_assets.iter().enumerate() {
        //retrieve individual price
        let price = updates.get(index as u32).unwrap_or_default();
        if price == 0 {
            continue; //skip zero prices
        }
        //resolve asset symbol
        let symbol = match asset {
            Asset::Stellar(address) => address.to_val(),
            Asset::Other(symbol) => symbol.to_val(),
        };
        //add to updates vector
        event_updates.push_back((symbol, price));
    }

    //compose and publish price update event
    let event = UpdateEvent {
        timestamp,
        update_data: event_updates,
    };
    e.events().publish_event(&event);
}
