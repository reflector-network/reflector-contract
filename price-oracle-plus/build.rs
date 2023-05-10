use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use stellar_strkey;

const DECIMAL_KEY: &str = "DECIMALS";
const RESOLUTION_KEY: &str = "RESOLUTION";
const ADMIN_KEY: &str = "ADMIN";
const BASE_KEY: &str = "BASE";
const FEE_ASSET_KEY: &str = "FEE_ASSET";

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

    let admin_str = env::var(ADMIN_KEY).expect(&format!(
        "Please provide the {} environment variable with a valid Stellar address.",
        ADMIN_KEY
    ));
    let base_str = env::var(BASE_KEY).expect(&format!(
        "Please provide the {} environment variable with a valid Stellar address.",
        BASE_KEY
    ));
    let fee_asset_str = env::var(FEE_ASSET_KEY).expect(&format!(
        "Please provide the {} environment variable with a valid Stellar address.",
        FEE_ASSET_KEY
    ));

    let decimals = decimals_str
        .parse::<u32>()
        .expect("Invalid DECIMALS value.");

    let resolution: u32 = resolution_str
        .parse::<u32>()
        .expect("Invalid RESOLUTION value.");

    let admin_bytes =
        string_public_key_to_bytes(&admin_str).expect("Invalid Stellar address for ADMIN");
    let base_bytes =
        string_public_key_to_bytes(&base_str).expect("Invalid Stellar address for BASE");
    let fee_asset_bytes =
        string_public_key_to_bytes(&fee_asset_str).expect("Invalid Stellar address for FEE_ASSET");

    let constants_path = Path::new("../shared/src/constants.rs");
    let backup_path = Path::new("../shared/src/constants.rs.bak");

    // Backup existing constants.rs
    fs::copy(&constants_path, &backup_path).expect("Failed to backup constants.rs");

    let mut constants_content: String = String::new();

    write_header(&mut constants_content);
    write_u32_to_constants(&mut constants_content, DECIMAL_KEY, decimals);
    write_u32_to_constants(&mut constants_content, RESOLUTION_KEY, resolution);
    write_array_to_constants(&mut constants_content, ADMIN_KEY, &admin_bytes);
    write_array_to_constants(&mut constants_content, BASE_KEY, &base_bytes);
    write_array_to_constants(&mut constants_content, FEE_ASSET_KEY, &fee_asset_bytes);
    write_footer(&mut constants_content);

    write_constants_to_file(&constants_path, &constants_content);
}

fn write_header( constants_content: &mut String) {
    writeln!(
        constants_content,
        "pub struct Constants; \nimpl Constants {{\n"
    )
    .expect("Failed to write header to constants.rs");
}

fn write_footer(constants_content: &mut String) {
    writeln!(constants_content, "}}").expect("Failed to write footer to constants.rs");
}

fn write_u32_to_constants(
    constants_content: &mut String,
    constant_name: &str,
    value: u32,
)  {
    writeln!(
        constants_content,
        "pub const {}: u32 = {};",
        constant_name, value
    )
    .expect(format!("Failed to write {} to constants.rs", constant_name).as_str());
}

fn write_array_to_constants(
    constants_content: &mut String,
    constant_name: &str,
    array: &[u8; 32],
) {
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
    let mut file = fs::File::create(&constants_path)
        .expect("Failed to create constants.rs");
    file.write_all(constants_content.as_bytes())
        .expect("Failed to write to constants.rs");
}

fn string_public_key_to_bytes(public_key_str: &str) -> std::io::Result<[u8; 32]> {
    let str_key = stellar_strkey::Strkey::from_str(public_key_str)
        .expect("Failed to parse Stellar address");
    match str_key {
        stellar_strkey::Strkey::Contract(contract_instance) => {
            return Ok(contract_instance.0);
        }
        stellar_strkey::Strkey::PublicKeyEd25519(pub_key) => {
            return Ok(pub_key.0);
        }
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid Stellar address"))
    }
}
