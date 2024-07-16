use multiversx_sc::{api::ManagedTypeApi, types::EsdtTokenPayment};

use multiversx_sc::derive_imports::*;

#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, ManagedVecItem)]
pub struct UnlockedToken<M: ManagedTypeApi> {
    pub token: EsdtTokenPayment<M>,
    pub unbond_epoch: u64,
}
