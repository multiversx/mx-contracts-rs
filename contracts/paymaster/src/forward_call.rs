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

        self.send()
            .contract_call::<()>(dest, endpoint_name)
            .with_raw_arguments(endpoint_args.to_arg_buffer())
            .with_multi_token_transfer(payments)
            .async_call()
            .with_callback(self.callbacks().transfer_callback(original_caller))
            .call_and_exit();
    }

    #[callback]
    fn transfer_callback(
        &self,
        original_caller: ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<MultiValueEncoded<ManagedBuffer>>,
    ) -> MultiValueEncoded<ManagedBuffer> {
        // TODO: use ManagedGetBackTransfers once rc1.6 is activated
        let back_transfers = self.blockchain().get_back_transfers();
        let payments = self.call_value().all_esdt_transfers();

        self.send().direct_multi(&original_caller, &payments.clone_value());

        // Send the original input tokens back to the original caller
        if !back_transfers.esdt_payments.is_empty() {
            self.send()
                .direct_multi(&original_caller, &back_transfers.esdt_payments);
        }
        if back_transfers.total_egld_amount != BigUint::zero() {
            self.send()
                .direct_egld(&original_caller, &back_transfers.total_egld_amount)
        }

        match result {
            ManagedAsyncCallResult::Ok(return_values) => {
                // Send the resulted tokens to the original caller
                return_values
            }
            ManagedAsyncCallResult::Err(err) => {
                let mut err_result = MultiValueEncoded::new();
                err_result.push(ManagedBuffer::new_from_bytes(ERR_CALLBACK_MSG));
                err_result.push(err.err_msg);

                err_result
            }
        }
    }
}
