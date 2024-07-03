use multiversx_sc::{api::ManagedTypeApi, types::EsdtTokenPayment};

use multiversx_sc::derive_imports::*;

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, ManagedVecItem)]
pub struct UnlockedToken<M: ManagedTypeApi> {
    pub token: EsdtTokenPayment<M>,
    pub unbond_epoch: u64,
}
