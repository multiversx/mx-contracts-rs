multiversx_sc::imports!();

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

static ERR_CALLBACK_MSG: &[u8] = b"Error received in callback:";

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

        if !self.blockchain().is_smart_contract(&dest) {
            self.send().direct_multi(&dest, &payments);
        } else {
            let mut args_buffer = ManagedArgBuffer::new();
            for arg in endpoint_args {
                args_buffer.push_arg(arg);
            }

            ContractCallWithMultiEsdt::<Self::Api, ()>::new(dest, endpoint_name, payments.clone())
                .with_raw_arguments(args_buffer)
                .async_call()
                .with_callback(
                    self.callbacks()
                        .transfer_callback(original_caller, payments),
                )
                .call_and_exit();
        }
    }
    #[callback]
    fn transfer_callback(
        &self,
        original_caller: ManagedAddress,
        initial_payments: ManagedVec<EsdtTokenPayment<Self::Api>>,
        #[call_result] result: ManagedAsyncCallResult<MultiValueEncoded<ManagedBuffer>>,
    ) -> MultiValueEncoded<ManagedBuffer> {
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
