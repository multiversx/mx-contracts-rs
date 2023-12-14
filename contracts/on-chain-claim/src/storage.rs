multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::address_info::*;

#[multiversx_sc::module]
pub trait StorageModule {
    #[view(getAddressInfo)]
    #[storage_mapper("address_info")]
    fn address_info(&self, address: &ManagedAddress) -> SingleValueMapper<AddressInfo>;
}
