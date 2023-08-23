use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use stellar_strkey;

const DECIMAL_KEY: &str = "DECIMALS";
const RESOLUTION_KEY: &str = "RESOLUTION";
const BASE_ASSET_TYPE: &str = "BASE_ASSET_TYPE";
const BASE_KEY: &str = "BASE";

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    if profile != "release" {
        return;
    }

    let decimals_str = env::var(DECIMAL_KEY).expect(&format!(
        "Please provide the {} environment variable with a valid number.",
        DECIMAL_KEY
    ));

    let resolution_str = env::var(RESOLUTION_KEY).expect(&format!(
        "Please provide the {} environment variable with a valid number.",
        RESOLUTION_KEY
    ));

    let base_asset_type_str = env::var(BASE_ASSET_TYPE).expect(&format!(
        "Please provide the {} environment variable with a valid value. Please specify 0 for Stellar assets and 1 for Generic assets.",
        BASE_ASSET_TYPE
    ));
    
    let base_str = env::var(BASE_KEY).expect(&format!(
        "Please provide the {} environment variable with a valid Stellar address or 32 bytes string for .",
        BASE_KEY
    ));

    let decimals = decimals_str
        .parse::<u32>()
        .expect("Invalid DECIMALS value.");

    let resolution: u32 = resolution_str
        .parse::<u32>()
        .expect("Invalid RESOLUTION value.");

    let base_asset_type = base_asset_type_str
        .parse::<u8>()
        .expect("Invalid BASE_ASSET_TYPE value. Please specify 0 for Stellar assets and 1 for Generic assets.");
    if base_asset_type != 0 && base_asset_type != 1 {
        panic!("Invalid BASE_ASSET_TYPE value. Please specify 0 for Stellar assets and 1 for Generic assets.");
    }

    let base_bytes = get_base_bytes(&base_str, &base_asset_type)
        .unwrap_or_else(|e| panic!("Invalid value for BASE: {}", e));

    let constants_path = Path::new("../shared/src/constants.rs");
    let backup_path = Path::new("../shared/src/constants.rs.bak");

    // Backup existing constants.rs
    fs::copy(&constants_path, &backup_path).expect("Failed to backup constants.rs");

    let mut constants_content: String = String::new();

    write_header(&mut constants_content);
    write_u32_to_constants(&mut constants_content, DECIMAL_KEY, decimals);
    write_u32_to_constants(&mut constants_content, RESOLUTION_KEY, resolution);
    write_asset_type_to_constants(&mut constants_content, &base_asset_type);
    write_array_to_constants(&mut constants_content, BASE_KEY, &base_bytes);
    write_footer(&mut constants_content);

    write_constants_to_file(&constants_path, &constants_content);
}

fn write_header(constants_content: &mut String) {
    writeln!(
        constants_content,
        "use crate::types::asset_type::AssetType; \n\npub struct Constants; \nimpl Constants {{\n"
    )
    .expect("Failed to write header to constants.rs");
}

fn write_footer(constants_content: &mut String) {
    writeln!(constants_content, "}}").expect("Failed to write footer to constants.rs");
}

fn write_u32_to_constants(constants_content: &mut String, constant_name: &str, value: u32) {
    writeln!(
        constants_content,
        "pub const {}: u32 = {};",
        constant_name, value
    )
    .expect(format!("Failed to write {} to constants.rs", constant_name).as_str());
}

fn write_asset_type_to_constants(constants_content: &mut String, asset_type: &u8) {
    let asset_type = if asset_type == &0 {
        "AssetType::S"
    } else {
        "AssetType::G"
    };
    writeln!(
        constants_content,
        "pub const BASE_ASSET_TYPE: AssetType = {};",
        asset_type
    )
    .expect(format!("Failed to write {} to constants.rs", BASE_ASSET_TYPE).as_str());
}

fn write_array_to_constants(constants_content: &mut String, constant_name: &str, array: &[u8; 32]) {
    writeln!(
        constants_content,
        "pub const {}: [u8; 32] = [",
        constant_name
    )
    .expect(format!("Failed to write {} to constants.rs", constant_name).as_str());
    for (i, byte) in array.iter().enumerate() {
        write!(constants_content, "{:?}", byte)
            .expect(format!("Failed to write {} to constants.rs", constant_name).as_str());
        if i < array.len() - 1 {
            write!(constants_content, ", ")
                .expect(format!("Failed to write {} to constants.rs", constant_name).as_str());
        }
    }
    writeln!(constants_content, "];")
        .expect(format!("Failed to write {} to constants.rs", constant_name).as_str());
}

fn write_constants_to_file(constants_path: &Path, constants_content: &String) {
    let mut file = fs::File::create(&constants_path).expect("Failed to create constants.rs");
    file.write_all(constants_content.as_bytes())
        .expect("Failed to write to constants.rs");
}

fn get_base_bytes(base: &str, asset_type: &u8) -> std::io::Result<[u8; 32]> {
    match asset_type {
        0 => {
            return string_public_key_to_bytes(&base);
        }
        1 => {
            let mut base_array: [u8; 32] = [0; 32];
            let length = base.len();
            if length > 32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Asset code too long",
                ));
            }
            base_array[..length].copy_from_slice(base.as_bytes());
            return Ok(base_array);
        }
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid asset type",
            ))
        }
    }
}

fn string_public_key_to_bytes(public_key_str: &str) -> std::io::Result<[u8; 32]> {
    let str_key = stellar_strkey::Strkey::from_str(public_key_str);
    if str_key.is_err() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            str_key.unwrap_err()
        ));
    }
    match str_key.unwrap() {
        stellar_strkey::Strkey::Contract(contract_instance) => {
            return Ok(contract_instance.0);
        }
        stellar_strkey::Strkey::PublicKeyEd25519(pub_key) => {
            return Ok(pub_key.0);
        }
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Stellar address type",
            ))
        }
    }
}
