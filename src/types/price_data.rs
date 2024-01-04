use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
/// The price data for an asset at a given timestamp.
pub struct PriceData {
    /// The price in contracts' base asset and decimals.
    pub price: i128,
    /// The timestamp of the price.
    pub timestamp: u64,
}
