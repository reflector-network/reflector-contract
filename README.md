# Reflector oracle smart contract

This contract implementation is fully compatible with 
[SEP-40](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0040.md) ecosystem standard.
Check the standard for general info and public consumer interface documentation.

## Usage example

### Forced position liquidation

```rust
pub fn check_liquidation(env: Env, reflector_contract_id: Address, loan: Loan, liquidation_threshold: i128) {
    // loan position example
    // {
    //    collateral_asset: Asset::Other(Symbol::new(&env, "BTC")),
    //    collateral_amount: 10753533963_i128,
    //    borrowed_asset: Asset::Other(Symbol::new(&env, "ETH")),
    //    borrowed_amount: 154850889072_i128
    // }

    // create the price oracle client instance
    let reflector_contract = PriceOracleClient::new(&env, &reflector_contract_id);

    // get oracle prcie precision
    let decimals = reflector_contract.decimals();

    // get the price and calculate the value of the collateral
    let collateral_asset_price = reflector_contract.lastprice(&loan.collateral_asset).unwrap();
    let collateral_value = collateral_asset_.price * loan.collateral_amount;

    // get the price and calculate the value of the borrowed asset
    let asset_price = reflector_contract.lastprice(&loan.borrowed_asset).unwrap();
    let borrowed_value = asset_price.price * loan.borrowed_amount;

    // calculate the current loan to value ratio, SAC contracts 
    let collateralization_ratio = collateral_value * 10000000_i128 / borrowed_value;

    if collateralization_ratio <= liquidation_threshold {
        // collateralization ratio is too small – liquidate the loan
    }
}
```

### Portfolio rebalancing

```rust
pub fn rebalance_portfolio(env: Env, reflector_contract_id: Address, portfolio: Vec<PortfolioPosition>) {
    // portfolio example
    // [{
    //    asset: Asset::Stellar(Address::from_str(&env, "CD8H6KNN9...")),
    //    amount: 45675353821010_i128,
    // },
    // {
    //    asset: Asset::Stellar(Symbol::new(&env, "BTC")),
    //    amount: 10753533963_i128,
    // }]

    // create the price oracle client instance
    let reflector_contract = PriceOracleClient::new(&env, &reflector_contract_id);

    // storage for portfolio position values
    let mut values: [i128; 3] = [0; 3];
    // calculate total value of the portfolio
    let mut total_value = 0_i128;
    let total_positions = portfolio.len()
    for i in 0..total_positions {
        let position = &portfolio[i];
        // get price of an asset
        let asset_price = reflector_contract.lastprice(&position.asset).unwrap();
        // calculate position USD value
        let asset_value = asset_price.price * position.amount;
        total_value += asset_value;
        values[i] = asset_value;
    }

    // calculate average value per position
    let average_position_value = total_value / (total_positions as i128);

    // calculate the difference between the target value and the actual value for each position
    for i in 0..total_positions {
        let value: i128 = values[i];
        if value > average_position_value {
            // sell some tokens to decrease share in the portfolio
        } else if value < average_position_value {
            // buy tokens to increase position size
        }
    }
}
```

### Algorithmic stablecoin price correction

```rust
pub fn maintain_stable_coin_peg(env: Env, reflector_contract_id: Address, current_price: i128) {
    // create oracle client instance
    let reflector_contract = PriceOracleClient::new(&env, &reflector_contract_id);

    // fetch TWAP-approximated external price for the associated reference ticker
    let coin = Asset::Other(Symbol::new(&env, "CHF"));
    let reference_price = reflector_contract.twap(&coin, &5).unwrap();

    // take action if the price diverts more than 0.1% from the reference price
    let threshold = reference_price / 1000_i128;
    if current_price > reference_price + threshold {
        // mint and sell coin
    }
    if current_price < reference_price - threshold {
        // buy and burn coin
    }
}
```


## Building the Contracts

### Prerequisites

- Ensure you have Rust installed and set up on your local machine. [Follow the official guide here.](https://www.rust-lang.org/tools/install)

### Building the Price Oracle

1. Navigate to the directory of the contract:

    ```bash
    cd ./price-oracle
    ```

2. Run the build command:
    ```bash
    cargo build --release --target wasm32-unknown-unknown
    ```