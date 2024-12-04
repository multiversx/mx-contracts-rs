use multiversx_sc::imports::*;

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

static ERR_CALLBACK_MSG: &[u8] = b"Error received in callback:";

#[multiversx_sc::module]
pub trait ForwardCall {
    fn forward_call(
        &self,
        dest: ManagedAddress,
        min_gas_limit: u64,
        endpoint_name: ManagedBuffer,
        payments: PaymentsVec<Self::Api>,
        endpoint_args: MultiValueEncoded<ManagedBuffer>,
    ) {
        self.require_min_gas_limit(min_gas_limit);
        let original_caller = self.blockchain().get_caller();

        self.tx()
            .to(&dest)
            .raw_call(endpoint_name)
            .arguments_raw(endpoint_args.to_arg_buffer())
            .payment(payments.clone())
            .callback(
                self.callbacks()
                    .transfer_callback(original_caller, payments),
            )
            .async_call_and_exit();
    }

    #[callback]
    fn transfer_callback(
        &self,
        original_caller: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
        #[call_result] result: ManagedAsyncCallResult<MultiValueEncoded<ManagedBuffer>>,
    ) -> MultiValueEncoded<ManagedBuffer> {
        match result {
            ManagedAsyncCallResult::Ok(return_values) => {
                let back_transfers = self.blockchain().get_back_transfers();

                // Send the original input tokens back to the original caller
                if !back_transfers.esdt_payments.is_empty() {
                    self.tx()
                        .to(&original_caller)
                        .payment(&back_transfers.esdt_payments)
                        .transfer();
                }
                if back_transfers.total_egld_amount > 0 {
                    self.tx()
                        .to(&original_caller)
                        .egld(back_transfers.total_egld_amount)
                        .transfer();
                }
                // Send the resulted tokens to the original caller
                return_values
            }
            ManagedAsyncCallResult::Err(err) => {
                // Send the resulted tokens to the original caller
                self.tx().to(&original_caller).payment(payments).transfer();

                let mut err_result = MultiValueEncoded::new();
                err_result.push(ManagedBuffer::new_from_bytes(ERR_CALLBACK_MSG));
                err_result.push(err.err_msg);

                err_result
            }
        }
    }

    fn require_min_gas_limit(&self, min_gas_limit: u64) {
        let gas_left = self.blockchain().get_gas_left();
        require!(
            gas_left >= min_gas_limit,
            "Minimum required gas not provided"
        );
    }
}
