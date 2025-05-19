use super::common::{PaymentsVec, RawCall};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AsyncCallModule: super::common::CommonModule {
    fn perform_raw_async_call_egld(
        &self,
        sc_address: ManagedAddress,
        raw_call_data: RawCall<Self::Api>,
        egld_value: BigUint,
    ) {
        self.require_dest_not_self(&sc_address);
        self.require_sc_address(&sc_address);

        let contract_call = self.build_raw_call_with_args(sc_address, raw_call_data);
        let contract_call_with_egld = contract_call.with_egld_transfer(egld_value);
        contract_call_with_egld
            .async_call_promise()
            .register_promise();

        // TODO: Add callback to promise
    }

    fn perform_raw_async_call_esdt(
        &self,
        sc_address: ManagedAddress,
        raw_call_data: RawCall<Self::Api>,
        esdt_payments: PaymentsVec<Self::Api>,
    ) {
        self.require_dest_not_self(&sc_address);
        self.require_sc_address(&sc_address);

        let contract_call = self.build_raw_call_with_args(sc_address, raw_call_data);
        if !esdt_payments.is_empty() {
            let contract_call_with_esdt = contract_call.with_multi_token_transfer(esdt_payments);

            contract_call_with_esdt
                .async_call_promise()
                .register_promise();
        } else {
            contract_call.async_call_promise().register_promise();
        }

        // TODO: Add callback to promise
    }

    #[promises_callback]
    fn raw_async_callback(
        &self,
        original_caller: ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
    }
}
