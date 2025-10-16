#[derive(PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
// Type of feed quoted by oracle contract
pub enum AssetType {
    Stellar = 1,
    Other = 2,
}
