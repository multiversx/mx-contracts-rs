use multiversx_sc::derive_imports::*;

#[type_abi]
#[derive(Default, NestedEncode, NestedDecode, TopEncode, TopDecode, ManagedVecItem)]
pub struct SeasonInfo {
    pub id: u16,
    pub start_epoch: u64,
}

impl SeasonInfo {
    pub fn new(id: u16, start_epoch: u64) -> Self {
        SeasonInfo { id, start_epoch }
    }
}
