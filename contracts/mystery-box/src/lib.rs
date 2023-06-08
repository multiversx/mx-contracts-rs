#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod config;
pub mod events;
pub mod rewards;
pub mod token_attributes;

use crate::config::{MysteryBoxAttributes, SFT_AMOUNT};
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
    fn init(&self, mystery_box_token_id: TokenIdentifier, mystery_box_epochs_cooldown_period: u64) {
        require!(
            mystery_box_token_id.is_valid_esdt_identifier(),
            "Invalid token ID"
        );
        self.mystery_box_token().set_token_id(mystery_box_token_id);
        self.mystery_box_cooldown_period()
            .set(mystery_box_epochs_cooldown_period);

        let caller = self.blockchain().get_caller();
        self.add_admin(caller);
    }

    #[endpoint(setupMysteryBox)]
    fn setup_mystery_box(&self, winning_rates: ManagedVec<Reward<Self::Api>>) {
        self.require_caller_is_admin();
        let mut accumulated_percentage = 0u64;
        for winning_rate in winning_rates.into_iter() {
            accumulated_percentage += winning_rate.percentage_chance;
            self.check_reward_validity(&winning_rate);
        }
        require!(
            accumulated_percentage == MAX_PERCENTAGE,
            "The total percentage must be 100%"
        );

        self.winning_rates().set(winning_rates);
        self.mystery_box_uris().set_if_empty(ManagedVec::new());
    }

    #[endpoint(updateMysteryBoxUris)]
    fn update_mystery_box_uris(&self, uris: ManagedVec<ManagedBuffer>) {
        self.require_caller_is_admin();
        self.mystery_box_uris().set(uris);
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
        let mystery_box_attributes =
            MysteryBoxAttributes::new(winning_rates_mapper.get(), current_epoch);
        let output_payment = self.create_new_tokens(amount, &mystery_box_attributes);
        let caller = self.blockchain().get_caller();
        self.send()
            .direct_non_zero_esdt_payment(&caller, &output_payment);

        self.emit_create_mystery_box_event(
            &caller,
            current_epoch,
            &output_payment,
            &mystery_box_attributes.rewards,
        );

        output_payment
    }

    #[payable("*")]
    #[endpoint(openMysteryBox)]
    fn open_mystery_box(&self) {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        let mystery_box_token_mapper = self.mystery_box_token();
        let mystery_box_token_id = mystery_box_token_mapper.get_token_id();
        require!(
            payment.token_identifier == mystery_box_token_id,
            "Bad payment token"
        );
        require!(payment.amount == SFT_AMOUNT, "Bad payment amount");
        let attributes: MysteryBoxAttributes<Self::Api> = self
            .mystery_box_token()
            .get_token_attributes(payment.token_nonce);

        let current_epoch = self.blockchain().get_block_epoch();
        let mystery_box_cooldown_period = self.mystery_box_cooldown_period().get();
        require!(
            attributes.create_epoch + mystery_box_cooldown_period <= current_epoch,
            "Mystery box cannot be opened yet"
        );

        let mut active_cooldown = true;
        let mut winning_reward = Reward::default();
        while active_cooldown {
            winning_reward = self.get_winning_reward(&attributes.rewards);
            active_cooldown = self.check_global_cooldown(current_epoch, &winning_reward);
        }

        // We send the mystery box rewards directly to the user
        if winning_reward.reward_type == RewardType::MysteryBox {
            let new_attributes =
                MysteryBoxAttributes::new(self.winning_rates().get(), current_epoch);
            let new_mystery_box_payment =
                self.create_new_tokens(BigUint::from(SFT_AMOUNT), &new_attributes);
            self.send()
                .direct_non_zero_esdt_payment(&caller, &new_mystery_box_payment);
        }

        mystery_box_token_mapper.nft_burn(payment.token_nonce, &payment.amount);

        self.emit_open_mystery_box_event(&caller, current_epoch, &winning_reward);
    }
}
