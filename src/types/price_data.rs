use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
// Asset price data at specific timestamp
pub struct PriceData {
    // Price stored with configured decimals places
    pub price: i128,
    // Record timestamp
    pub timestamp: u64,
}
