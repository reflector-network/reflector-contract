build-testnet:
	cargo build --target wasm32-unknown-unknown --profile release-with-logs
	stellar contract optimize --wasm target/wasm32-unknown-unknown/release-with-logs/etherfuse_yield_oracle.wasm

deploy-testnet: build-testnet
	stellar contract deploy --network testnet --source blend-test-net --wasm target/wasm32-unknown-unknown/release-with-logs/reflector_oracle.optimized.wasm --alias etherfuse-yield-oracle

update-testnet: build-testnet
	@HASH=$$(stellar contract upload --source blend-test-net --wasm target/wasm32-unknown-unknown/release-with-logs/etherfuse_yield_oracle.optimized.wasm | tail -n 1); \
	stellar contract invoke --id $$TESTNET_CONTRACT_ID --source blend-test-net --send=yes -- update_contract --wasm_hash $$HASH

test:
	cargo test