use crate::types::asset_type::AssetType;


pub struct Constants;
impl Constants {
    pub const DECIMALS: u32 = 14;
    pub const RESOLUTION: u32 = 300000;
    pub const ADMIN: [u8; 32] = [
        79, 57, 19, 0, 161, 16, 245, 231, 123, 19, 220, 109, 195, 237, 164, 218, 8, 24, 199, 38,
        210, 201, 112, 30, 234, 130, 195, 77, 199, 201, 150, 190,
    ];
    pub const BASE_ASSET_TYPE: AssetType = AssetType::STELLAR;
    pub const BASE: [u8; 32] = [
        45, 170, 221, 166, 188, 166, 220, 219, 230, 110, 118, 121, 105, 231, 215, 118, 255, 166,
        238, 120, 134, 236, 9, 163, 52, 250, 49, 214, 31, 35, 232, 151,
    ];
    pub const FEE_ASSET: [u8; 32] = [
        186, 35, 9, 248, 65, 227, 202, 100, 15, 160, 139, 2, 81, 47, 194, 158, 242, 104, 118, 126,
        235, 140, 170, 162, 148, 122, 173, 163, 254, 37, 25, 71,
    ];
}
