use crate::common::{TakeFeesResult, MAX_FEE_PERCENTAGE};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct EndpointInfo<M: ManagedTypeApi> {
    pub endpoint_name: ManagedBuffer<M>,
    pub input_fee_percentage: u32,
    pub burn_input: bool,
    pub output_fee_percentage: u32,
    pub burn_output: bool,
}

#[multiversx_sc::module]
pub trait ExchangeActionsModule: crate::common::CommonModule {
    /// Arguments: endpoint_name,
    /// input_fee_percentage: between 0 and 10_000,
    /// burn_input: bool, burn input tokens taken as fee,
    /// output_fee_percentage: between 0 and 10_000,
    /// burn_output: bool, burn output taken as fee
    #[only_owner]
    #[endpoint(addExchangeEndpoint)]
    fn add_exchange_endpoint(
        &self,
        sc_addr: ManagedAddress,
        endpoint_name_fee_percentage_pairs: MultiValueEncoded<
            MultiValue5<ManagedBuffer, u32, bool, u32, bool>,
        >,
    ) {
        let mut new_endpoints = ManagedVec::<Self::Api, EndpointInfo<Self::Api>>::new();
        for pair in endpoint_name_fee_percentage_pairs {
            let (
                endpoint_name,
                input_fee_percentage,
                burn_input,
                output_fee_percentage,
                burn_output,
            ) = pair.into_tuple();

            require!(
                input_fee_percentage <= MAX_FEE_PERCENTAGE
                    && output_fee_percentage <= MAX_FEE_PERCENTAGE,
                "Invalid fee percentage"
            );

            new_endpoints.push(EndpointInfo {
                endpoint_name,
                input_fee_percentage,
                burn_input,
                output_fee_percentage,
                burn_output,
            });
        }

        let mapper = self.known_contracts(&sc_addr);
        let mut current_endpoints = mapper.get();
        for new_endpoint in &new_endpoints {
            for current_endpoint in &current_endpoints {
                require!(
                    current_endpoint.endpoint_name != new_endpoint.endpoint_name,
                    "Endpoint already known"
                );
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
                if endpoint.endpoint_name == endpoint_to_remove {
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
    ) {
        let egld_value = self.call_value().egld_value().clone_value();
        require!(egld_value == 0, "Invalid payment");

        let caller = self.blockchain().get_caller();
        let payments = self.call_value().all_esdt_transfers().clone_value();
        let endpoint_info = self.find_endpoint_info(&dest, &endpoint_name);

        let mut input_fees_percentage = ManagedVec::new();
        for _ in 0..payments.len() {
            input_fees_percentage.push(endpoint_info.input_fee_percentage);
        }

        let (_, back_transfers) = if !payments.is_empty() {
            let take_fees_result =
                self.take_fees(caller.clone(), payments.clone(), input_fees_percentage);

            if endpoint_info.burn_input {
                self.burn_tokens(&take_fees_result.fees);
            }

            ContractCallNoPayment::<_, MultiValueEncoded<ManagedBuffer>>::new(dest, endpoint_name)
                .with_multi_token_transfer(take_fees_result.transfers)
                .with_raw_arguments(ManagedArgBuffer::from(extra_args.into_vec_of_buffers()))
                .execute_on_dest_context_with_back_transfers::<MultiValueEncoded<ManagedBuffer>>()
        } else {
            ContractCallNoPayment::<_, MultiValueEncoded<ManagedBuffer>>::new(dest, endpoint_name)
                .with_raw_arguments(ManagedArgBuffer::from(extra_args.into_vec_of_buffers()))
                .execute_on_dest_context_with_back_transfers::<MultiValueEncoded<ManagedBuffer>>()
        };

        if !back_transfers.esdt_payments.is_empty() {
            let mut output_fees_percentage = ManagedVec::new();
            for _ in 0..back_transfers.esdt_payments.len() {
                output_fees_percentage.push(endpoint_info.output_fee_percentage);
            }

            let take_fees_from_results =
                self.take_fees(caller, back_transfers.esdt_payments, output_fees_percentage);

            if endpoint_info.burn_output {
                self.burn_tokens(&take_fees_from_results.fees);
            }

            self.send().direct_multi(
                &take_fees_from_results.original_caller,
                &take_fees_from_results.transfers,
            );
        }
    }

    /// forward an async call on an exchange SC. In case of failure, all tokens are returned to the user.
    #[payable("*")]
    #[endpoint(forwardAsyncCall)]
    fn forward_async_call(
        &self,
        dest: ManagedAddress,
        endpoint_name: ManagedBuffer,
        extra_args: MultiValueEncoded<ManagedBuffer>,
    ) {
        let egld_value = self.call_value().egld_value().clone_value();
        require!(egld_value == 0, "Invalid payment");

        let caller = self.blockchain().get_caller();
        let payments = self.call_value().all_esdt_transfers().clone_value();
        let endpoint_info = self.find_endpoint_info(&dest, &endpoint_name);

        let mut input_fees_percentage = ManagedVec::new();
        for _ in 0..payments.len() {
            input_fees_percentage.push(endpoint_info.input_fee_percentage);
        }

        let take_fees_result =
            self.take_fees(caller.clone(), payments.clone(), input_fees_percentage);

        if !payments.is_empty() {
            ContractCallNoPayment::<_, MultiValueEncoded<ManagedBuffer>>::new(dest, endpoint_name)
                .with_multi_token_transfer(take_fees_result.transfers.clone())
                .with_raw_arguments(ManagedArgBuffer::from(extra_args.into_vec_of_buffers()))
                .async_call()
                .with_callback(self.callbacks().call_exchange_async_callback(
                    take_fees_result,
                    endpoint_info.burn_input,
                    endpoint_info.output_fee_percentage,
                    endpoint_info.burn_output,
                ))
                .call_and_exit();
        } else {
            ContractCallNoPayment::<_, MultiValueEncoded<ManagedBuffer>>::new(dest, endpoint_name)
                .with_raw_arguments(ManagedArgBuffer::from(extra_args.into_vec_of_buffers()))
                .async_call()
                .with_callback(self.callbacks().call_exchange_async_callback(
                    take_fees_result,
                    endpoint_info.burn_input,
                    endpoint_info.output_fee_percentage,
                    endpoint_info.burn_output,
                ))
                .call_and_exit();
        }
    }

    #[callback]
    fn call_exchange_async_callback(
        &self,
        input_take_fees_result: TakeFeesResult<Self::Api>,
        burn_input: bool,
        output_fee_percentage: u32,
        burn_output: bool,
        #[call_result] call_result: ManagedAsyncCallResult<MultiValueEncoded<ManagedBuffer>>,
    ) {
        match call_result {
            ManagedAsyncCallResult::Ok(_) => {
                if !input_take_fees_result.fees.is_empty() && burn_input {
                    self.burn_tokens(&input_take_fees_result.fees);
                }

                let back_transfers = self.blockchain().get_back_transfers();
                if !back_transfers.esdt_payments.is_empty() {
                    let mut output_fees_percentage = ManagedVec::new();
                    for _ in 0..back_transfers.esdt_payments.len() {
                        output_fees_percentage.push(output_fee_percentage);
                    }

                    let take_fees_from_results = self.take_fees(
                        input_take_fees_result.original_caller,
                        back_transfers.esdt_payments,
                        output_fees_percentage,
                    );

                    if burn_output {
                        self.burn_tokens(&take_fees_from_results.fees);
                    }

                    self.send().direct_multi(
                        &take_fees_from_results.original_caller,
                        &take_fees_from_results.transfers,
                    );
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                self.send().direct_multi(
                    &input_take_fees_result.original_caller,
                    &input_take_fees_result.original_payments,
                );
            }
        }
    }

    fn find_endpoint_info(
        &self,
        dest: &ManagedAddress,
        endpoint_name: &ManagedBuffer,
    ) -> EndpointInfo<Self::Api> {
        let known_sc_mapper = self.known_contracts(&dest);
        require!(
            !known_sc_mapper.is_empty(),
            "Unknown SC, use forwardTransfer endpoint"
        );

        let endpoints_info = known_sc_mapper.get();
        let mut opt_info = None;
        for info in &endpoints_info {
            if &info.endpoint_name == endpoint_name {
                opt_info = Some(info);
                break;
            }
        }

        match opt_info {
            Some(info) => info,
            None => sc_panic!("Unknown endpoint"),
        }
    }

    #[storage_mapper("knownContracts")]
    fn known_contracts(
        &self,
        sc_addr: &ManagedAddress,
    ) -> SingleValueMapper<ManagedVec<EndpointInfo<Self::Api>>>;
}
