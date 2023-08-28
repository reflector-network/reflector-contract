# Reflector oracle smart contract

## Building the Contracts

The contract optimizes on calls and storage by using constants for values that will not change throughout the contract's lifetime, as well as for the default administrator. To build the contract with these values, you need to execute a script with parameters. 

### Prerequisites

- Ensure you have Rust installed and set up on your local machine. [Follow the official guide here.](https://www.rust-lang.org/tools/install)

### Building the Price Oracle and Price Oracle Plus

1. Navigate to the directory of the contract you want to build:

    ```bash
    cd ./price-oracle
    ```

2. Run the `build-wasm.sh` script with the appropriate parameters:
    ```bash
    ./build-wasm.sh --decimals 14 --resolution 300000 --base_asset_type 0 --base CCG...TY6
    ```

### Parameters

- `--decimals`: Number of decimal places the asset uses.
- `--resolution`: The frequency of price updates, specified in milliseconds.
- `--base_asset_type`: The base asset type. 0 for Stellar assets and 1 for Generic assets.
- `--base`: The base asset's contract address or generic code.

The script will replace the values for constants, compile the wasm file, and restore the previous constant values.