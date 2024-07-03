use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Copy)]
pub enum ErcHookType {
    // can't be done, execute_on_dest does not work on init
    _BeforeInitialize,
    _AfterInitialize,
    BeforeTransfer,
    BeforeExchangeAction,
    AfterExchangeAction,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, PartialEq)]
pub struct Hook<M: ManagedTypeApi> {
    pub dest_address: ManagedAddress<M>,
    pub endpoint_name: ManagedBuffer<M>,
}

pub trait ErcHook {
    type Sc: ContractBase;

    fn before_transfer(
        sc: &Self::Sc,
        tokens: PaymentsVec<<Self::Sc as ContractBase>::Api>,
        original_caller: ManagedAddress<<Self::Sc as ContractBase>::Api>,
        dest: ManagedAddress<<Self::Sc as ContractBase>::Api>,
    );

    fn before_exchange_action(
        sc: &Self::Sc,
        tokens: PaymentsVec<<Self::Sc as ContractBase>::Api>,
        original_caller: ManagedAddress<<Self::Sc as ContractBase>::Api>,
        dest: ManagedAddress<<Self::Sc as ContractBase>::Api>,
        args: MultiValueEncoded<
            <Self::Sc as ContractBase>::Api,
            ManagedBuffer<<Self::Sc as ContractBase>::Api>,
        >,
    );

    fn after_exchange_action(
        sc: &Self::Sc,
        tokens: PaymentsVec<<Self::Sc as ContractBase>::Api>,
        original_caller: ManagedAddress<<Self::Sc as ContractBase>::Api>,
    );
}
