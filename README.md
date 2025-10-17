# Reflector oracle smart contracts

This contract implementation is fully compatible with 
[SEP-40](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0040.md) ecosystem standard.
Check the standard for general info and public consumer interface documentation.

### Pulse contract

The Pulse contract provides free access to the latest asset prices and time-weighted average prices (TWAP) for on-chain applications.

### Beam contract

The Beam contract is designed for faster updates of the prices, and charges a fee for each price query.

## Usage example

### **Pulse contract**

### Invocation from consumer contract

#### Utilize this example to invoke oracles from your contract code.

```rust
/* contract.rs */
use crate::reflector::{ReflectorClient, Asset as ReflectorAsset}; // Import Reflector interface
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};
#[contract]
pub struct MyAwesomeContract; // Of course, it's awesome, we know it!
#[contractimpl]
impl MyAwesomeContract {
    pub fn lets_rock(e: Env) {
        // Oracle contract address to use
        let oracle_address = Address::from_str(&e, "CAFJZQWSED6YAWZU3GWRTOCNPPCGBN32L7QV43XX5LZLFTK6JLN34DLN");
        // Create client for working with oracle
        let reflector_client = ReflectorClient::new(&e, &oracle_address);
        // Ticker to lookup the price
        let ticker = ReflectorAsset::Other(Symbol::new(&e, &("BTC")));
        // Fetch the most recent price record for it
        let recent = reflector_client.lastprice(&ticker);
        // Check the result
        if recent.is_none() {
            //panic_with_error!(&e, "price not available");
        }
        // Retrieve the price itself
        let price = recent.unwrap().price;
        // Do not forget for price precision, get decimals from the oracle
        // (this value can be also hardcoded once the price feed has been
        // selected because decimals never change in live oracles)
        let price_decimals = reflector_client.decimals();
        
        // Let's check how much of quoted asset we can potentially purchase for $10
        let usd_balance = 10_0000000i128; // $10 with standard Stellar token precision
        let can_purchase = (usd_balance * 10i128.pow(price_decimals)) / price;
        
        // How many USD we'll need to buy 5 quoted asset tokens?
        let want_purchase = 5_0000000i128; // 5 tokens with standard Stellar token precision
        let need_usd = (want_purchase * price) / 10i128.pow(price_decimals);
        
        // Please note: check for potential overflows or use safe math when dealing with prices
    }
}
```

### Interface for Pulse contract

#### Copy and save it in your smart contract project as "reflector_pulse.rs" file. This is the oracle client..

```rust
/* reflector.rs */
use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

// Oracle contract interface exported as ReflectorClient
#[soroban_sdk::contractclient(name = "ReflectorClient")]
pub trait Contract {
    // Base oracle symbol the price is reported in
    fn base(e: Env) -> Asset;
    // All assets quoted by the contract
    fn assets(e: Env) -> Vec<Asset>;
    // Number of decimal places used to represent price for all assets quoted by the oracle
    fn decimals(e: Env) -> u32;
    // Quotes asset price in base asset at specific timestamp
    fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData>;
    // Quotes the most recent price for an asset
    fn lastprice(e: Env, asset: Asset) -> Option<PriceData>;
    // Quotes last N price records for the given asset
    fn prices(e: Env, asset: Asset, records: u32) -> Option<Vec<PriceData>>;
    // Quotes the most recent cross price record for the pair of assets
    fn x_last_price(e: Env, base_asset: Asset, quote_asset: Asset) -> Option<PriceData>;
    // Quotes the cross price for the pair of assets at specific timestamp
    fn x_price(e: Env, base_asset: Asset, quote_asset: Asset, timestamp: u64) -> Option<PriceData>;
    // Quotes last N cross price records of for the pair of assets
    fn x_prices(e: Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<Vec<PriceData>>;
    // Quotes the time-weighted average price for the given asset over N recent records
    fn twap(e: Env, asset: Asset, records: u32) -> Option<i128>;
    // Quotes the time-weighted average cross price for the given asset pair over N recent records
    fn x_twap(e: Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<i128>;
    // Price feed resolution (default tick period timeframe, in seconds - 5 minutes by default)
    fn resolution(e: Env) -> u32;
    // Historical records retention period, in seconds (24 hours by default)
    fn history_retention_period(e: Env) -> Option<u64>;
    // The most recent price update timestamp
    fn last_timestamp(e: Env) -> u64;
    // Contract version
    fn version(e: Env) -> u32;
    // Contract admin address
    fn admin(e: Env) -> Option<Address>;
    // Extend asset TTL (time-to-live) in the contract storage
    fn extend_asset_ttl(e: Env, sponsor: Address, asset: Asset);
    // Get asset expiration timestamp
    fn expires(e: &Env, asset: Asset) -> Option<u64>;
    // Get retention FeeConfig configuration
    fn retention_config(e: &Env) -> FeeConfig;
}

// Quoted asset definition
#[contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Asset {
    Stellar(Address), // for Stellar Classic and Soroban assets
    Other(Symbol)     // for any external currencies/tokens/assets/symbols
}

// Price record definition
#[contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PriceData {
    pub price: i128,   // asset price at given point in time
    pub timestamp: u64 // record timestamp
}

// Possible runtime errors
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Error {
    AlreadyInitialized = 0,
    Unauthorized = 1,
    AssetMissing = 2,
    AssetAlreadyExists = 3,
    InvalidConfigVersion = 4,
    InvalidTimestamp = 5,
    InvalidUpdateLength = 6,
    AssetLimitExceeded = 7,
    InvalidPricesUpdate = 8
}
```

