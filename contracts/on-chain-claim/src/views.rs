multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::storage::{self};

#[multiversx_sc::module]
pub trait ViewsModule:
    storage::StorageModule {
    #[view(canBeRepaired)]
    fn can_be_repaired(&self, address: &ManagedAddress) -> bool {
        let address_info_mapper = self.address_info(address);
        if address_info_mapper.is_empty() {
            return false;
        }

        let address_info = address_info_mapper.get();

        address_info.last_epoch_claimed + 2 == self.blockchain().get_block_epoch()
    }
}
