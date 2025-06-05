#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod config;
pub mod events;
pub mod rewards;
pub mod token_attributes;

use crate::config::{RewardCooldown, SFT_AMOUNT};
use config::{CooldownType, Reward, RewardType, MAX_PERCENTAGE};
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

    #[upgrade]
    fn upgrade(&self) {}

    #[endpoint(setupMysteryBox)]
    fn setup_mystery_box(
        &self,
        winning_rates_list: MultiValueEncoded<
            MultiValue8<
                RewardType,
                EgldOrEsdtTokenIdentifier,
                BigUint,
                ManagedBuffer,
                u64,
                CooldownType,
                u64,
                u64,
            >,
        >,
    ) {
        self.require_caller_is_admin();
        let current_epoch = self.blockchain().get_block_epoch();
        let mut reward_id = self.last_reward_id().get();
        let mut accumulated_percentage = 0u64;
        let mut winning_rates = ManagedVec::new();
        for winning_rate in winning_rates_list.into_iter() {
            let (
                reward_type,
                reward_token_id,
                value,
                description,
                percentage_chance,
                cooldown_type,
                wins_per_cooldown,
                cooldown_epochs,
            ) = winning_rate.into_tuple();
            accumulated_percentage += percentage_chance;
            reward_id += 1;
            let reward = Reward::new(
                reward_id,
                reward_type,
                reward_token_id,
                value,
                description,
                percentage_chance,
            );
            let reward_cooldown = RewardCooldown::new(
                cooldown_type,
                wins_per_cooldown,
                cooldown_epochs,
                wins_per_cooldown,
                current_epoch,
            );
            self.check_reward_validity(&reward, &reward_cooldown);
            winning_rates.push(reward);

            self.reward_cooldown(reward_id).set(reward_cooldown);
        }
        require!(
            accumulated_percentage == MAX_PERCENTAGE,
            "The total percentage must be 100%"
        );

        self.last_reward_id().set(reward_id);
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
        self.send()
            .direct_non_zero_esdt_payment(&caller, &output_payment);

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
            active_cooldown = self.check_reward_cooldown(current_epoch, &winning_reward);
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
        self.send()
            .direct_non_zero_esdt_payment(address, &new_mystery_box_payment);
    }
}
