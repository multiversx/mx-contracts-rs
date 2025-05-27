use super::common::{PaymentsVec, RawCall};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SyncCallModule: super::common::CommonModule {
    #[must_use]
    fn perform_raw_sync_call_egld(
        &self,
        sc_address: ManagedAddress,
        raw_call_data: RawCall<Self::Api>,
        egld_value: BigUint,
    ) -> BackTransfers<Self::Api> {
        self.require_dest_not_self(&sc_address);
        self.require_sc_address(&sc_address);

        let contract_call = self.build_raw_call_with_args(sc_address, raw_call_data);
        let contract_call_with_egld = contract_call.with_egld_transfer(egld_value);
        let (_, back_transfers): (IgnoreValue, _) =
            contract_call_with_egld.execute_on_dest_context_with_back_transfers();
        self.clear_back_transfers();

        back_transfers
    }

    #[must_use]
    fn perform_raw_sync_call_esdt(
        &self,
        sc_address: ManagedAddress,
        raw_call_data: RawCall<Self::Api>,
        esdt_payments: PaymentsVec<Self::Api>,
    ) -> BackTransfers<Self::Api> {
        self.require_dest_not_self(&sc_address);
        self.require_sc_address(&sc_address);
        self.require_not_empty_payments(&esdt_payments);

        let contract_call = self.build_raw_call_with_args(sc_address, raw_call_data);
        let contract_call_with_esdt = contract_call.with_multi_token_transfer(esdt_payments);
        let (_, back_transfers): (IgnoreValue, _) =
            contract_call_with_esdt.execute_on_dest_context_with_back_transfers();
        self.clear_back_transfers();

        back_transfers
    }
}
