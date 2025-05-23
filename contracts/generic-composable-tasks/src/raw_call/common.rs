multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type GasLimit = u64;
pub type FunctionName<M> = ManagedBuffer<M>;
pub type RawArgs<M> = ManagedVec<M, ManagedBuffer<M>>;
pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

pub const MIN_GAS_LIMIT: GasLimit = 1_000_000;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct RawCall<M: ManagedTypeApi> {
    pub gas_limit: GasLimit,
    pub function_name: FunctionName<M>,
    pub args: RawArgs<M>,
}

#[multiversx_sc::module]
pub trait CommonModule {
    #[allow(deprecated)]
    fn build_raw_call_with_args(
        &self,
        sc_address: ManagedAddress,
        raw_call_data: RawCall<Self::Api>,
    ) -> ContractCallNoPayment<Self::Api, IgnoreValue> {
        require!(
            raw_call_data.gas_limit >= MIN_GAS_LIMIT,
            "Invalid gas limit"
        );

        let mut contract_call =
            ContractCallNoPayment::<_, IgnoreValue>::new(sc_address, raw_call_data.function_name);
        contract_call = contract_call.with_gas_limit(raw_call_data.gas_limit);

        for arg in raw_call_data.args {
            contract_call = contract_call.argument(&arg);
        }

        contract_call
    }

    #[inline]
    fn clear_back_transfers(&self) {
        let _ = self.blockchain().get_back_transfers();
    }

    fn require_dest_not_self(&self, dest_address: &ManagedAddress) {
        let own_sc_address = self.blockchain().get_sc_address();
        require!(
            dest_address != &own_sc_address,
            "May not have own SC as destination"
        );
    }

    fn require_sc_address(&self, dest_sc_address: &ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(dest_sc_address),
            "Invalid destination"
        );
    }

    fn require_not_empty_payments(&self, payments: &PaymentsVec<Self::Api>) {
        require!(!payments.is_empty(), "May not send empty ESDT payments");
    }
}
