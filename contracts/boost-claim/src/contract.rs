#![no_std]
#![allow(unused_attributes)]

use address_boost_info::AddressBoostInfo;
use multiversx_sc::imports::*;
use multiversx_sc_modules::only_admin;

pub mod address_boost_info;
pub mod config;

#[multiversx_sc::contract]
pub trait BoostClaimContract: config::ConfigModule + only_admin::OnlyAdminModule {
    #[init]
    fn init(
        &self,
        difference_between_claims: u64,
        levels_prizes: MultiValueEncoded<ManagedBuffer>,
    ) {
        self.set_difference_between_claims(difference_between_claims);
        self.set_levels_prizes(levels_prizes);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_admin]
    #[endpoint(setDifferenceBetweenClaims)]
    fn set_difference_between_claims(&self, difference_between_claims: u64) {
        self.time_difference_in_seconds()
            .set(difference_between_claims);
    }

    #[only_admin]
    #[endpoint(setLevelsPrizes)]
    fn set_levels_prizes(&self, levels_prizes: MultiValueEncoded<ManagedBuffer>) {
        self.levels_prizes().clear();
        for level_prize in levels_prizes.into_iter() {
            self.levels_prizes().push(&level_prize);
        }
    }

    #[endpoint(claim)]
    fn claim(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.blockchain().is_smart_contract(&caller),
            "Only user accounts can perform claim"
        );
        self.require_same_shard(&caller);

        let current_timestamp = self.blockchain().get_block_timestamp();

        let address_boost_info_mapper = self.address_boost_info(&caller);
        if address_boost_info_mapper.is_empty() {
            let address_boost_info = AddressBoostInfo::new(1, current_timestamp, 0);
            self.address_boost_info(&caller).set(&address_boost_info);
            self.boost_claim_event(&caller, &self.levels_prizes().get(1));
            return;
        }

        let timestamp = self.time_difference_in_seconds().get();

        address_boost_info_mapper.update(|address_boost_info| {
            require!(
                address_boost_info.last_claim_timestamp + timestamp < current_timestamp,
                "User can't claim yet"
            );

            let current_level = address_boost_info.current_level;

            let mut next_level = current_level + 1;
            if next_level > self.levels_prizes().len() {
                next_level = 1;
            }

            address_boost_info.last_claim_timestamp = current_timestamp;
            address_boost_info.current_level = next_level;

            if next_level == self.levels_prizes().len() {
                address_boost_info.total_cycles_completed += 1;
            }

            self.boost_claim_event(&caller, &self.levels_prizes().get(next_level));
        });
    }
}
