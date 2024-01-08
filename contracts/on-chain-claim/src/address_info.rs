multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct AddressInfo {
    pub current_streak: u64,
    pub last_epoch_claimed: u64,
    pub total_epochs_claimed: u64,
    pub best_streak: u64,
}

impl AddressInfo {
    #[inline]
    pub fn new(
        current_streak: u64,
        last_epoch_claimed: u64,
        total_epochs_claimed: u64,
        best_streak: u64,
    ) -> Self {
        AddressInfo {
            current_streak,
            last_epoch_claimed,
            total_epochs_claimed,
            best_streak,
        }
    }
}
