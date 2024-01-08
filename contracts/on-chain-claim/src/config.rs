multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::address_info::*;

pub const SFT_AMOUNT: u64 = 1;
pub const MAX_REPAIR_GAP: u64 = 5;

#[multiversx_sc::module]
pub trait ConfigModule {
    fn get_missed_epochs(&self, current_epoch: u64, last_epoch_claimed: u64) -> u64 {
        if current_epoch - last_epoch_claimed <= 1 {
            return 0;
        }

        current_epoch - last_epoch_claimed - 1
    }

    #[view(canBeRepaired)]
    fn can_be_repaired(&self, address: &ManagedAddress) -> bool {
        let address_info_mapper = self.address_info(address);
        if address_info_mapper.is_empty() {
            return false;
        }

        let address_info = address_info_mapper.get();
        let current_epoch = self.blockchain().get_block_epoch();
        let missed_epochs = self.get_missed_epochs(current_epoch, address_info.last_epoch_claimed);

        missed_epochs > 0 && missed_epochs <= MAX_REPAIR_GAP
    }

    #[view(getAddressInfo)]
    #[storage_mapper("address_info")]
    fn address_info(&self, address: &ManagedAddress) -> SingleValueMapper<AddressInfo>;

    #[view(getRepairStreakTokenIdentifier)]
    #[storage_mapper("repair_streak_token_identifier")]
    fn repair_streak_token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;
}
