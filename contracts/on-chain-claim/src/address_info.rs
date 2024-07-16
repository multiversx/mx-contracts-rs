use multiversx_sc::derive_imports::*;

#[type_abi]
#[derive(Default, NestedEncode, NestedDecode, TopEncode, TopDecode)]
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

    pub fn new_with_epoch(current_epoch: u64) -> Self {
        AddressInfo {
            current_streak: 1,
            last_epoch_claimed: current_epoch,
            total_epochs_claimed: 1,
            best_streak: 1,
        }
    }
}
