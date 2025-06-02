use multiversx_sc::derive_imports::*;

#[type_abi]
#[derive(Default, NestedEncode, NestedDecode, TopEncode, TopDecode)]
pub struct AddressBoostInfo {
    pub current_level: usize,
    pub last_claim_timestamp: u64,
    pub total_cycles_completed: u64,
}

impl AddressBoostInfo {
    pub fn new(
        current_level: usize,
        last_claim_timestamp: u64,
        total_cycles_completed: u64,
    ) -> Self {
        AddressBoostInfo {
            current_level,
            last_claim_timestamp,
            total_cycles_completed,
        }
    }
}
