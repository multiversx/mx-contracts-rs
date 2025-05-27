use super::common::{GasLimit, PaymentsVec, RawCall};

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

        let original_caller = self.blockchain().get_caller();
        let gas_for_callback = self.get_gas_for_callback(raw_call_data.gas_limit);

        let contract_call = self.build_raw_call_with_args(sc_address, raw_call_data);
        let contract_call_with_egld = contract_call.with_egld_transfer(egld_value.clone());
        contract_call_with_egld
            .async_call_promise()
            .with_callback(self.callbacks().raw_async_callback(
                original_caller,
                egld_value,
                PaymentsVec::new(),
            ))
            .with_extra_gas_for_callback(gas_for_callback)
            .register_promise();
    }

    fn perform_raw_async_call_esdt(
        &self,
        sc_address: ManagedAddress,
        raw_call_data: RawCall<Self::Api>,
        esdt_payments: PaymentsVec<Self::Api>,
    ) {
        self.require_dest_not_self(&sc_address);
        self.require_sc_address(&sc_address);
        self.require_not_empty_payments(&esdt_payments);

        let original_caller = self.blockchain().get_caller();
        let gas_for_callback = self.get_gas_for_callback(raw_call_data.gas_limit);

        let contract_call = self.build_raw_call_with_args(sc_address, raw_call_data);
        let contract_call_with_esdt =
            contract_call.with_multi_token_transfer(esdt_payments.clone());
        contract_call_with_esdt
            .async_call_promise()
            .with_callback(self.callbacks().raw_async_callback(
                original_caller,
                BigUint::zero(),
                esdt_payments,
            ))
            .with_extra_gas_for_callback(gas_for_callback)
            .register_promise();
    }

    #[inline]
    fn get_gas_for_callback(&self, full_gas_limit: GasLimit) -> GasLimit {
        full_gas_limit / 8
    }

    #[promises_callback]
    fn raw_async_callback(
        &self,
        original_caller: ManagedAddress,
        original_egld_value: BigUint,
        original_esdt_transfers: PaymentsVec<Self::Api>,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                let egld_amount = self.call_value().egld_direct_non_strict().clone_value();
                self.send()
                    .direct_non_zero_egld(&original_caller, &egld_amount);

                let esdt_transfers = self.call_value().all_esdt_transfers().clone_value();
                if !esdt_transfers.is_empty() {
                    self.send().direct_multi(&original_caller, &esdt_transfers);
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                self.send()
                    .direct_non_zero_egld(&original_caller, &original_egld_value);

                if !original_esdt_transfers.is_empty() {
                    self.send()
                        .direct_multi(&original_caller, &original_esdt_transfers);
                }
            }
        }
    }
}
