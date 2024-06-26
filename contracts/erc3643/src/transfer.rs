use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

use crate::hooks::hook_type::ErcHookType;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct CallbackArgs<M: ManagedTypeApi> {
    pub payments: PaymentsVec<M>,
    pub original_caller: ManagedAddress<M>,
}

#[multiversx_sc::module]
pub trait TransferModule:
    crate::exchange_actions::ExchangeActionsModule
    + crate::users::UsersModule
    + crate::hooks::call_hook::CallHookModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// Forward the transfer to the specified address
    /// Part of the tokens may be taken as fees
    /// If the destination is a SC, the first argument is the function name
    #[payable("*")]
    #[endpoint(forwardTransfer)]
    fn forward_transfer(&self, dest: ManagedAddress, extra_args: MultiValueEncoded<ManagedBuffer>) {
        self.require_not_paused();
        require!(
            self.known_contracts(&dest).is_empty(),
            "Cannot transfer to this SC. Use forwardExecuteOnDest instead."
        );

        let payments = self.call_value().all_esdt_transfers().clone_value();
        require!(!payments.is_empty(), "Empty payments");

        self.check_transfer_allowed(&dest, &payments);

        let caller = self.blockchain().get_caller();
        self.require_whitelisted(&caller);

        let payments_after_hook = self.call_hook(
            ErcHookType::BeforeTransfer,
            caller.clone(),
            payments,
            extra_args.to_vec(),
        );

        if !self.blockchain().is_smart_contract(&dest) {
            self.tx().to(dest).payment(&payments_after_hook).transfer();

            return;
        }

        require!(!extra_args.is_empty(), "No arguments for SC Call");

        let all_args = extra_args.into_vec_of_buffers();
        let endpoint_name = all_args.get(0).clone_value();
        let func_args = match all_args.slice(1, all_args.len()) {
            Some(args) => args,
            None => ManagedVec::new(),
        };
        self.transfer_to_sc(caller, dest, payments_after_hook, endpoint_name, func_args);
    }

    fn check_transfer_allowed(&self, _dest: &ManagedAddress, _payments: &PaymentsVec<Self::Api>) {
        // custom user logic
    }

    fn transfer_to_sc(
        &self,
        caller: ManagedAddress,
        dest: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
        endpoint_name: ManagedBuffer,
        args: ManagedVec<ManagedBuffer>,
    ) -> ! {
        let cb_args = CallbackArgs {
            payments: payments.clone(),
            original_caller: caller,
        };
        self.tx()
            .to(dest)
            .raw_call(endpoint_name)
            .arguments_raw(ManagedArgBuffer::from(args))
            .with_multi_token_transfer(payments.clone())
            .with_callback(
                <Self as TransferModule>::callbacks(self).transfer_to_sc_callback(cb_args),
            )
            .async_call_and_exit();
    }

    #[callback]
    fn transfer_to_sc_callback(
        &self,
        args: CallbackArgs<Self::Api>,
        #[call_result] call_result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        if call_result.is_err() {
            self.tx()
                .to(&args.original_caller)
                .payment(&args.payments)
                .transfer();
        }
    }
}
