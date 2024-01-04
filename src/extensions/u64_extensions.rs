pub trait U64Extensions {
    fn get_normalized_timestamp(self, timeframe: u64) -> u64;
    fn is_valid_timestamp(self, timeframe: u64) -> bool;
}

impl U64Extensions for u64 {    
    fn get_normalized_timestamp(self, timeframe: u64) -> u64 {
        if (self == 0) || (timeframe == 0) {
            return 0;
        }
        (self / timeframe) * timeframe
    }
    
    fn is_valid_timestamp(self, timeframe: u64) -> bool {
        self == Self::get_normalized_timestamp(self, timeframe)
    }
}