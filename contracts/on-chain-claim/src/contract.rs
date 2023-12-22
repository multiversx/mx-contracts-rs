#![no_std]
#![allow(unused_attributes)]

pub use address_info::AddressInfo;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod address_info;
pub mod config;

use crate::config::{MAX_REPAIR_GAP, SFT_AMOUNT};

#[multiversx_sc::contract]
pub trait OnChainClaimContract: config::ConfigModule {
    #[init]
    fn init(&self, repair_streak_token_id: TokenIdentifier) {
        self.repair_streak_token_identifier()
            .set(repair_streak_token_id);
    }

    #[endpoint(claim)]
    fn claim(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.blockchain().is_smart_contract(&caller),
            "Only user accounts can perform claim"
        );

        let current_epoch = self.blockchain().get_block_epoch();

        let address_info_mapper = self.address_info(&caller);
        if address_info_mapper.is_empty() {
            let address_info = AddressInfo::new(1, current_epoch, 1);
            self.address_info(&caller).set(address_info);
            return;
        }

        address_info_mapper.update(|address_info| {
            require!(
                address_info.last_epoch_claimed < current_epoch,
                "epoch already claimed"
            );

            if address_info.last_epoch_claimed + 1 == current_epoch {
                address_info.current_streak += 1;
            } else {
                address_info.current_streak = 1;
            }

            address_info.total_epochs_claimed += 1;
            address_info.last_epoch_claimed = current_epoch;
        });
    }

    #[payable("*")]
    #[endpoint(claimAndRepair)]
    fn claim_and_repair(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.blockchain().is_smart_contract(&caller),
            "Only user accounts can perform claim and repair"
        );
        let payment = self.call_value().single_esdt();
        let repair_streak_token_identifier = self.repair_streak_token_identifier().get();
        require!(
            payment.token_identifier == repair_streak_token_identifier,
            "Bad payment token"
        );
        require!(payment.amount == SFT_AMOUNT, "Bad payment amount");

        let current_epoch = self.blockchain().get_block_epoch();

        let address_info_mapper = self.address_info(&caller);

        require!(
            !address_info_mapper.is_empty(),
            "can't repair streak for address"
        );

        address_info_mapper.update(|address_info| {
            require!(
                address_info.last_epoch_claimed + MAX_REPAIR_GAP == current_epoch,
                "can't repair streak for current epoch"
            );

            address_info.current_streak += MAX_REPAIR_GAP;
            address_info.total_epochs_claimed += MAX_REPAIR_GAP;
            address_info.last_epoch_claimed = current_epoch;
        });

        self.send().esdt_local_burn(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );
    }

    #[only_owner]
    #[endpoint(updateState)]
    fn update_state(
        &self,
        address: &ManagedAddress,
        current_streak: u64,
        last_epoch_claimed: u64,
        total_epochs_claimed: u64,
    ) {
        let address_info =
            AddressInfo::new(current_streak, last_epoch_claimed, total_epochs_claimed);
        self.address_info(address).set(address_info);
    }
}
