#![no_std]
#![allow(unused_attributes)]

pub use address_info::AddressInfo;
use config::{FIRST_SEASON_ID, FIRST_SEASON_START_EPOCH};

use multiversx_sc::imports::*;

pub mod address_info;
pub mod config;
pub mod events;

use crate::config::MAX_REPAIR_GAP;
use multiversx_sc_modules::only_admin;

#[multiversx_sc::contract]
pub trait OnChainClaimContract:
    config::ConfigModule + events::EventsModule + only_admin::OnlyAdminModule
{
    #[init]
    fn init(&self, repair_streak_token_id: TokenIdentifier, repair_streak_token_nonce: u64) {
        self.internal_set_repair_streak_payment(repair_streak_token_id, repair_streak_token_nonce);

        let caller = self.blockchain().get_caller();
        self.add_admin(caller);

        self.seasons().push(&FIRST_SEASON_START_EPOCH);
    }

    #[upgrade]
    fn upgrade(&self) {
        if self.seasons().is_empty() {
            self.seasons().push(&FIRST_SEASON_START_EPOCH);
        }
    }

    fn migrate_legacy_address_info(&self, address: &ManagedAddress) {
        if self.address_info(address).is_empty() {
            return;
        }

        let address_info = self.address_info(address).take();
        self.address_info_by_season(address, FIRST_SEASON_ID)
            .set(address_info);
    }

    #[endpoint(claim)]
    fn claim(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.blockchain().is_smart_contract(&caller),
            "Only user accounts can perform claim"
        );
        self.require_same_shard(&caller);

        self.migrate_legacy_address_info(&caller);

        let current_epoch = self.blockchain().get_block_epoch();
        let current_season_id = self.get_current_season();

        let address_info_by_season_mapper = self.address_info_by_season(&caller, current_season_id);
        if address_info_by_season_mapper.is_empty() {
            let address_info = AddressInfo::new_with_epoch(current_epoch);
            self.address_info_by_season(&caller, current_season_id)
                .set(&address_info);
            self.new_claim_event(&caller, &address_info);
            return;
        }

        address_info_by_season_mapper.update(|address_info| {
            self.increment_address_info(address_info, current_epoch);
        });
    }

    #[payable]
    #[endpoint(claimAndRepair)]
    fn claim_and_repair(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.blockchain().is_smart_contract(&caller),
            "Only user accounts can perform claim and repair"
        );
        self.require_same_shard(&caller);

        self.migrate_legacy_address_info(&caller);

        let payment = self.call_value().single_esdt();
        let repair_streak_payment = self.repair_streak_payment().get();
        require!(
            *payment == repair_streak_payment,
            "Bad payment token/amount"
        );

        let current_epoch = self.blockchain().get_block_epoch();
        let current_season_id = self.get_current_season();
        let address_info = self.get_address_info(&caller);

        require!(
            address_info.total_epochs_claimed > 0,
            "can't repair streak for address"
        );

        let address_info_by_season_mapper = self.address_info_by_season(&caller, current_season_id);
        address_info_by_season_mapper.update(|address_info| {
            self.repair_address_info_streak(&caller, address_info, current_epoch);
        });

        self.send().esdt_local_burn(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );
    }

    #[endpoint(addSeason)]
    fn add_season(&self, epoch: u64) {
        self.require_caller_is_admin();

        let mut seasons = self.seasons();

        if seasons.is_empty() {
            seasons.push(&epoch);
            return;
        }

        let current_epoch = self.blockchain().get_block_epoch();
        let last_season_starting_epoch = seasons.get(seasons.len());

        require!(
            last_season_starting_epoch < current_epoch,
            "last season must start before the current epoch"
        );

        require!(
            current_epoch < epoch,
            "new season must start after the last season"
        );
        seasons.push(&epoch);
    }

    #[endpoint(updateState)]
    fn update_state(
        &self,
        season_id: usize,
        address: &ManagedAddress,
        current_streak: u64,
        last_epoch_claimed: u64,
        total_epochs_claimed: u64,
        best_streak: u64,
    ) {
        self.require_caller_is_admin();
        self.require_same_shard(address);
        let current_season_id = self.get_current_season();
        require!(
            current_season_id == season_id,
            "season must be the current season"
        );

        self.migrate_legacy_address_info(address);

        let address_info = AddressInfo::new(
            current_streak,
            last_epoch_claimed,
            total_epochs_claimed,
            best_streak,
        );

        self.address_info_by_season(address, current_season_id)
            .set(&address_info);

        self.new_update_state_event(address, &address_info);
    }

    #[endpoint(setRepairStreakPayment)]
    fn set_repair_streak_payment(
        &self,
        repair_streak_token_identifier: TokenIdentifier,
        repair_streak_token_nonce: u64,
    ) {
        self.require_caller_is_admin();

        self.internal_set_repair_streak_payment(
            repair_streak_token_identifier,
            repair_streak_token_nonce,
        );
    }

    fn internal_set_repair_streak_payment(
        &self,
        repair_streak_token_identifier: TokenIdentifier,
        repair_streak_token_nonce: u64,
    ) {
        require!(
            repair_streak_token_identifier.is_valid_esdt_identifier(),
            "Invalid token ID",
        );

        let payment = EsdtTokenPayment::new(
            repair_streak_token_identifier,
            repair_streak_token_nonce,
            BigUint::from(1u64),
        );
        self.repair_streak_payment().set(payment);
    }

    fn increment_address_info(&self, address_info: &mut AddressInfo, current_epoch: u64) {
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

        if address_info.best_streak < address_info.current_streak {
            address_info.best_streak = address_info.current_streak;
        }
    }

    fn repair_address_info_streak(
        &self,
        caller: &ManagedAddress,
        address_info: &mut AddressInfo,
        current_epoch: u64,
    ) {
        let missed_epochs = self.get_missed_epochs(current_epoch, address_info.last_epoch_claimed);

        // Allow MAX_REPAIR_GAP + 1 in order to not have failed transaction when the user sends the claimAndRepair transaction
        // in the last round of the allowed epoch. From UI, we allow MAX_REPAIR_GAP = 5 (using canBeRepaired view)
        require!(
            missed_epochs > 0 && missed_epochs <= MAX_REPAIR_GAP + 1,
            "can't repair streak for current epoch"
        );

        address_info.current_streak += missed_epochs + 1;
        address_info.total_epochs_claimed += missed_epochs + 1;
        address_info.last_epoch_claimed = current_epoch;
        if address_info.best_streak < address_info.current_streak {
            address_info.best_streak = address_info.current_streak;
        }

        self.new_claim_and_repair_event(caller, address_info);
    }
}
