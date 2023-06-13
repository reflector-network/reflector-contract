#!/bin/bash

ADMIN=""
BASE=""
DECIMALS=""
RESOLUTION=""

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --admin)
            ADMIN="$2"
            shift 2
            ;;
        --base_asset_type)
            BASE_ASSET_TYPE="$2"
            shift 2
            ;;
        --base)
            BASE="$2"
            shift 2
            ;;
        --decimals)
            DECIMALS="$2"
            shift 2
            ;;
        --resolution)
            RESOLUTION="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

if [ -z "$ADMIN" ]; then
    echo "--admin argument is required."
    exit 1
fi

if [ -z "$BASE_ASSET_TYPE" ]; then
    echo "--base_asset_type argument is required."
    exit 1
fi

if [ -z "$BASE" ]; then
    echo "--base argument is required."
    exit 1
fi

if [ -z "$DECIMALS" ]; then
    echo "--decimals argument is required."
    exit 1
fi

if [ -z "$RESOLUTION" ]; then
    echo "--resolution argument is required."
    exit 1
fi


# touch lib.rs and constants.rs to force a rebuild
touch ./src/lib.rs
touch ../shared/src/constants.rs
# build the contract with the provided arguments
DECIMALS="$DECIMALS" RESOLUTION="$RESOLUTION" ADMIN="$ADMIN" BASE_ASSET_TYPE="$BASE_ASSET_TYPE" BASE="$BASE" cargo build --release --target wasm32-unknown-unknown

# restore constants.rs
# check if the backup file exists
if [ -f "../shared/src/constants.rs.bak" ]; then
    # restore the original constants.rs file
    mv "../shared/src/constants.rs.bak" "../shared/src/constants.rs"
    echo "Restored constants.rs from constants.rs.bak"
else
    echo "Backup file constants.rs.bak not found. No restoration performed."
fi