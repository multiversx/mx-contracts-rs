use multiversx_sc::{
    api::ManagedTypeApi,
    types::{BigUint, TokenIdentifier},
};

multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct Esdt<M: ManagedTypeApi> {
    pub token_identifier: TokenIdentifier<M>,
    pub amount: BigUint<M>,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct AvailableAmount<M: ManagedTypeApi> {
    pub epoch: u64,
    pub amount: BigUint<M>,
}
