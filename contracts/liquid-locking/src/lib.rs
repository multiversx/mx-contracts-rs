#![no_std]

mod unlocked_token;
use unlocked_token::UnlockedToken;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait LiquidLocking {
    #[init]
    fn init(&self, unbond_period: u64) {
        self.unbond_period().set_if_empty(unbond_period);
    }

    #[only_owner]
    #[endpoint]
    fn set_unbond_period(&self, unbond_period: u64) {
        self.unbond_period().set(unbond_period);
    }

    #[only_owner]
    #[endpoint]
    fn whitelist_token(&self, token: TokenIdentifier) {
        require!(token.is_valid_esdt_identifier(), "invalid token provided");
        let _ = self.token_whitelist().insert(token);
    }

    #[only_owner]
    #[endpoint]
    fn blacklist_token(&self, token: &TokenIdentifier) {
        let _ = self.token_whitelist().swap_remove(token);
    }

    #[payable("*")]
    #[endpoint]
    fn lock(&self) {
        let payments = self.call_value().all_esdt_transfers();
        let caller = self.blockchain().get_caller();
        for payment in payments.iter() {
            self.validate_payment(&payment);
            self.stake_token(&caller, payment);
        }
    }

    fn validate_payment(&self, payment: &EsdtTokenPayment) {
        require!(payment.token_nonce == 0, "invalid token provided");
        require!(payment.amount != 0, "amount must be greater than 0");
        require!(
            self.token_whitelist().contains(&payment.token_identifier),
            "token is not whitelisted"
        );
    }

    fn stake_token(&self, caller: &ManagedAddress, payment: EsdtTokenPayment) {
        self.locked_token_amounts(caller, &payment.token_identifier)
            .update(|amount| *amount += &payment.amount);

        if !self
            .locked_tokens(caller)
            .contains(&payment.token_identifier)
        {
            self.locked_tokens(caller).insert(payment.token_identifier);
        }
    }

    #[endpoint]
    fn unlock(&self, tokens: ManagedVec<EsdtTokenPayment<Self::Api>>) {
        let caller = self.blockchain().get_caller();
        let block_epoch = self.blockchain().get_block_epoch();
        for token in tokens.iter() {
            self.locked_token_amounts(&caller, &token.token_identifier)
                .update(|staked_amount| {
                    require!(token.amount > 0, "requested amount cannot be 0");
                    require!(*staked_amount >= token.amount, "unavailable amount");
                    *staked_amount -= token.amount.clone();
                    let unbounding_epoch = block_epoch + self.unbond_period().get();

                    self.unlocked_token_amounts(&caller, &token.token_identifier, unbounding_epoch)
                        .update(|amount| {
                            *amount += &token.amount;
                            if *amount == BigUint::zero() {
                                self.locked_tokens(&caller)
                                    .swap_remove(&token.token_identifier);
                            }
                        });

                    if !self
                        .unlocked_token_epochs(&caller, &token.token_identifier)
                        .contains(&unbounding_epoch)
                    {
                        self.unlocked_token_epochs(&caller, &token.token_identifier)
                            .insert(unbounding_epoch);
                    }
                    if !self
                        .unlocked_tokens(&caller)
                        .contains(&token.token_identifier)
                    {
                        self.unlocked_tokens(&caller).insert(token.token_identifier);
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
            let mut unbound_amount = BigUint::zero();
            let mut unbound_token_epochs = ManagedVec::<Self::Api, u64>::new();
            for epoch in self
                .unlocked_token_epochs(&caller, &token_identifier)
                .iter()
            {
                if epoch > block_epoch {
                    unbound_amount += self
                        .unlocked_token_amounts(&caller, &token_identifier, epoch)
                        .take();
                    unbound_token_epochs.push(epoch);
                }
            }
            for epoch in unbound_token_epochs.iter() {
                self.unlocked_token_epochs(&caller, &token_identifier)
                    .swap_remove(&epoch);

                if self
                    .unlocked_token_epochs(&caller, &token_identifier)
                    .is_empty()
                {
                    self.unlocked_tokens(&caller).swap_remove(&token_identifier);
                }
            }
            if unbound_amount > 0u64 {
                unbond_tokens.push(EsdtTokenPayment::new(
                    token_identifier.clone_value(),
                    0,
                    unbound_amount,
                ));
            }
        }
        if !unbond_tokens.is_empty() {
            self.send().direct_multi(&caller, &unbond_tokens);
        }
    }

    #[view(lockedTokenAmounts)]
    fn locked_token_amounts_by_address(
        &self,
        address: ManagedAddress,
    ) -> ManagedVec<EsdtTokenPayment> {
        let mut amounts = ManagedVec::<Self::Api, EsdtTokenPayment<Self::Api>>::new();
        for token in self.locked_tokens(&address).iter() {
            let amount = self.locked_token_amounts(&address, &token).get();
            let payment = EsdtTokenPayment::new(token, 0, amount);
            amounts.push(payment);
        }
        amounts
    }

    #[view(unlockedTokenAmounts)]
    fn unlocked_token_by_address(
        &self,
        address: ManagedAddress,
    ) -> ManagedVec<UnlockedToken<Self::Api>> {
        let mut amounts = ManagedVec::<Self::Api, UnlockedToken<Self::Api>>::new();
        for token in self.unlocked_tokens(&address).iter() {
            for epoch in self.unlocked_token_epochs(&address, &token).iter() {
                let amount = self.locked_token_amounts(&address, &token).get();
                let payment = EsdtTokenPayment::new(token.clone(), 0, amount);
                let unlocked_token = UnlockedToken {
                    token: payment,
                    unbond_epoch: epoch,
                };
                amounts.push(unlocked_token);
            }
        }
        amounts
    }

    #[view(lockedTokens)]
    #[storage_mapper("locked_tokens")]
    fn locked_tokens(&self, address: &ManagedAddress) -> UnorderedSetMapper<TokenIdentifier>;

    #[storage_mapper("locked_token_amounts")]
    fn locked_token_amounts(
        &self,
        address: &ManagedAddress,
        token: &TokenIdentifier,
    ) -> SingleValueMapper<BigUint>;

    #[view(unlockedTokens)]
    #[storage_mapper("unlocked_tokens")]
    fn unlocked_tokens(&self, address: &ManagedAddress) -> UnorderedSetMapper<TokenIdentifier>;

    #[storage_mapper("unlocked_token_epochs")]
    fn unlocked_token_epochs(
        &self,
        address: &ManagedAddress,
        token: &TokenIdentifier,
    ) -> UnorderedSetMapper<u64>;

    #[storage_mapper("unlocked_token_amounts")]
    fn unlocked_token_amounts(
        &self,
        address: &ManagedAddress,
        token: &TokenIdentifier,
        epoch: u64,
    ) -> SingleValueMapper<BigUint>;

    #[view(whitelistedTokens)]
    #[storage_mapper("token_whitelist")]
    fn token_whitelist(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(unbondPeriod)]
    #[storage_mapper("unbond_period")]
    fn unbond_period(&self) -> SingleValueMapper<u64>;
}
