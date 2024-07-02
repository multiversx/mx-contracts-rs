use multiversx_sc::imports::*;

use crate::address_info::*;

pub const MAX_REPAIR_GAP: u64 = 5;

#[multiversx_sc::module]
pub trait ConfigModule {
    fn require_same_shard(&self, address: &ManagedAddress) {
        let address_shard = self.blockchain().get_shard_of_address(address);
        let sc_address = self.blockchain().get_sc_address();
        let sc_shard = self.blockchain().get_shard_of_address(&sc_address);

        require!(address_shard == sc_shard, "wrong shard");
    }

    fn get_missed_epochs(&self, current_epoch: u64, last_epoch_claimed: u64) -> u64 {
        if current_epoch <= last_epoch_claimed {
            return 0;
        }

        current_epoch - last_epoch_claimed - 1
    }

    #[view(getAddressInfo)]
    fn get_address_info(&self, address: &ManagedAddress) -> AddressInfo {
        let address_info_mapper = self.address_info(address);

        if address_info_mapper.is_empty() {
            return AddressInfo::default();
        }

        address_info_mapper.get()
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

    #[storage_mapper("address_info")]
    fn address_info(&self, address: &ManagedAddress) -> SingleValueMapper<AddressInfo>;

    #[view(getRepairStreakPayment)]
    #[storage_mapper("repair_streak_payment")]
    fn repair_streak_payment(&self) -> SingleValueMapper<EsdtTokenPayment>;
}
