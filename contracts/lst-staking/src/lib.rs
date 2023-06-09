#![no_std]

multiversx_sc::imports!();

mod esdt;
use esdt::{AvailableAmount, Esdt};

#[multiversx_sc::contract]
pub trait LstStakingContract {
    #[init]
    fn init(&self, unbond_period: u64) {
        self.unbond_period().set(unbond_period);
    }

    #[only_owner]
    #[endpoint]
    fn whitelist_token(&self, token: TokenIdentifier) {
        let _ = self.token_whitelist().insert(token);
    }

    #[only_owner]
    #[endpoint]
    fn blacklist_token(&self, token: &TokenIdentifier) {
        let _ = self.token_whitelist().swap_remove(token);
    }

    #[payable("*")]
    #[endpoint]
    fn stake(&self) {
        let payments = self.call_value().all_esdt_transfers();
        let caller = self.blockchain().get_caller();
        for payment in payments.iter() {
            self.require_whitelisted_token(&payment.token_identifier);
            self.stake_token(&caller, payment);
        }
    }

    fn require_whitelisted_token(&self, token: &TokenIdentifier) {
        require!(
            self.token_whitelist().contains(token),
            "The provided token is not whitelisted"
        );
    }

    fn stake_token(&self, caller: &ManagedAddress, payment: EsdtTokenPayment) {
        self.staked_tokens(caller, &payment.token_identifier)
            .update(|amount| *amount += &payment.amount);
    }

    #[endpoint]
    fn unstake(&self, tokens: ManagedVec<Esdt<Self::Api>>) {
        let caller = self.blockchain().get_caller();
        let block_epoch = self.blockchain().get_block_epoch();
        for token in tokens.iter() {
            self.staked_tokens(&caller, &token.token_identifier)
                .update(|staked_amount| {
                    if *staked_amount >= token.amount {
                        *staked_amount -= token.amount.clone();
                        let available_amount = AvailableAmount {
                            epoch: block_epoch + self.unbond_period().get(),
                            amount: token.amount,
                        };
                        self.unstaked_tokens(&caller, &token.token_identifier)
                            .insert(available_amount);
                    }
                });
        }
    }

    #[endpoint]
    fn unbond(&self, tokens: ManagedVec<TokenIdentifier>) {
        let caller = self.blockchain().get_caller();
        let block_epoch = self.blockchain().get_block_epoch();
        let mut unbond_tokens = ManagedVec::<Self::Api, EsdtTokenPayment<Self::Api>>::new();
        for token_identifier in tokens.iter() {
            for available_amount in self.unstaked_tokens(&caller, &token_identifier).iter() {
                if available_amount.epoch >= block_epoch {
                    unbond_tokens.push(EsdtTokenPayment::new(
                        token_identifier.clone_value(),
                        0,
                        available_amount.amount.clone(),
                    ));
                    self.unstaked_tokens(&caller, &token_identifier)
                        .swap_remove(&available_amount);
                }
            }
        }
        if !unbond_tokens.is_empty() {
            self.send().direct_multi(&caller, &unbond_tokens);
        }
    }

    #[storage_mapper("staked_tokens")]
    fn staked_tokens(
        &self,
        address: &ManagedAddress,
        token: &TokenIdentifier,
    ) -> SingleValueMapper<BigUint>;

    #[storage_mapper("unstaked_tokens")]
    fn unstaked_tokens(
        &self,
        address: &ManagedAddress,
        token: &TokenIdentifier,
    ) -> UnorderedSetMapper<AvailableAmount<Self::Api>>;

    #[storage_mapper("token_whitelist")]
    fn token_whitelist(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[storage_mapper("unbond_period")]
    fn unbond_period(&self) -> SingleValueMapper<u64>;
}
