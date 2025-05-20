use crate::common::{PaymentsVec, Percentage, TakeFeesResult, MAX_FEE_PERCENTAGE};

use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait TransferModule:
    crate::exchange_actions::ExchangeActionsModule
    + crate::initial_launch::InitialLaunchModule
    + crate::common::CommonModule
    + crate::token_info::TokenInfoModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// Percentage should be between 0 and 10_000
    #[only_owner]
    #[endpoint(setTokenFees)]
    fn set_token_fees(&self, token_id: TokenIdentifier, fees_percentage: Percentage) {
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
    #[payable]
    #[endpoint(forwardTransfer)]
    fn forward_transfer(&self, dest: ManagedAddress, extra_args: MultiValueEncoded<ManagedBuffer>) {
        self.require_not_paused();
        require!(
            self.known_contracts(&dest).is_empty(),
            "Cannot transfer to this SC. Use forwardExecuteOnDest or forwardAsyncCall instead."
        );

        let payments = self.call_value().all_esdt_transfers().clone();
        require!(!payments.is_empty(), "Empty payments");

        self.check_transfer_allowed(&dest, &payments);

        let caller = self.blockchain().get_caller();
        let mut fees_percentage = ManagedVec::new();
        for payment in &payments {
            let percentage = self.token_fees(&payment.token_identifier).get();
            fees_percentage.push(percentage);
        }

        let take_fees_result = self.take_fees(caller, payments, fees_percentage);

        if !self.blockchain().is_smart_contract(&dest) {
            let owner = self.blockchain().get_owner_address();
            self.tx()
                .to(&owner)
                .payment(&take_fees_result.fees)
                .transfer();

            self.tx()
                .to(&dest)
                .payment(&take_fees_result.transfers)
                .transfer();

            return;
        }

        require!(!extra_args.is_empty(), "No arguments for SC Call");

        let all_args = extra_args.into_vec_of_buffers();
        let endpoint_name = all_args.get(0).clone();
        let func_args = match all_args.slice(1, all_args.len()) {
            Some(args) => args,
            None => ManagedVec::new(),
        };
        self.transfer_to_sc(dest, take_fees_result, endpoint_name, func_args);
    }

    fn check_transfer_allowed(&self, _dest: &ManagedAddress, _payments: &PaymentsVec<Self::Api>) {
        // custom user logic
    }

    fn transfer_to_sc(
        &self,
        dest: ManagedAddress,
        take_fees_result: TakeFeesResult<Self::Api>,
        endpoint_name: ManagedBuffer,
        args: ManagedVec<ManagedBuffer>,
    ) -> ! {
        self.tx()
            .to(dest)
            .raw_call(endpoint_name)
            .arguments_raw(ManagedArgBuffer::from(args))
            .with_multi_token_transfer(take_fees_result.transfers.clone())
            .async_call_and_exit();
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
                    self.tx()
                        .to(&owner)
                        .payment(&take_fees_result.fees)
                        .transfer();
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                self.tx()
                    .to(&take_fees_result.original_caller)
                    .payment(&take_fees_result.original_payments)
                    .transfer();
            }
        }
    }
}