### **Pulse contract**

### Invocation from consumer contract

#### Utilize this example to invoke oracles from your contract code.

```rust
/* contract.rs */
use crate::reflector::{ReflectorClient, Asset as ReflectorAsset}; // Import Reflector interface
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};
#[contract]
pub struct MyAwesomeContract; // Of course, it's awesome, we know it!
#[contractimpl]
impl MyAwesomeContract {
    pub fn lets_rock(e: Env) {
        // Oracle contract address to use
        let oracle_address = Address::from_str(&e, "CAFJZQWSED6YAWZU3GWRTOCNPPCGBN32L7QV43XX5LZLFTK6JLN34DLN");
        // Create client for working with oracle
        let reflector_client = ReflectorClient::new(&e, &oracle_address);
        // Ticker to lookup the price
        let ticker = ReflectorAsset::Other(Symbol::new(&e, &("BTC")));
        // Fetch the most recent price record for it
        let recent = reflector_client.lastprice(&env.current_contract_address(), &ticker);
        // Check the result
        if recent.is_none() {
            //panic_with_error!(&e, "price not available");
        }
        // Retrieve the price itself
        let price = recent.unwrap().price;
        // Do not forget for price precision, get decimals from the oracle
        // (this value can be also hardcoded once the price feed has been
        // selected because decimals never change in live oracles)
        let price_decimals = reflector_client.decimals();
        
        // Let's check how much of quoted asset we can potentially purchase for $10
        let usd_balance = 10_0000000i128; // $10 with standard Stellar token precision
        let can_purchase = (usd_balance * 10i128.pow(price_decimals)) / price;
        
        // How many USD we'll need to buy 5 quoted asset tokens?
        let want_purchase = 5_0000000i128; // 5 tokens with standard Stellar token precision
        let need_usd = (want_purchase * price) / 10i128.pow(price_decimals);
        
        // Please note: check for potential overflows or use safe math when dealing with prices
    }
}
```

### Interface for Beam contract

#### Copy and save it in your smart contract project as "reflector_beam.rs" file. This is the oracle client..

