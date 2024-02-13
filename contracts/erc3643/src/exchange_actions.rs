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
        let mut new_endpoints = ManagedVec::<Self::Api, _>::new();
        for endpoint_name in endpoint_names {
            new_endpoints.push(endpoint_name);
        }

        let mapper = self.known_contracts(&sc_addr);
        let mut current_endpoints = mapper.get();
        for new_endpoint in &new_endpoints {
            for current_endpoint in &current_endpoints {
                require!(current_endpoint != new_endpoint, "Endpoint already known");
            }

            current_endpoints.push(new_endpoint);
        }

        mapper.set(current_endpoints);
    }

    #[only_owner]
    #[endpoint(removeExchangeEndpoint)]
    fn remove_exchange_endpoint(
        &self,
        sc_addr: ManagedAddress,
        endpoint_names: MultiValueEncoded<ManagedBuffer>,
    ) {
        let mapper = self.known_contracts(&sc_addr);
        let mut current_endpoints = mapper.get();

        for endpoint_to_remove in endpoint_names {
            let mut removed = false;
            for (i, endpoint) in current_endpoints.iter().enumerate() {
                if *endpoint == endpoint_to_remove {
                    removed = true;
                    current_endpoints.remove(i);

                    break;
                }
            }

            require!(removed, "Unknown endpoint name");
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

        let (_, back_transfers) =
            ContractCallNoPayment::<_, MultiValueEncoded<ManagedBuffer>>::new(dest, endpoint_name)
                .with_multi_token_transfer(payments_after_hook)
                .with_raw_arguments(ManagedArgBuffer::from(extra_args.into_vec_of_buffers()))
                .execute_on_dest_context_with_back_transfers::<MultiValueEncoded<ManagedBuffer>>();

        let output_payments = self.call_hook(
            ErcHookType::AfterExchangeAction,
            caller.clone(),
            back_transfers.esdt_payments,
            ManagedVec::new(),
        );

        if !output_payments.is_empty() {
            self.send().direct_multi(&caller, &output_payments);
        }

        output_payments
    }

    fn require_known_endpoint(&self, dest: &ManagedAddress, endpoint_name: &ManagedBuffer) {
        let known_sc_mapper = self.known_contracts(dest);
        require!(
            !known_sc_mapper.is_empty(),
            "Unknown SC, use forwardTransfer endpoint"
        );

        let endpoint_names = known_sc_mapper.get();
        for name in &endpoint_names {
            if &name == endpoint_name {
                return;
            }
        }

        sc_panic!("Unknown endpoint");
    }

    #[storage_mapper("knownContracts")]
    fn known_contracts(
        &self,
        sc_addr: &ManagedAddress,
    ) -> SingleValueMapper<ManagedVec<EndpointName<Self::Api>>>;
}
