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

pub mod pair_proxy {
    #[multiversx_sc::proxy]
    pub trait PairProxy {
        #[payable("*")]
        #[endpoint(swapTokensFixedInput)]
        fn swap_tokens_fixed_input(
            &self,
            token_out: TokenIdentifier,
            amount_out_min: BigUint,
        ) -> EsdtTokenPayment;

        #[view(getSafePriceByDefaultOffset)]
        fn get_safe_price_by_default_offset(
            &self,
            pair_address: ManagedAddress,
            input_payment: EsdtTokenPayment,
        ) -> EsdtTokenPayment;
    }
}

#[multiversx_sc::module]
pub trait TaskCall {
    #[payable("*")]
    #[endpoint(composeTasks1)]
    fn compose_tasks1(
        &self,
        tasks: MultiValueEncoded<
            MultiValue4<ManagedAddress, ManagedBuffer, MultiValueEncoded<ManagedBuffer>, u64>,
        >,
    ) //-> Result<(), EsdtTokenPayment>
    {
        let payment = self.call_value().egld_or_single_esdt();
        let payment_to_next_task = payment.clone();

        for task in tasks.into_iter() {
            let (dest_addr, endpoint_name, endpoint_args, gas_limit) = task.into_tuple();

            let payment_to_next_task = self
                .send()
                .contract_call::<EsdtTokenPayment>(dest_addr, endpoint_name)
                .with_egld_or_single_esdt_transfer(payment_to_next_task.clone())
                .with_raw_arguments(endpoint_args.to_arg_buffer())
                .with_gas_limit(gas_limit)
                .transfer_execute();
        }
    }

    #[payable("*")]
    #[endpoint(composeTasks2)]
    fn compose_tasks2(
        &self,
        tasks: MultiValueEncoded<
            MultiValue4<TaskType, ManagedAddress, MultiValueEncoded<ManagedBuffer>, u64>,
        >,
    )
    //-> Result<(), EsdtTokenPayment>
    {
        let payment = self.call_value().egld_or_single_esdt();
        let payment_to_next_task = payment.clone();

        for task in tasks.into_iter() {
            let (task_type, dest_addr, endpoint_args, gas_limit) = task.into_tuple();

            let payment_to_next_task = match task_type {
                TaskType::WrapEGLD => {
                    //TODO
                }
                TaskType::UnwrapEgld => {
                    //TODO
                }
                TaskType::Swap => {
                    let args = endpoint_args.into_vec_of_buffers();
                    let token_out = TokenIdentifier::from(args.get(0).clone_value());
                    let min_amount_out = BigUint::from(args.get(1).clone_value());
                    let token_payment = payment_to_next_task
                        .token_identifier
                        .clone()
                        .into_esdt_option();
                    if token_payment.is_none() {
                        return;
                    }
                    self.pair_contract_proxy(dest_addr)
                        .swap_tokens_fixed_input(token_out, min_amount_out)
                        .with_esdt_transfer(EsdtTokenPayment::new(
                            token_payment.unwrap(),
                            payment_to_next_task.token_nonce,
                            payment_to_next_task.amount.clone(),
                        ))
                        .with_gas_limit(gas_limit)
                        .transfer_execute();
                }
                TaskType::SendEsdt => {
                    self.send()
                        .contract_call::<EsdtTokenPayment>(dest_addr, b"".into())
                        .with_egld_or_single_esdt_transfer(payment_to_next_task.clone())
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
    }
    #[proxy]
    fn pair_contract_proxy(&self, to: ManagedAddress) -> pair_proxy::Proxy<Self::Api>;
}
