#![no_std]

multiversx_sc::imports!();

mod structs;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait ProxyActionsContract: structs::TaskCall {
    #[init]
    fn init(&self) {}
}
