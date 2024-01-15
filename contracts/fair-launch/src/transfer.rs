multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const MAX_FEE_PERCENTAGE: u32 = 10_000; // 100%

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct TakeFeesResult<M: ManagedTypeApi> {
    pub original_caller: ManagedAddress<M>,
    pub original_payments: ManagedVec<M, EsdtTokenPayment<M>>,
    pub fees: ManagedVec<M, EsdtTokenPayment<M>>,
    pub transfers: ManagedVec<M, EsdtTokenPayment<M>>,
}

#[multiversx_sc::module]
pub trait TransferModule {
    /// Percentage should be between 0 and 10_000
    #[only_owner]
    #[endpoint(setTokenFees)]
    fn set_token_fees(&self, token_id: TokenIdentifier, fees_percentage: u32) {
        require!(
            fees_percentage > 0 && fees_percentage <= MAX_FEE_PERCENTAGE,
            "Invalid fees percentage"
        );

        self.token_fees(&token_id).set(fees_percentage);
    }

    /// users in whitelist can transfer without fees
    #[only_owner]
    #[endpoint(addUsersToWhitelist)]
    fn add_users_to_whitelist(&self, users: MultiValueEncoded<ManagedAddress>) {
        let whitelist = self.user_whitelist();
        for user in users {
            whitelist.add(&user);
        }
    }

    #[only_owner]
    #[endpoint(removeUsersFromWhitelist)]
    fn remove_users_from_whitelist(&self, users: MultiValueEncoded<ManagedAddress>) {
        let whitelist = self.user_whitelist();
        for user in users {
            whitelist.remove(&user);
        }
    }

    /// Forward the transfer to the specified address
    /// Part of the tokens is taken as fees
    /// If the destination is a SC, the first argument is the function name
    #[payable("*")]
    #[endpoint(forwardTransfer)]
    fn forward_transfer(&self, dest: ManagedAddress, extra_args: MultiValueEncoded<ManagedBuffer>) {
        let payments = self.call_value().all_esdt_transfers().clone_value();
        require!(!payments.is_empty(), "Empty payments");

        self.check_transfer_allowed(&dest, &payments);

        let caller = self.blockchain().get_caller();
        let take_fees_result = self.take_fees(caller, payments);

        if !self.blockchain().is_smart_contract(&dest) {
            let owner = self.blockchain().get_owner_address();
            self.send().direct_multi(&owner, &take_fees_result.fees);

            self.send().direct_multi(&dest, &take_fees_result.transfers);

            return;
        }

        require!(!extra_args.is_empty(), "No arguments for SC Call");

        let all_args = extra_args.into_vec_of_buffers();
        let endpoint_name = all_args.get(0).clone_value();
        let func_args = match all_args.slice(1, all_args.len()) {
            Some(args) => args,
            None => ManagedVec::new(),
        };
        self.transfer_to_sc(dest, take_fees_result, endpoint_name, func_args);
    }

    fn check_transfer_allowed(
        &self,
        _dest: &ManagedAddress,
        _payments: &ManagedVec<EsdtTokenPayment>,
    ) {
        // custom user logic
    }

    fn take_fees(
        &self,
        caller: ManagedAddress,
        payments: ManagedVec<EsdtTokenPayment>,
    ) -> TakeFeesResult<Self::Api> {
        if self.user_whitelist().contains(&caller) {
            return TakeFeesResult {
                original_caller: caller,
                original_payments: payments.clone(),
                fees: ManagedVec::new(),
                transfers: payments,
            };
        }

        let original_payments = payments.clone();
        let mut final_payments = ManagedVec::new();
        let mut fees_payments = ManagedVec::new();

        for payment in &payments {
            let token_fees_percentage = self.token_fees(&payment.token_identifier).get();
            if token_fees_percentage == 0 {
                final_payments.push(payment);

                continue;
            }

            let fee_amount = self.calculate_fee_rounded_up(&payment.amount, token_fees_percentage);
            let user_payment = EsdtTokenPayment::new(
                payment.token_identifier.clone(),
                payment.token_nonce,
                &payment.amount - &fee_amount,
            );

            if fee_amount > 0 {
                let fee_payment = EsdtTokenPayment::new(
                    payment.token_identifier.clone(),
                    payment.token_nonce,
                    fee_amount,
                );
                fees_payments.push(fee_payment);
            }

            if user_payment.amount > 0 {
                final_payments.push(user_payment);
            }
        }

        TakeFeesResult {
            original_caller: caller,
            original_payments,
            fees: fees_payments,
            transfers: final_payments,
        }
    }

    fn calculate_fee_rounded_up(&self, payment_amount: &BigUint, fees_percentage: u32) -> BigUint {
        (payment_amount * fees_percentage + MAX_FEE_PERCENTAGE - 1u32) / MAX_FEE_PERCENTAGE
    }

    fn transfer_to_sc(
        &self,
        dest: ManagedAddress,
        take_fees_result: TakeFeesResult<Self::Api>,
        endpoint_name: ManagedBuffer,
        args: ManagedVec<ManagedBuffer>,
    ) -> ! {
        ContractCallNoPayment::<_, MultiValueEncoded<ManagedBuffer>>::new(dest, endpoint_name)
            .with_multi_token_transfer(take_fees_result.transfers.clone())
            .with_raw_arguments(ManagedArgBuffer::from(args))
            .async_call()
            .with_callback(self.callbacks().transfer_to_sc_callback(take_fees_result))
            .call_and_exit();
    }

    #[callback]
    fn transfer_to_sc_callback(
        &self,
        take_fees_result: TakeFeesResult<Self::Api>,
        #[call_result] call_result: ManagedAsyncCallResult<MultiValueEncoded<ManagedBuffer>>,
    ) {
        match call_result {
            ManagedAsyncCallResult::Ok(_) => {
                if !take_fees_result.fees.is_empty() {
                    let owner = self.blockchain().get_owner_address();
                    self.send().direct_multi(&owner, &take_fees_result.fees);
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                self.send().direct_multi(
                    &take_fees_result.original_caller,
                    &take_fees_result.original_payments,
                );
            }
        }
    }

    #[view(getTokenFees)]
    #[storage_mapper("tokenFees")]
    fn token_fees(&self, token_id: &TokenIdentifier) -> SingleValueMapper<u32>;

    #[storage_mapper("userWhitelist")]
    fn user_whitelist(&self) -> WhitelistMapper<Self::Api, ManagedAddress>;
}