```rust
/* reflector.rs */
use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

// Oracle contract interface exported as ReflectorClient
#[soroban_sdk::contractclient(name = "ReflectorClient")]
pub trait Contract {
    // Base oracle symbol the price is reported in
    fn base(e: Env) -> Asset;
    // All assets quoted by the contract
    fn assets(e: Env) -> Vec<Asset>;
    // Number of decimal places used to represent price for all assets quoted by the oracle
    fn decimals(e: Env) -> u32;
    // Quotes asset price in base asset at specific timestamp
    fn price(e: Env, caller: Address, asset: Asset, timestamp: u64) -> Option<PriceData>;
    // Quotes the most recent price for an asset
    fn lastprice(e: Env, caller: Address, asset: Asset) -> Option<PriceData>;
    // Quotes last N price records for the given asset
    fn prices(e: Env, caller: Address, asset: Asset, records: u32) -> Option<Vec<PriceData>>;
    // Quotes the most recent cross price record for the pair of assets
    fn x_last_price(e: Env, caller: Address, base_asset: Asset, quote_asset: Asset) -> Option<PriceData>;
    // Quotes the cross price for the pair of assets at specific timestamp
    fn x_price(e: Env, caller: Address, base_asset: Asset, quote_asset: Asset, timestamp: u64) -> Option<PriceData>;
    // Quotes last N cross price records of for the pair of assets
    fn x_prices(e: Env, caller: Address, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<Vec<PriceData>>;
    // Quotes the time-weighted average price for the given asset over N recent records
    fn twap(e: Env, caller: Address, asset: Asset, records: u32) -> Option<i128>;
    // Quotes the time-weighted average cross price for the given asset pair over N recent records
    fn x_twap(e: Env, caller: Address, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<i128>;
    // Price feed resolution (default tick period timeframe, in seconds - 5 minutes by default)
    fn resolution(e: Env) -> u32;
    // Historical records retention period, in seconds (24 hours by default)
    fn history_retention_period(e: Env) -> Option<u64>;
    // The most recent price update timestamp
    fn last_timestamp(e: Env) -> u64;
    // Contract version
    fn version(e: Env) -> u32;
    // Contract admin address
    fn admin(e: Env) -> Option<Address>;
    // Extend asset TTL (time-to-live) in the contract storage
    fn extend_asset_ttl(e: Env, sponsor: Address, asset: Asset);
    // Get asset expiration timestamp
    fn expires(e: &Env, asset: Asset) -> Option<u64>;
    // Get retention FeeConfig configuration
    fn retention_config(e: &Env) -> FeeConfig;
    // Get invocation FeeConfig configuration
    fn invocation_config(e: &Env) -> FeeConfig;
}

// Quoted asset definition
#[contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Asset {
    Stellar(Address), // for Stellar Classic and Soroban assets
    Other(Symbol)     // for any external currencies/tokens/assets/symbols
}

// Price record definition
#[contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PriceData {
    pub price: i128,   // asset price at given point in time
    pub timestamp: u64 // record timestamp
}

// Possible runtime errors
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Error {
    AlreadyInitialized = 0,
    Unauthorized = 1,
    AssetMissing = 2,
    AssetAlreadyExists = 3,
    InvalidConfigVersion = 4,
    InvalidTimestamp = 5,
    InvalidUpdateLength = 6,
    AssetLimitExceeded = 7,
    InvalidPricesUpdate = 8
}
```

## Testing Contracts

### Prerequisites

- Ensure you have Rust installed and set up ([official installation guide](https://www.rust-lang.org/tools/install))

### Running All Tests

1. Navigate to the root directory of the project:

    ```bash
    cd ./reflector-contract
    ```
2. Run the tests:

    ```bash
    cargo test
    ```

### Running Specific Contract Tests

1. Navigate to the directory of the contract:

    ```bash
    cd ./reflector-contract
    ```

2. Run the tests:

    ```bash
    cargo test --package pulse-contract
    ```

## Building Contracts

### Prerequisites

- Ensure you have Rust installed and set up ([official installation guide](https://www.rust-lang.org/tools/install))
- Install Stellar CLI ([CLI installation guide](https://developers.stellar.org/docs/tools/cli/install-cli))

### Building All Contracts

1. Navigate to the directory of the contract:

    ```bash
    cd ./reflector-contract
    ```
    
2. Run the build command:
    ```bash
    stellar contract build 
    ```

### Building Specific Contract

1. Navigate to the directory of the contract:

    ```bash
    cd ./reflector-contract
    ```
2. Run the build command for the specific contract:
    ```bash
    stellar contract build --package pulse-contract
    ```

### Optimizing WASM

1. Run stellar optimize command:
    ```bash
    stellar contract optimize --wasm ./target/wasm32v1-none/release/pulse_contract.wasm
    ```
This will generate an optimized WASM file at `./target/wasm32v1-none/release/pulse_contract.optimized.wasm`.

**Note**: Make sure to replace `pulse_contract.wasm` with the actual name of the contract you are optimizing. Also, replace the path if your build output directory is different.