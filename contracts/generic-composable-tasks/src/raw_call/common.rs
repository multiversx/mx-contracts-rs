multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type GasLimit = u64;
pub type FunctionName<M> = ManagedBuffer<M>;
pub type RawArgs<M> = ManagedVec<M, ManagedBuffer<M>>;
pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum PaymentType<M: ManagedTypeApi> {
    None,
    FixedPayments(PaymentsVec<M>),
    ReceivedPaymentsFromSc,
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct RawCall<M: ManagedTypeApi> {
    pub gas_limit: GasLimit,
    pub function_name: FunctionName<M>,
    pub args: RawArgs<M>,
}

#[multiversx_sc::module]
pub trait CommonModule {
    fn build_raw_call_with_args(
        &self,
        sc_address: ManagedAddress,
        raw_call_data: RawCall<Self::Api>,
    ) -> ContractCallNoPayment<Self::Api, IgnoreValue> {
        let mut contract_call =
            ContractCallNoPayment::<_, IgnoreValue>::new(sc_address, raw_call_data.function_name);
        contract_call = contract_call.with_gas_limit(raw_call_data.gas_limit);

        for arg in raw_call_data.args {
            contract_call = contract_call.argument(&arg);
        }

        contract_call
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
}
