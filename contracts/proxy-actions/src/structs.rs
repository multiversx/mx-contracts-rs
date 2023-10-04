multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, ManagedVecItem)]
pub enum TaskType {
    None,
    WrapEGLD,
    UnwrapEgld,
    Swap,
    SendEsdt,
    ExitLP,
}

#[multiversx_sc::module]
pub trait TaskCall {
    #[payable("*")]
    #[endpoint(composeTasks1)]
    fn compose_tasks1(
        &self,
        tasks: MultiValueEncoded<
            MultiValue5<
                TaskType,
                ManagedAddress,
                ManagedBuffer,
                MultiValueEncoded<ManagedBuffer>,
                u64,
            >,
        >,
    ) //-> Result<(), EsdtTokenPayment>
    {
        let payment = self.call_value().egld_or_single_esdt();
        let mut payment_to_next_task = payment.clone();

        for task in tasks.into_iter() {
            let (task_type, dest_addr, endpoint_name, endpoint_args, gas_limit) = task.into_tuple();

            let payment_to_next_task = self
                .send()
                .contract_call::<(EsdtTokenPayment)>(dest_addr, endpoint_name)
                .with_egld_or_single_esdt_transfer(payment)
                .with_raw_arguments(endpoint_args.to_arg_buffer())
                .with_gas_limit(gas_limit)
                .transfer_execute();
        }
        // let result;
        // for proxy_action_id in proxy_action_ids {
        //     let proxy_action = self.proxy_actions(proxy_action_id).get();

        //     // Payments options
        //     let payments = self.call_value().all_esdt_transfers();
        //     // let payments = egld payment based on ContractCallType
        //     // let payments = use the previous result;

        //     let contract_call =
        //         ContractCallNoPayment::new(proxy_address, call.endpoint_name, payments);

        //     for arg in proxy_action.args {
        //         // or use previous result
        //         contract_call.proxy_arg(&arg);
        //     }

        //     raw_result = match contract_call_type {
        //         ContractCallType::ContractCallWithAnyPayment => {
        //             contract_call
        //                 .with_gas_limit(proxy_action.gas_limit)
        //                 .transfer_execute();
        //         }
        //     }

        //     result = raw_result.decode()
        // }
    }

    #[payable("*")]
    #[endpoint(composeTasks2)]
    fn compose_tasks2(&self, tasks: MultiValueEncoded<MultiValue3<TaskType, ManagedAddress, u64>>)
    //-> Result<(), EsdtTokenPayment>
    {
        let payment = self.call_value().egld_or_single_esdt();
        let mut payment_to_next_task = payment.clone();

        for task in tasks.into_iter() {
            let (task_type, dest_addr, gas_limit) = task.into_tuple();

            let payment_to_next_task = match task_type {
                TaskType::WrapEGLD => {
                    //TODO
                }
                TaskType::UnwrapEgld => {
                    //TODO
                }
                TaskType::Swap => {
                    // self
                    // .pair_proxy(dest_addr)
                    // .swap_tokens_fixed_input(requested_token_id, min_amount_out)
                    // .with_esdt_transfer(payment)
                    // .execute_on_dest_context(),
                }
                TaskType::SendEsdt => {
                    self.send()
                        .contract_call::<(EsdtTokenPayment)>(dest_addr, b"")
                        .with_egld_or_single_esdt_transfer(payment)
                        .with_gas_limit(gas_limit)
                        .transfer_execute();
                }
                TaskType::ExitLP => {
                    //TODO
                }
                TaskType::None => {
                    // return Ok(())
                }
            };
        }
        // let result;
        // for proxy_action_id in proxy_action_ids {
        //     let proxy_action = self.proxy_actions(proxy_action_id).get();

        //     // Payments options
        //     let payments = self.call_value().all_esdt_transfers();
        //     // let payments = egld payment based on ContractCallType
        //     // let payments = use the previous result;

        //     let contract_call =
        //         ContractCallNoPayment::new(proxy_address, call.endpoint_name, payments);

        //     for arg in proxy_action.args {
        //         // or use previous result
        //         contract_call.proxy_arg(&arg);
        //     }

        //     raw_result = match contract_call_type {
        //         ContractCallType::ContractCallWithAnyPayment => {
        //             contract_call
        //                 .with_gas_limit(proxy_action.gas_limit)
        //                 .transfer_execute();
        //         }
        //     }

        //     result = raw_result.decode()
        // }
    }
    #[proxy]
    fn pair_contract_proxy(&self, to: ManagedAddress) -> pair::Proxy<Self::Api>;
}
