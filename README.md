# Reflector oracle smart contract

## Building the Contracts

The contract optimizes on calls and storage by using constants for values that will not change throughout the contract's lifetime, as well as for the default administrator. To build the contract with these values, you need to execute a script with parameters. 

### Prerequisites

- Ensure you have Rust installed and set up on your local machine. [Follow the official guide here.](https://www.rust-lang.org/tools/install)

### Building the Price Oracle and Price Oracle Plus

1. Navigate to the directory of the contract you want to build:

    ```bash
    cd ./price-oracle
    # OR
    cd ./price-oracle-plus
    ```

2. Run the `build-wasm.sh` script with the appropriate parameters:

    For `price-oracle`:

    ```bash
    ./build-wasm.sh --decimals 14 --resolution 300000 --admin GDK...Y3N --base_asset_type 0 --base CCG...TY6
    ```

    For `price-oracle-plus`:

    ```bash
    ./build-wasm.sh --decimals 14 --resolution 300000 --admin GDK...Y3N --base_asset_type 1 --base USD --fee_asset CCG...TY6
    ```

### Parameters

- `--decimals`: Number of decimal places the asset uses.
- `--resolution`: The frequency of price updates, specified in milliseconds.
- `--admin`: The default administrator's public key.
- `--base_asset_type`: The base asset type. 0 for Stellar assets and 1 for Generic assets.
- `--base`: The base asset's contract address or generic code.
- `--fee_asset`: The fee asset's contract address (only for Price Oracle Plus).

The script will replace the values for constants, compile the wasm file, and restore the previous constant values.