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

    pub fn default() -> Self {
        AddressInfo {
            current_streak: 0,
            last_epoch_claimed: 0,
            total_epochs_claimed: 0,
            best_streak: 0,
        }
    }

    pub fn new_with_epoch(current_epoch: u64) -> Self {
        AddressInfo {
            current_streak: 1,
            last_epoch_claimed: current_epoch,
            total_epochs_claimed: 1,
            best_streak: 1,
        }
    }
}
