use crate::types::{asset::Asset, error::Error};
use crate::{assets, UpdateEvent};
use soroban_sdk::{panic_with_error, Env, Vec};

// Compose and publish price update event
#[inline]
pub fn publish_update_event(e: &Env, updates: &Vec<i128>, timestamp: u64) {
    //load all registered assets
    let assets = assets::load_all_assets(e);
    //validate length
    if assets.len() < updates.len() {
        panic_with_error!(&e, Error::AssetLimitExceeded);
    }
    //prepare update event
    let mut event_updates = Vec::new(&e);
    for (index, asset) in assets.iter().enumerate() {
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
