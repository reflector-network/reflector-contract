use crate::types::asset_type::AssetType;


pub struct Constants;
impl Constants {
    pub const DECIMALS: u32 = 14;
    pub const RESOLUTION: u32 = 300000;
    pub const BASE_ASSET_TYPE: AssetType = AssetType::S;
    pub const BASE: [u8; 32] = [
        45, 170, 221, 166, 188, 166, 220, 219, 230, 110, 118, 121, 105, 231, 215, 118, 255, 166,
        238, 120, 134, 236, 9, 163, 52, 250, 49, 214, 31, 35, 232, 151,
    ];
}
