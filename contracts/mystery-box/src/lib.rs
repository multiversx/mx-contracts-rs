#![no_std]

use multiversx_sc::imports::*;

pub mod config;
pub mod events;
pub mod mystery_box_proxy;
pub mod rewards;
pub mod token_attributes;

use crate::config::SFT_AMOUNT;
use config::{Reward, RewardType, MAX_PERCENTAGE};
use multiversx_sc_modules::only_admin;

#[multiversx_sc::contract]
pub trait MysteryBox:
    config::ConfigModule
    + rewards::RewardsModule
    + token_attributes::TokenAttributesModule
    + only_admin::OnlyAdminModule
    + events::EventsModule
{
    #[init]
    fn init(&self, mystery_box_token_id: TokenIdentifier) {
        require!(
            mystery_box_token_id.is_valid_esdt_identifier(),
            "Invalid token ID"
        );
        self.mystery_box_token_id()
            .set_if_empty(mystery_box_token_id);
        let caller = self.blockchain().get_caller();
        self.add_admin(caller);
    }

    #[endpoint(setupMysteryBox)]
    fn setup_mystery_box(
        &self,
        winning_rates_list: MultiValueEncoded<
            MultiValue6<RewardType, EgldOrEsdtTokenIdentifier, BigUint, ManagedBuffer, u64, u64>,
        >,
    ) {
        self.require_caller_is_admin();
        let mut accumulated_percentage = 0u64;
        let mut winning_rates = ManagedVec::new();
        for winning_rate in winning_rates_list.into_iter() {
            let (
                reward_type,
                reward_token_id,
                value,
                description,
                percentage_chance,
                epochs_cooldown,
            ) = winning_rate.into_tuple();
            accumulated_percentage += percentage_chance;
            let reward = Reward::new(
                reward_type,
                reward_token_id,
                value,
                description,
                percentage_chance,
                epochs_cooldown,
            );
            self.check_reward_validity(&reward);
            winning_rates.push(reward);
        }
        require!(
            accumulated_percentage == MAX_PERCENTAGE,
            "The total percentage must be 100%"
        );

        self.winning_rates().set(winning_rates);
        self.mystery_box_uris().set_if_empty(ManagedVec::new());
    }

    #[endpoint(updateMysteryBoxUris)]
    fn update_mystery_box_uris(&self, uris: MultiValueEncoded<ManagedBuffer>) {
        self.require_caller_is_admin();
        self.mystery_box_uris().set(uris.to_vec());
    }

    #[endpoint(createMysteryBox)]
    fn create_mystery_box(&self, amount: BigUint) -> EsdtTokenPayment<Self::Api> {
        self.require_caller_is_admin();
        let winning_rates_mapper = self.winning_rates();
        require!(
            !winning_rates_mapper.is_empty(),
            "The Mystery Box must be set up first"
        );

        let current_epoch = self.blockchain().get_block_epoch();
        let mystery_box_attributes = winning_rates_mapper.get();
        let output_payment = self.create_new_tokens(amount, &mystery_box_attributes);
        let caller = self.blockchain().get_caller();
        self.tx()
            .to(&caller)
            .payment(&output_payment)
            .transfer_if_not_empty();

        self.emit_create_mystery_box_event(
            &caller,
            current_epoch,
            &output_payment,
            &mystery_box_attributes,
        );

        output_payment
    }

    #[payable("*")]
    #[endpoint(openMysteryBox)]
    fn open_mystery_box(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.blockchain().is_smart_contract(&caller),
            "Only user accounts can open mystery boxes"
        );
        let payment = self.call_value().single_esdt();
        let mystery_box_token_id = self.mystery_box_token_id().get();
        require!(
            payment.token_identifier == mystery_box_token_id,
            "Bad payment token"
        );
        require!(payment.amount == SFT_AMOUNT, "Bad payment amount");
        let attributes: ManagedVec<Reward<Self::Api>> = self
            .blockchain()
            .get_token_attributes(&payment.token_identifier, payment.token_nonce);

        let current_epoch = self.blockchain().get_block_epoch();

        let mut active_cooldown = true;
        let mut winning_reward = Reward::default();
        while active_cooldown {
            winning_reward = self.get_winning_reward(&attributes);
            active_cooldown = self.check_global_cooldown(current_epoch, &winning_reward);
        }

        // We send the mystery box rewards directly to the user
        if winning_reward.reward_type == RewardType::MysteryBox {
            self.create_and_send_mystery_box(&caller);
        }

        self.send().esdt_local_burn(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );

        self.emit_open_mystery_box_event(&caller, current_epoch, &winning_reward);
    }

    fn create_and_send_mystery_box(&self, address: &ManagedAddress) {
        let new_attributes = self.winning_rates().get();
        let new_mystery_box_payment =
            self.create_new_tokens(BigUint::from(SFT_AMOUNT), &new_attributes);
        self.tx()
            .to(address)
            .payment(&new_mystery_box_payment)
            .transfer_if_not_empty();
    }
}
