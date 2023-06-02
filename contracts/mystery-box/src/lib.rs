#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod config;
pub mod events;
pub mod permissions_module;
pub mod rewards;

use config::{Reward, RewardType, MAX_PERCENTAGE, NFT_AMOUNT};

#[multiversx_sc::contract]
pub trait MysteryBox:
    config::ConfigModule
    + rewards::RewardsModule
    + permissions_module::PermissionsModule
    + events::EventsModule
{
    #[init]
    fn init(&self, mystery_box_token_id: TokenIdentifier) {
        require!(
            mystery_box_token_id.is_valid_esdt_identifier(),
            "Invalid token ID"
        );
        self.mystery_box_token_identifier()
            .set(mystery_box_token_id);

        let caller = self.blockchain().get_caller();
        self.add_admin(&caller);
    }

    #[endpoint(createMysteryBox)]
    fn create_mystery_box(
        &self,
        receiver: ManagedAddress,
        mystery_box_setup: ManagedVec<Reward<Self::Api>>,
    ) -> EsdtTokenPayment<Self::Api> {
        let caller = self.blockchain().get_caller();
        self.require_admin(&caller);

        let mystery_box_token_id = self.mystery_box_token_identifier().get();

        let mut accumulated_percentage = 0u64;

        for mystery_box_reward in mystery_box_setup.into_iter() {
            accumulated_percentage += mystery_box_reward.percentage_chance;
            self.check_reward_validity(&mystery_box_reward);
        }
        require!(
            accumulated_percentage == MAX_PERCENTAGE,
            "The total percentage must be 100%"
        );

        let mystery_box_token_nonce = self.send().esdt_nft_create_compact(
            &mystery_box_token_id,
            &BigUint::from(NFT_AMOUNT),
            &mystery_box_setup,
        );

        let user_payment = EsdtTokenPayment::new(
            mystery_box_token_id,
            mystery_box_token_nonce,
            BigUint::from(1u64),
        );
        self.send().direct_esdt(
            &receiver,
            &user_payment.token_identifier,
            user_payment.token_nonce,
            &user_payment.amount,
        );

        self.emit_create_mystery_box_event(&receiver, &mystery_box_setup);

        user_payment
    }

    #[payable("*")]
    #[endpoint(openMysteryBox)]
    fn open_mystery_box(&self) {
        let mystery_box_token_id = self.mystery_box_token_identifier().get();
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == mystery_box_token_id,
            "Bad payment token"
        );
        let mystery_box_attributes: ManagedVec<Reward<Self::Api>> = self
            .blockchain()
            .get_esdt_token_data(
                &self.blockchain().get_sc_address(),
                &payment.token_identifier,
                payment.token_nonce,
            )
            .decode_attributes();

        let current_epoch = self.blockchain().get_block_epoch();
        let cooldown_epoch = self.cooldown_epoch().get();
        let mut active_cooldown = true;
        let mut winning_reward = Reward::default();
        while active_cooldown {
            winning_reward = self.get_winning_reward(&mystery_box_attributes);
            active_cooldown =
                self.check_reward_cooldown(current_epoch, cooldown_epoch, &winning_reward);
        }

        // We send the tokens only for the rewards of type FixedValue
        if winning_reward.reward_type == RewardType::FixedValue {
            let caller = self.blockchain().get_caller();
            let reward_reserves = self.blockchain().get_sc_balance(
                &winning_reward.reward_token_id,
                winning_reward.reward_token_nonce,
            );
            require!(
                reward_reserves >= winning_reward.value,
                "Not enough reward reserves available"
            );
            self.send().direct(
                &caller,
                &winning_reward.reward_token_id,
                winning_reward.reward_token_nonce,
                &winning_reward.value,
            );
        }

        self.send().esdt_local_burn(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );

        self.emit_open_mystery_box_event(&winning_reward);
    }

    // We limit the deposit function only to admins
    // Can be further extended to store reward amounts in storage
    #[payable("*")]
    #[endpoint(depositRewards)]
    fn deposit_rewards(&self) {
        let caller = self.blockchain().get_caller();
        self.require_admin(&caller);
    }
}
