use multiversx_sc::{api::ManagedTypeApi, types::BigUint};

multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct UnstakedAmount<M: ManagedTypeApi> {
    pub epoch: u64,
    pub amount: BigUint<M>,
}
