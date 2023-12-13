#[derive(PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum AssetType {
    Stellar = 1,
    Other = 2
}