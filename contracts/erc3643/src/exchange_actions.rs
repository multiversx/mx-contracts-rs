use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

use crate::hooks::hook_type::ErcHookType;

multiversx_sc::imports!();

pub type EndpointName<M> = ManagedBuffer<M>;

#[multiversx_sc::module]
pub trait ExchangeActionsModule:
    crate::users::UsersModule
    + crate::hooks::call_hook::CallHookModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[only_owner]
    #[endpoint(addExchangeEndpoint)]
    fn add_exchange_endpoint(
        &self,
        sc_addr: ManagedAddress,
        endpoint_names: MultiValueEncoded<EndpointName<Self::Api>>,
    ) {
        let mut mapper = self.known_contracts(&sc_addr);
        for new_endpoint in endpoint_names {
            let _ = mapper.insert(new_endpoint);
        }
    }

    #[only_owner]
    #[endpoint(removeExchangeEndpoint)]
    fn remove_exchange_endpoint(
        &self,
        sc_addr: ManagedAddress,
        endpoint_names: MultiValueEncoded<ManagedBuffer>,
    ) {
        let mut mapper = self.known_contracts(&sc_addr);
        for endpoint_to_remove in endpoint_names {
            let is_removed = mapper.swap_remove(&endpoint_to_remove);

            require!(is_removed, "Unknown endpoint name");
        }
    }

    /// forward an execute on dest context call on an exchange SC
    #[payable("*")]
    #[endpoint(forwardExecuteOnDest)]
    fn forward_execute_on_dest(
        &self,
        dest: ManagedAddress,
        endpoint_name: ManagedBuffer,
        extra_args: MultiValueEncoded<ManagedBuffer>,
    ) -> PaymentsVec<Self::Api> {
        self.require_not_paused();
        self.require_known_endpoint(&dest, &endpoint_name);

        let egld_value = self.call_value().egld_value().clone_value();
        require!(egld_value == 0, "Invalid payment");

        let caller = self.blockchain().get_caller();
        self.require_whitelisted(&caller);

        let payments = self.call_value().all_esdt_transfers().clone_value();
        let payments_after_hook = self.call_hook(
            ErcHookType::BeforeExchangeAction,
            caller.clone(),
            payments,
            extra_args.to_vec(),
        );

        let back_transfers = self
            .tx()
            .to(dest)
            .raw_call(endpoint_name)
            .arguments_raw(ManagedArgBuffer::from(extra_args.into_vec_of_buffers()))
            .with_multi_token_transfer(payments_after_hook)
            .returns(ReturnsBackTransfers)
            .sync_call();

        let output_payments = self.call_hook(
            ErcHookType::AfterExchangeAction,
            caller.clone(),
            back_transfers.esdt_payments,
            ManagedVec::new(),
        );

        if !output_payments.is_empty() {
            self.tx().to(ToCaller).payment(&output_payments).transfer();
        }

        output_payments
    }

    fn require_known_endpoint(&self, dest: &ManagedAddress, endpoint_name: &ManagedBuffer) {
        let known_sc_mapper = self.known_contracts(dest);
        require!(
            !known_sc_mapper.is_empty(),
            "Unknown SC, use forwardTransfer endpoint"
        );

        require!(known_sc_mapper.contains(endpoint_name), "Unknown endpoint");
    }

    #[storage_mapper("knownContracts")]
    fn known_contracts(
        &self,
        sc_addr: &ManagedAddress,
    ) -> UnorderedSetMapper<EndpointName<Self::Api>>;
}
