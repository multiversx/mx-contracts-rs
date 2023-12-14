multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct AddressInfo {
    pub current_streak: u64,
    pub last_epoch_claimed: u64,
    pub total_epochs_claimed: u64,
}
