multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::address_info::*;

pub const SFT_AMOUNT: u64 = 1;
pub const MAX_REPAIR_GAP: u64 = 2;

#[multiversx_sc::module]
pub trait ConfigModule {
    #[view(canBeRepaired)]
    fn can_be_repaired(&self, address: &ManagedAddress) -> bool {
        let address_info_mapper = self.address_info(address);
        if address_info_mapper.is_empty() {
            return false;
        }

        let address_info = address_info_mapper.get();

        address_info.last_epoch_claimed + MAX_REPAIR_GAP == self.blockchain().get_block_epoch()
    }

    #[view(getAddressInfo)]
    #[storage_mapper("address_info")]
    fn address_info(&self, address: &ManagedAddress) -> SingleValueMapper<AddressInfo>;

    #[view(getRepairStreakTokenIdentifier)]
    #[storage_mapper("repair_streak_token_identifier")]
    fn repair_streak_token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;
}
