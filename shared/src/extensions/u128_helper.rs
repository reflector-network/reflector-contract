pub struct U128Helper(u128);

impl U128Helper {
    pub fn new(val_u64: u64, val_u8: u8) -> Self {
        U128Helper((val_u64 as u128) << 64 | val_u8 as u128)
    }

    pub fn encode_to_u128(val_u64: u64, val_u8: u8) -> u128 {
        (val_u64 as u128) << 64 | val_u8 as u128
    }
    
    pub fn decode(&self) -> (u64, u8) {
        let val_u64 = (self.0 >> 64) as u64;
        let val_u8 = (self.0 & 0xFF) as u8;
        (val_u64, val_u8)
    }
}