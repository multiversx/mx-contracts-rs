multiversx_sc::imports!();

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

static ERR_CALLBACK_MSG: &[u8] = b"Error received in callback:";
pub const ESDT_TRANSFER_FUNC_NAME: &str = "ESDTTransfer";
#[multiversx_sc::module]
pub trait ForwardCall {
    fn forward_call(
        &self,
        dest: ManagedAddress,
        endpoint_name: ManagedBuffer,
        endpoint_args: MultiValueEncoded<ManagedBuffer>,
        payments: PaymentsVec<Self::Api>,
    ) {
        let original_caller = self.blockchain().get_caller();

        let (contract_call_endpoint, contract_call_args) =
            if !self.blockchain().is_smart_contract(&dest) {
                let mut args_buffer = ManagedArgBuffer::new();
                args_buffer.push_arg(endpoint_name);

                (ESDT_TRANSFER_FUNC_NAME.into(), args_buffer)
            } else {
                (endpoint_name, endpoint_args.to_arg_buffer())
            };

        self.send()
            .contract_call::<()>(dest, contract_call_endpoint)
            .with_raw_arguments(contract_call_args)
            .with_multi_token_transfer(payments)
            .async_call()
            .with_callback(self.callbacks().transfer_callback(original_caller))
            .call_and_exit();
    }
    #[callback]
    fn transfer_callback(
        &self,
        original_caller: ManagedAddress,
        // initial_payments: ManagedVec<EsdtTokenPayment<Self::Api>>,
        #[call_result] result: ManagedAsyncCallResult<MultiValueEncoded<ManagedBuffer>>,
    ) -> MultiValueEncoded<ManagedBuffer> {
        let initial_payments = self.call_value().all_esdt_transfers();

        match result {
            ManagedAsyncCallResult::Ok(return_values) => return_values,
            ManagedAsyncCallResult::Err(err) => {
                if !initial_payments.is_empty() {
                    self.send()
                        .direct_multi(&original_caller, &initial_payments);
                }

                let mut err_result = MultiValueEncoded::new();
                err_result.push(ManagedBuffer::new_from_bytes(ERR_CALLBACK_MSG));
                err_result.push(err.err_msg.clone());

                sc_print!("{}", err.err_msg);

                err_result
            }
        }
    }
}
