#![no_std]
#![allow(unused_attributes)]

use address_info::AddressInfo;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

mod address_info;
mod storage;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait OnchainClaimContract:
    storage::StorageModule {
    #[init]
    fn init(&self) {}

    #[endpoint(claim)]
    fn claim(&self) {
        let caller = self.blockchain().get_caller();

        let address_info_mapper = self.address_info(&caller);
        if address_info_mapper.is_empty() {
            let address_info = AddressInfo {
                current_streak: 1,
                last_epoch_claimed: self.blockchain().get_block_epoch(),
                total_epochs_claimed: 1,
            };
            self.address_info(&caller).set(&address_info);
            return;
        }

        address_info_mapper.update(|address_info| {
            require!(
                address_info.last_epoch_claimed < self.blockchain().get_block_epoch(),
                "epoch already claimed"
            );

            if address_info.last_epoch_claimed + 1 == self.blockchain().get_block_epoch() {
                address_info.current_streak += 1;
            } else {
                address_info.current_streak = 1;
            }

            address_info.total_epochs_claimed += 1;
            address_info.last_epoch_claimed = self.blockchain().get_block_epoch();
        });
    }
}
