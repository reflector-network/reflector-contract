# Etherfuse Asset Yield Oracle

This contract implementation is fully compatible with 
[SEP-40](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0040.md) ecosystem standard.
Check the standard for general info and public consumer interface documentation.


## Building the Contracts

### Prerequisites

- Ensure you have Rust installed and set up on your local machine. [Follow the official guide here.](https://www.rust-lang.org/tools/install)
- Install the stellar cli client. (Version used for this was 23.1.4)
- Get the `Yield Oracle Devnet Keypair` from the `Blend Pool` wallet in 1Password, and add a stellar key named `blend-test-net` or `blend-main-net` depending on which environment you are interacting with.

### Deploying

- For testnet, run `make deploy-testnet`. This will make a NEW contract. If you want to modify the existing testnet contract, run `make update-testnet`

### Initial Configurations

- Testnet Asset Contracts (Order matters!)

1. USTRY (CCUGMX7HX6CH6T4XNPALQ7HCVU37RLUNHAYNSAIALTPFHFQVIRIHALBC)
2. CETES (CC72F57YTPX76HAA64JQOEGHQAPSADQWSY5DWVBR66JINPFDLNCQYHIC)

- Testnet was configured like so

```sh
stellar contract invoke --id $TESTNET_CONTRACT_ID -- config --config '{"admin": "GBH62ESUWAJGVIDWMQTIJ4T24IWIGMYM2LGGVCBZGAZY7EYHDYMMA7HX", "base_asset": {"Other": "USD"}, "decimals": 14, "period": 86400000, "resolution": 300000}'
```

- Afterwards, assets were added with matching FX symbols like so:

```sh
stellar contract invoke --id $TESTNET_CONTRACT_ID -- add_assets --fxs '["USD","MXN"]' --assets '[{"Stellar":"CCUGMX7HX6CH6T4XNPALQ7HCVU37RLUNHAYNSAIALTPFHFQVIRIHALBC"},{"Stellar":"CC72F57YTPX76HAA64JQOEGHQAPSADQWSY5DWVBR66JINPFDLNCQYHIC"}]'
```

### Testing methods

Use the stellar CLI to execute methods on the contract.  Set your network to `testnet` using `stellar network use testnet`.

`stellar contract invoke --id $TESTNET_CONTRACT_ID -- <instruction_name>`

You can use the `--help` param to see how to pass the variables for any given instruction.

`stellar contract invoke --id $TESTNET_CONTRACT_ID -- <instruction_name> --help`

You usually need to wrap JSON in single quotes, and be sure to wrap property names in double quotes
